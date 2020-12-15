/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod fragments;

use std::{collections::{HashMap, hash_map::{Iter, Values}}, io::{Cursor, Read, Seek, SeekFrom, Write}};

use byteorder::LittleEndian;
use flate2::{Compression, bufread::ZlibEncoder, read::ZlibDecoder};

use self::fragments::{XP3FileIndexAdler, XP3FileIndexInfo, XP3FileIndexSegment, XP3FileIndexTime};

use super::{XP3Error, XP3ErrorKind, XP3_INDEX_ADLR_IDENTIFIER, XP3_INDEX_FILE_IDENTIFIER, XP3_INDEX_INFO_IDENTIFIER, XP3_INDEX_TIME_IDENTIFIER, XP3_INDEX_SEGM_IDENTIFIER};

use byteorder::{WriteBytesExt, ReadBytesExt};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum XP3FileIndexCompression {

    UnCompressed = 0,

    Compressed = 1

}
 
/// FileIndex for xp3 archive.
/// Contains information about file and data offsets.
#[derive(Debug, Clone)]
pub struct XP3FileIndex {

    info: XP3FileIndexInfo,
    segments: Vec<XP3FileIndexSegment>,
    adler: XP3FileIndexAdler,
    time: Option<XP3FileIndexTime>

}

impl XP3FileIndex {

    pub fn new(
        info: XP3FileIndexInfo,
        segments: Vec<XP3FileIndexSegment>,
        adler: XP3FileIndexAdler,
        time: Option<XP3FileIndexTime>
    ) -> Self {
        Self {
            info,
            segments,
            adler,
            time
        }
    }

    /// File time
    pub fn time(&self) -> Option<XP3FileIndexTime> {
        self.time
    }

    /// File adler hash
    pub fn adler(&self) -> XP3FileIndexAdler {
        self.adler
    }

    /// File information
    pub fn info(&self) -> &XP3FileIndexInfo {
        &self.info
    }

    /// File segments
    pub fn segments(&self) -> &Vec<XP3FileIndexSegment> {
        &self.segments
    }

    /// Read xp3 file index from stream.
    /// Returns read size, XP3FileIndex tuple.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let file_identifier = stream.read_u32::<LittleEndian>()?;

        if file_identifier != XP3_INDEX_FILE_IDENTIFIER {
            return Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None));
        }

        let file_size = stream.read_u64::<LittleEndian>()?;

        let mut info: Option<XP3FileIndexInfo> = None;
        let mut segm: Option<Vec<XP3FileIndexSegment>> = None;
        let mut time: Option<XP3FileIndexTime> = None;
        let mut adler32: Option<XP3FileIndexAdler> = None;

        let mut total_read: u64 = 0;
        while total_read < file_size {
            let identifier: u32 = stream.read_u32::<LittleEndian>()?;
            let size: u64 = stream.read_u64::<LittleEndian>()?;

            let mut data = Vec::with_capacity(size as usize);
            stream.take(size).read_to_end(&mut data)?;
            let mut data_stream = Cursor::new(data);

            match identifier {

                XP3_INDEX_INFO_IDENTIFIER => {
                    if info.is_some() {
                        return Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None));
                    }

                    info = Some(XP3FileIndexInfo::from_bytes(&mut data_stream)?.1);
                },

                XP3_INDEX_SEGM_IDENTIFIER => {
                    if segm.is_some() {
                        return Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None));
                    }

                    let count = data_stream.get_ref().len() / 28;

                    let mut list: Vec<XP3FileIndexSegment> = Vec::new();
                    for _ in 0..count {
                        list.push(XP3FileIndexSegment::from_bytes(&mut data_stream)?.1);
                    }

                    segm = Some(list);
                },

                XP3_INDEX_ADLR_IDENTIFIER => {
                    if size != 4 || adler32.is_some() {
                        return Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None));
                    }

                    adler32 = Some(XP3FileIndexAdler::from_bytes(&mut data_stream)?.1);
                },

                XP3_INDEX_TIME_IDENTIFIER => {
                    if size != 8 {
                        return Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None));
                    }

                    time = Some(XP3FileIndexTime::from_bytes(&mut data_stream)?.1);
                },
                
                _ => {
                    // SKIP infos we don't know yet.
                }
            }

            
            total_read += size + 12;
        }

        if info.is_none() || adler32.is_none() || segm.is_none() {
            return Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None));
        }

        Ok((total_read + 12, XP3FileIndex::new(info.unwrap(), segm.unwrap(), adler32.unwrap(), time)))
    }

    /// Write xp3 file index to stream.
    pub fn write_bytes<T: Write>(&self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u32::<LittleEndian>(XP3_INDEX_FILE_IDENTIFIER)?;
        
        let mut file_buffer = Cursor::new(Vec::<u8>::new());
        {
            let mut buffer = Cursor::new(Vec::<u8>::new());
            self.adler.write_bytes(&mut buffer)?;
            Self::write_part(XP3_INDEX_ADLR_IDENTIFIER, buffer.get_ref(), &mut file_buffer)?;
        }

        if self.time.is_some() {
            let mut buffer = Cursor::new(Vec::<u8>::new());
            self.time.as_ref().unwrap().write_bytes(&mut buffer)?;
            Self::write_part(XP3_INDEX_TIME_IDENTIFIER, buffer.get_ref(), &mut file_buffer)?;
        }

        {
            let mut buffer = Cursor::new(Vec::<u8>::new());
            self.info.write_bytes(&mut buffer)?;
            Self::write_part(XP3_INDEX_INFO_IDENTIFIER, buffer.get_ref(), &mut file_buffer)?;
        }

        {
            let mut buffer = Cursor::new(Vec::<u8>::new());
            for segment in self.segments.iter() {
                segment.write_bytes(&mut buffer)?;
            }

            Self::write_part(XP3_INDEX_SEGM_IDENTIFIER, buffer.get_ref(), &mut file_buffer)?;
        }

        let mut file_index_data = file_buffer.into_inner();

        stream.write_u64::<LittleEndian>(file_index_data.len() as u64)?;
        stream.write_all(&mut file_index_data)?;

        Ok(file_index_data.len() as u64 + 12)
    }

    fn write_part<T: Write>(identifier: u32, data: &[u8], buffer: &mut T) -> Result<u64, XP3Error> {
        buffer.write_u32::<LittleEndian>(identifier)?;
        
        let data_length = data.len() as u64;
        buffer.write_u64::<LittleEndian>(data_length)?;
        buffer.write_all(data)?;

        Ok(12 + data_length)

    }

}

