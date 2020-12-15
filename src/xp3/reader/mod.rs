/*
 * Created on Mon Dec 14 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LittleEndian};

use super::{XP3Error, XP3ErrorKind, XP3_MAGIC, archive::XP3Archive, file_index::XP3FileIndexSet, header::{XP3Header, XP3HeaderVersion}};

pub struct XP3Reader;

impl XP3Reader {

    /// Reads XP3 archive and returns XP3Archive struct.
    pub fn read_archive<T: Read + Seek>(mut stream: T) -> Result<XP3Archive<T>, XP3Error> {
        let current = stream.seek(SeekFrom::Current(0))?;

        Self::check_archive(&mut stream)?;

        let (_, header) = XP3Header::from_bytes(&mut stream)?;

        // Move to index part
        let index_offset = match header.version() {
            XP3HeaderVersion::Old => Ok(0_u64),

            XP3HeaderVersion::Current { minor_version: _, index_size_offset } => {
                stream.seek(SeekFrom::Current(index_size_offset as i64))?;

                stream.read_u64::<LittleEndian>()
            }
        }?;
        stream.seek(SeekFrom::Start(current + index_offset))?;

        let (_, file_index_set) = XP3FileIndexSet::from_bytes(&mut stream)?;

        // Reset read position to start.
        stream.seek(SeekFrom::Start(current))?;

        Ok(XP3Archive::new(header, file_index_set, stream))
    }

    /// Check if stream is valid xp3 archive or not.
    /// Will read 11 bytes from current position.
    pub fn check_archive<T: Read>(stream: &mut T) -> Result<(), XP3Error> {
        let mut magic_buffer = [0_u8; 11];

        stream.read_exact(&mut magic_buffer)?;

        if magic_buffer != XP3_MAGIC {
            return Err(XP3Error::new(XP3ErrorKind::InvalidFile, None));
        }

        Ok(())
    }

}