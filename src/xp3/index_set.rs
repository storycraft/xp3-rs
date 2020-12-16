/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{collections::{HashMap, hash_map::{Iter, Values}}, io::{Cursor, Read, Write}};

use byteorder::LittleEndian;
use flate2::{Compression, read::{ZlibDecoder, ZlibEncoder}};

use super::{XP3Error, XP3ErrorKind, XP3_INDEX_FILE_IDENTIFIER, index::{XP3Index, file::XP3FileIndex}};

use byteorder::{WriteBytesExt, ReadBytesExt};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum XP3IndexCompression {

    UnCompressed = 0,

    Compressed = 1

}

#[derive(Debug)]
pub struct XP3IndexSet {

    compression: XP3IndexCompression,

    extra: Vec<XP3Index>,

    file_map: HashMap<String, XP3FileIndex>

}

impl XP3IndexSet {

    pub fn new(
        compression: XP3IndexCompression,
        extra: Vec<XP3Index>,
        file_map: HashMap<String, XP3FileIndex>
    ) -> Self {
        Self {
            compression,
            extra,
            file_map
        }
    }

    pub fn compression(&self) -> XP3IndexCompression {
        self.compression
    }

    pub fn set_compression(&mut self, compression: XP3IndexCompression) {
        self.compression = compression;
    }

    pub fn indices(&self) -> Values<String, XP3FileIndex> {
        self.file_map.values()
    }

    pub fn extra(&self) -> &Vec<XP3Index> {
        &self.extra
    }

    pub fn entries(&self) -> Iter<String, XP3FileIndex> {
        self.file_map.iter()
    }

    pub fn get(&self, name: &String) -> Option<&XP3FileIndex> {
        self.file_map.get(name)
    }

    /// Read xp3 file index set from stream.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let flag = {
            let raw_flag = stream.read_u8()?;

            if raw_flag == XP3IndexCompression::UnCompressed as u8 {
                Ok(XP3IndexCompression::UnCompressed)
            // Compressed
            } else if raw_flag == XP3IndexCompression::Compressed as u8 {
                Ok(XP3IndexCompression::Compressed)
            // Unknown
            } else {
                Err(XP3Error::new(XP3ErrorKind::InvalidFileIndexHeader, None))
            }
        }?;

        let index_data: Vec<u8>;
        let index_size: u64;
        let raw_index_read: u64;
        match flag {
            XP3IndexCompression::UnCompressed => {
                index_size = stream.read_u64::<LittleEndian>()?;

                let mut data = Vec::<u8>::with_capacity(index_size as usize);
                stream.take(index_size).read_to_end(&mut data)?;

                index_data = data;
                raw_index_read = index_size;
            },

            XP3IndexCompression::Compressed => {
                let compressed_size = stream.read_u64::<LittleEndian>()?;
                index_size = stream.read_u64::<LittleEndian>()?;

                let mut compressed_buffer = Vec::<u8>::with_capacity(compressed_size as usize);
                stream.take(compressed_size).read_to_end(&mut compressed_buffer)?;

                let decoder = ZlibDecoder::new(&compressed_buffer[..]);
    
                let mut data = Vec::new();
                decoder.take(index_size).read_to_end(&mut data)?;
    
                index_data = data;
                raw_index_read = compressed_size;
            }
        }

        let mut index_buffer = Cursor::new(index_data);

        let mut file_map: HashMap<String, XP3FileIndex> = HashMap::new();

        let mut extra: Vec<XP3Index> = Vec::new();

        let mut index_read: u64 = 0;
        while index_read < index_size {
            let (read, mut index) = XP3Index::from_bytes(&mut index_buffer)?;
            
            match index.identifier() {

                XP3_INDEX_FILE_IDENTIFIER => {
                    let (_, file_index) = XP3FileIndex::from_bytes(index.data().len() as u64, &mut Cursor::new(index.data_mut()))?;
                    file_map.insert(file_index.info().name().clone(), file_index);
                }

                _ => {
                    extra.push(index);
                }
            }

            
            index_read += read;
        }

        Ok((raw_index_read + index_size, Self::new(flag, extra, file_map)))
    }

    /// Write xp3 file index set to stream.
    pub fn write_bytes<T: Write>(&mut self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u8(self.compression as u8)?;

        let mut index_buffer = Cursor::new(Vec::<u8>::new());

        for extra_index in self.extra.iter_mut() {
            extra_index.write_bytes(&mut index_buffer)?;
        }

        for index in self.file_map.values() {
            let mut buffer = Vec::<u8>::new();
            index.write_bytes(&mut buffer)?;
            XP3Index::new(XP3_INDEX_FILE_IDENTIFIER, buffer).write_bytes(&mut index_buffer)?;
        }

        match self.compression {
            XP3IndexCompression::UnCompressed => {
                stream.write_all(index_buffer.get_mut())?;

                Ok(1 + index_buffer.into_inner().len() as u64)
            },

            XP3IndexCompression::Compressed => {
                let mut compressed_data = Vec::<u8>::new();

                // Reset read stream
                let mut encoder = ZlibEncoder::new(Cursor::new(index_buffer.into_inner()), Compression::fast());

                encoder.read_to_end(&mut compressed_data)?;

                stream.write_u64::<LittleEndian>(compressed_data.len() as u64)?;
                stream.write_u64::<LittleEndian>(encoder.get_ref().get_ref().len() as u64)?;

                stream.write_all(&mut compressed_data)?;

                Ok(1 + compressed_data.len() as u64)   
            }
        }
    }

}