#[derive(Debug)]
pub struct XP3FileIndexSet {

    compression: XP3FileIndexCompression,

    extra_data: Option<(u32, Vec<u8>)>,

    indices: HashMap<String, XP3FileIndex>

}

impl XP3FileIndexSet {

    pub fn new(
        compression: XP3FileIndexCompression,
        extra_data: Option<(u32, Vec<u8>)>,
        indices: HashMap<String, XP3FileIndex>
    ) -> Self {
        Self {
            compression,
            extra_data,
            indices
        }
    }

    pub fn compression(&self) -> XP3FileIndexCompression {
        self.compression
    }

    pub fn set_compression(&mut self, compression: XP3FileIndexCompression) {
        self.compression = compression;
    }

    pub fn indices(&self) -> Values<String, XP3FileIndex> {
        self.indices.values()
    }

    pub fn extra_data(&self) -> &Option<(u32, Vec<u8>)> {
        &self.extra_data
    }

    pub fn entries(&self) -> Iter<String, XP3FileIndex> {
        self.indices.iter()
    }

    pub fn get(&self, name: &String) -> Option<&XP3FileIndex> {
        self.indices.get(name)
    }

    /// Read xp3 file index set from stream.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let flag = {
            let raw_flag = stream.read_u8()?;

            if raw_flag == XP3FileIndexCompression::UnCompressed as u8 {
                Ok(XP3FileIndexCompression::UnCompressed)
            // Compressed
            } else if raw_flag == XP3FileIndexCompression::Compressed as u8 {
                Ok(XP3FileIndexCompression::Compressed)
            // Unknown
            } else {
                Err(XP3Error::new(XP3ErrorKind::InvalidFileIndexHeader, None))
            }
        }?;

        let index_data: Vec<u8>;
        let index_size: u64;
        let raw_index_read: u64;
        match flag {
            XP3FileIndexCompression::UnCompressed => {
                index_size = stream.read_u64::<LittleEndian>()?;

                let mut data = Vec::<u8>::with_capacity(index_size as usize);
                stream.take(index_size).read_to_end(&mut data)?;

                index_data = data;
                raw_index_read = index_size;
            },

            XP3FileIndexCompression::Compressed => {
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

        let mut map: HashMap<String, XP3FileIndex> = HashMap::new();

        let index_start = index_buffer.seek(SeekFrom::Current(0))?;

        let first = index_buffer.read_u32::<LittleEndian>()?;

        let mut extra: Option<(u32, Vec<u8>)> = None;

        let file_index_size;
        if first != XP3_INDEX_FILE_IDENTIFIER {
            let extra_length = index_buffer.read_u64::<LittleEndian>()?;
            
            let mut extra_data = Vec::with_capacity(extra_length as usize);
            (&mut index_buffer).take(extra_length).read_to_end(&mut extra_data)?;

            extra = Some((first, extra_data));
            file_index_size = index_size - extra_length - 12;
        } else {
            index_buffer.seek(SeekFrom::Start(index_start))?;
            file_index_size = index_size;
        }

        let mut index_read: u64 = 0;
        while index_read < file_index_size {
            let (read, index) = XP3FileIndex::from_bytes(&mut index_buffer)?;

            map.insert(index.info().name().clone(), index);
            index_read += read;
        }

        Ok((raw_index_read + index_size, Self::new(flag, extra, map)))
    }

    /// Write xp3 file index set to stream.
    pub fn write_bytes<T: Write>(&self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u8(self.compression as u8)?;

        let mut index_buffer = Cursor::new(Vec::<u8>::new());

        match &self.extra_data {
            Some(extra_data) => {
                index_buffer.write_u32::<LittleEndian>(extra_data.0)?;
                index_buffer.write_u64::<LittleEndian>(extra_data.1.len() as u64)?;
                index_buffer.write_all(&extra_data.1)?;
            },

            None => {}
        }

        for index in self.indices.values() {
            index.write_bytes(&mut index_buffer)?;
        }

        match self.compression {
            XP3FileIndexCompression::UnCompressed => {
                stream.write_all(index_buffer.get_mut())?;

                Ok(1 + index_buffer.into_inner().len() as u64)
            },

            XP3FileIndexCompression::Compressed => {
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