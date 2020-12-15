/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::XP3Error;

pub mod file;

#[derive(Debug)]
/// General XP3 index
pub struct XP3Index {

    identifier: u32,
    data: Vec<u8>

}

impl XP3Index {

    pub fn new(identifier: u32, data: Vec<u8>) -> Self {
        Self {
            identifier,
            data
        }
    }

    pub fn identifier(&self) -> u32 {
        self.identifier
    }

    pub fn set_identifier(&mut self, identifier: u32) {
        self.identifier = identifier;
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Read xp3 index from stream.
    /// Returns read size, XP3Index tuple.
    pub fn from_bytes<T: Read>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let identifier: u32 = stream.read_u32::<LittleEndian>()?;
        let size: u64 = stream.read_u64::<LittleEndian>()?;
        let mut data = Vec::<u8>::with_capacity(size as usize);

        stream.take(size).read_to_end(&mut data)?;

        Ok((12 + size, Self::new(identifier, data)))
    }

    /// Write xp3 index to stream.
    pub fn write_bytes<T: Write>(&mut self, stream: &mut T) -> Result<u64, XP3Error> {
        stream.write_u32::<LittleEndian>(self.identifier)?;
        let len = self.data.len() as u64;
        stream.write_u64::<LittleEndian>(len)?;
        stream.write_all(&mut self.data)?;

        Ok(12 + len)
    }

}