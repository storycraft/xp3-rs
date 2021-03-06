/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::XP3Error;

pub mod file;

#[derive(Debug, Clone)]
/// General XP3 index
pub struct XP3Index {

    pub identifier: u32,
    pub data: Vec<u8>

}

impl XP3Index {

    /// Read xp3 index from stream.
    /// Returns read size, XP3Index tuple.
    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, Self), XP3Error> {
        let identifier: u32 = stream.read_u32::<LittleEndian>()?;
        let size: u64 = stream.read_u64::<LittleEndian>()?;
        let mut data = Vec::<u8>::with_capacity(size as usize);

        stream.take(size).read_to_end(&mut data)?;

        Ok((12 + size, Self { identifier, data }))
    }

    /// Write xp3 index to stream.
    pub fn write_bytes(&mut self, stream: &mut impl Write) -> Result<u64, XP3Error> {
        stream.write_u32::<LittleEndian>(self.identifier)?;
        let len = self.data.len() as u64;
        stream.write_u64::<LittleEndian>(len)?;
        stream.write_all(&mut self.data)?;

        Ok(12 + len)
    }

}