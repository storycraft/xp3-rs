/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::LittleEndian;

use super::{XP3Error, XP3ErrorKind, XP3_CURRENT_VER_IDENTIFIER, XP3_VERSION_IDENTIFIER};
use byteorder::{WriteBytesExt, ReadBytesExt};
 
#[derive(Debug, Copy, Clone)]
pub struct XP3Header {

    version: XP3HeaderVersion

}

impl XP3Header {

    pub fn new(version: XP3HeaderVersion) -> Self {
        Self {
            version
        }
    }

    pub fn version(&self) -> XP3HeaderVersion {
        self.version
    }

    pub fn set_version(&mut self, version: XP3HeaderVersion) {
        self.version = version;
    }

    /// Read xp3 header from current position.
    /// Returns read size, XP3header tuple.
    pub fn from_bytes<T: Read + Seek>(stream: &mut T) -> Result<(u64, Self), XP3Error> {
        let current = stream.seek(SeekFrom::Current(0))?;

        let identifier = stream.read_u64::<LittleEndian>()?;

        let header: XP3Header;
        let read: u64;
        if identifier == XP3_CURRENT_VER_IDENTIFIER {
            let minor_version = stream.read_u32::<LittleEndian>()?;

            if stream.read_u8()? != XP3_VERSION_IDENTIFIER {
                return Err(XP3Error::new(XP3ErrorKind::InvalidHeader, None));
            }
            
            header = XP3Header::new(
                XP3HeaderVersion::Current {
                    minor_version,
                    index_size_offset: stream.read_u64::<LittleEndian>()?
                }
            );

            read = 21;
        } else {
            stream.seek(SeekFrom::Start(current))?;
            header = XP3Header::new(
                XP3HeaderVersion::Old
            );
            read = 0;
        }

        Ok((read, header))
    }

    /// Write xp3 header to stream.
    /// Returns written size.
    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, XP3Error> {
        let ver_written: u64;
        match self.version {
            XP3HeaderVersion::Old => {
                ver_written = 0;
            },

            XP3HeaderVersion::Current { minor_version, index_size_offset } => {
                stream.write_u64::<LittleEndian>(XP3_CURRENT_VER_IDENTIFIER)?;
                stream.write_u32::<LittleEndian>(minor_version)?;
                stream.write_u8(XP3_VERSION_IDENTIFIER)?;
                stream.write_u64::<LittleEndian>(index_size_offset)?;
                ver_written = 21;
            }
        }

        Ok(ver_written)
    }

}

#[derive(Debug, Copy, Clone)]
/// Header starting with XP3_HEADER_IDENTIFIER is Current, otherwise Old.
pub enum XP3HeaderVersion {

    Old,

    Current {

        minor_version: u32,
        index_size_offset: u64

    }

}