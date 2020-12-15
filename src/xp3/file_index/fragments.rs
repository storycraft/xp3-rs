/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use byteorder::LittleEndian;
use encoding::{DecoderTrap, EncoderTrap, Encoding, all::UTF_16LE};

use crate::xp3::{XP3Error, XP3ErrorKind};
use byteorder::{ReadBytesExt, WriteBytesExt};

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum IndexInfoFlag {

    NotProtected = 0,
    Protected = 2147483648

}

#[derive(Debug, Clone)]
/// Index info represents file metadatas like name.
pub struct XP3FileIndexInfo {

    flag: IndexInfoFlag,
    file_size: u64,
    saved_file_size: u64,
    name: String

}

impl XP3FileIndexInfo {

    pub fn new(
        flag: IndexInfoFlag,
        file_size: u64,
        saved_file_size: u64,
        name: String
    ) -> Self {
        Self {
            flag,
            file_size,
            saved_file_size,
            name
        }
    }

    pub fn flag(&self) -> IndexInfoFlag {
        self.flag
    }

    pub fn set_flag(&mut self, flag: IndexInfoFlag) {
        self.flag = flag;
    }

    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    pub fn set_file_size(&mut self, file_size: u64) {
        self.file_size = file_size;
    }

    pub fn saved_file_size(&self) -> u64 {
        self.saved_file_size
    }

    pub fn set_saved_file_size(&mut self, saved_file_size: u64) {
        self.saved_file_size = saved_file_size;
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Read xp3 file index info from stream.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let flag = {
            let raw_flag = stream.read_u32::<LittleEndian>()?;

            if raw_flag == IndexInfoFlag::NotProtected as u32 {
                Ok(IndexInfoFlag::NotProtected)
            } else if raw_flag == IndexInfoFlag::Protected as u32 {
                Ok(IndexInfoFlag::Protected)
            } else {
                Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None))
            }
        }?;

        let file_size = stream.read_u64::<LittleEndian>()?;
        let saved_file_size = stream.read_u64::<LittleEndian>()?;
        
        let name_byte_size = stream.read_u16::<LittleEndian>()? * 2;
        let name = {
            let mut name_buffer = Vec::with_capacity(name_byte_size as usize);
            stream.take(name_byte_size as u64).read_to_end(&mut name_buffer)?;
            
            UTF_16LE.decode(&name_buffer, DecoderTrap::Replace).unwrap()
        };

        Ok((22 + name_byte_size as u64, Self::new(flag, file_size, saved_file_size, name)))
    }

    /// Write xp3 file index info to stream.
    /// Returns written size.
    pub fn write_bytes<T: Write>(&self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u32::<LittleEndian>(self.flag as u32)?;
        
        stream.write_u64::<LittleEndian>(self.file_size)?;
        stream.write_u64::<LittleEndian>(self.saved_file_size)?;

        let encoded = UTF_16LE.encode(&self.name, EncoderTrap::Replace).unwrap();
        let name_byte_size = encoded.len().min(65536);
        stream.write_u16::<LittleEndian>(name_byte_size as u16)?;
        stream.write_all(&encoded[..name_byte_size])?;

        Ok(22 + name_byte_size as u64)
    }

}

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum IndexSegmentFlag {

    UnCompressed = 0,
    Compressed = 1

}

#[derive(Debug, Copy, Clone)]
/// Index segments representing data fragments of target file.
/// Almost all archive have 1 segments each files but there can be more.
pub struct XP3FileIndexSegment {

    flag: IndexSegmentFlag,
    data_offset: u64,
    original_size: u64,
    saved_size: u64

}

impl XP3FileIndexSegment {

    pub fn new(
        flag: IndexSegmentFlag,
        data_offset: u64,
        original_size: u64,
        saved_size: u64
    ) -> Self {
        Self {
            flag, data_offset, original_size, saved_size
        }
    }

    pub fn flag(&self) -> IndexSegmentFlag {
        self.flag
    }

    pub fn set_flag(&mut self, flag: IndexSegmentFlag) {
        self.flag = flag;
    }

    pub fn data_offset(&self) -> u64 {
        self.data_offset
    }

    pub fn set_data_offset(&mut self, data_offset: u64) {
        self.data_offset = data_offset;
    }

    pub fn original_size(&self) -> u64 {
        self.original_size
    }

    pub fn set_original_size(&mut self, original_size: u64) {
        self.original_size = original_size;
    }

    pub fn saved_size(&self) -> u64 {
        self.saved_size
    }

    pub fn set_saved_size(&mut self, saved_size: u64) {
        self.saved_size = saved_size;
    }

    /// Read xp3 file index segment from stream.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let flag = {
            let raw_flag = stream.read_u32::<LittleEndian>()?;

            if raw_flag == IndexSegmentFlag::UnCompressed as u32 {
                Ok(IndexSegmentFlag::UnCompressed)
            } else if raw_flag == IndexSegmentFlag::Compressed as u32 {
                Ok(IndexSegmentFlag::Compressed)
            } else {
                Err(XP3Error::new(XP3ErrorKind::InvalidFileIndex, None))
            }
        }?;

        let data_offset = stream.read_u64::<LittleEndian>()?;
        let original_size = stream.read_u64::<LittleEndian>()?;
        let saved_size = stream.read_u64::<LittleEndian>()?;

        Ok((28, Self::new(flag, data_offset, original_size, saved_size)))
    }

    /// Write xp3 file index segment to stream.
    /// Returns written size.
    pub fn write_bytes<T: Write>(&self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u32::<LittleEndian>(self.flag as u32)?;

        stream.write_u64::<LittleEndian>(self.data_offset)?;
        stream.write_u64::<LittleEndian>(self.original_size)?;
        stream.write_u64::<LittleEndian>(self.saved_size)?;

        Ok(28)
    }

}

#[derive(Debug, Copy, Clone)]
/// Index time representing timestamp of target file.
pub struct XP3FileIndexTime {

    timestamp: u64

}

impl XP3FileIndexTime {

    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp
        }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }

    /// Read xp3 file index time from stream.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let timestamp = stream.read_u64::<LittleEndian>()?;

        Ok((8, Self::new(timestamp)))
    }

    /// Write xp3 file time to stream.
    pub fn write_bytes<T: Write>(&self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u64::<LittleEndian>(self.timestamp)?;

        Ok(8)
    }

}

#[derive(Debug, Copy, Clone)]
/// Index adler representing adler32 checksum of target file.
pub struct XP3FileIndexAdler {

    checksum: u32

}

impl XP3FileIndexAdler {

    pub fn new(checksum: u32) -> Self {
        Self {
            checksum
        }
    }

    pub fn checksum(&self) -> u32 {
        self.checksum
    }

    pub fn set_checksum(&mut self, checksum: u32) {
        self.checksum = checksum;
    }

    /// Read xp3 file index time from stream.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let checksum = stream.read_u32::<LittleEndian>()?;

        Ok((4, Self::new(checksum)))
    }

    /// Write xp3 file time to stream.
    pub fn write_bytes<T: Write>(&self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u32::<LittleEndian>(self.checksum)?;

        Ok(4)
    }

}