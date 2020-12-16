/*
 * Created on Mon Dec 14 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{ReadBytesExt, LittleEndian};
use flate2::read::ZlibDecoder;

use super::{VirtualXP3, XP3Error, XP3ErrorKind, XP3_MAGIC, archive::XP3Archive, header::{XP3Header, XP3HeaderVersion}, index::file::{IndexSegmentFlag, XP3FileIndexSegment}, index_set::XP3IndexSet};

pub struct XP3Reader;

impl XP3Reader {

    /// Open XP3 archive.
    pub fn open_archive<T: Read + Seek>(mut stream: T) -> Result<XP3Archive<T>, XP3Error> {
        Ok(XP3Archive::new(Self::read_container(&mut stream)?, stream))
    }

    /// Read xp3 container using stream.
    pub fn read_container<T: Read + Seek>(stream: &mut T) -> Result<VirtualXP3, XP3Error> {
        let current = stream.seek(SeekFrom::Current(0))?;

        Self::check_archive(stream)?;

        // This is 0x01 currently
        let _ = stream.read_u8()?;

        let (_, header) = XP3Header::from_bytes(stream)?;

        // Move to index part
        let index_offset = match header.version() {
            XP3HeaderVersion::Old => Ok(0_u64),

            XP3HeaderVersion::Current { minor_version: _, index_size_offset } => {
                stream.seek(SeekFrom::Current(index_size_offset as i64))?;

                stream.read_u64::<LittleEndian>()
            }
        }?;
        stream.seek(SeekFrom::Start(current + index_offset))?;

        let (_, index_set) = XP3IndexSet::from_bytes(stream)?;

        // Reset read position to start.
        stream.seek(SeekFrom::Start(current))?;

        Ok(VirtualXP3::new(header, index_set))
    }

    /// Read data from provided segment.
    pub fn read_segment<O: Read + Seek, T: Write>(segment: &XP3FileIndexSegment, from: &mut O, stream: &mut T) -> Result<(), XP3Error> {
        let read_size = segment.saved_size();
        let read_offset = segment.data_offset();

        let mut buffer = Vec::with_capacity(read_size as usize);

        let pos = from.seek(SeekFrom::Current(read_offset as i64))?;
        from.take(read_size).read_to_end(&mut buffer)?;
        from.seek(SeekFrom::Start(pos - read_offset))?;

        match segment.flag() {
            IndexSegmentFlag::UnCompressed => stream.write_all(&mut buffer[..segment.saved_size() as usize])?,

            IndexSegmentFlag::Compressed => {
                let mut uncompressed = Vec::<u8>::with_capacity(segment.original_size() as usize);

                let decoder = ZlibDecoder::new(&buffer[..]);

                decoder.take(segment.original_size()).read_to_end(&mut uncompressed)?;

                stream.write_all(&mut uncompressed)?;
            }
        }

        Ok(())
    }

    /// Check if stream is valid xp3 archive or not.
    /// Will read 11 bytes from current position.
    pub fn check_archive(stream: &mut impl Read) -> Result<(), XP3Error> {
        let mut magic_buffer = [0_u8; 10];

        stream.read_exact(&mut magic_buffer)?;

        if magic_buffer != XP3_MAGIC {
            return Err(XP3Error::new(XP3ErrorKind::InvalidFile, None));
        }

        Ok(())
    }

}