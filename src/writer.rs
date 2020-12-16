/*
 * Created on Mon Dec 14 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{collections::HashMap, io::{self, Read, Seek, SeekFrom, Write}};

use adler32::RollingAdler32;
use byteorder::{WriteBytesExt, LittleEndian};
use flate2::{Compression, read::ZlibEncoder};

use super::{VirtualXP3, XP3Error, XP3_MAGIC, header::{XP3Header, XP3HeaderVersion}, index::{XP3Index, file::{IndexInfoFlag, IndexSegmentFlag, XP3FileIndex, XP3FileIndexAdler, XP3FileIndexInfo, XP3FileIndexSegment, XP3FileIndexTime}}, index_set::{XP3IndexCompression, XP3IndexSet}};

pub struct XP3Writer<T: Write + Seek> {

    stream: T,

    // Position of start
    start_pos: u64,

    // Position of index offset writing
    index_pos: u64,

    header: XP3Header,
    index_set: XP3IndexSet

}

impl<T: Write + Seek> XP3Writer<T> {

    /// Start new XP3 file writing
    pub fn start(
        mut stream: T,
        version: XP3HeaderVersion,
        index_compression: XP3IndexCompression
    ) -> Result<Self, XP3Error> {
        let header = XP3Header::new(version);
        let start_pos = stream.seek(SeekFrom::Current(0))?;

        // Write magic
        stream.write_all(&XP3_MAGIC)?;
        // Fixed version
        stream.write_u8(1)?;
        // Write header
        header.write_bytes(&mut stream)?;
        // Save position
        let index_pos = stream.seek(SeekFrom::Current(0))?;
        // index offset will be overridden after data stream written.
        stream.write_u64::<LittleEndian>(0)?;

        Ok(Self {
            stream,
            start_pos,
            index_pos,
            header,
            index_set: XP3IndexSet::new(index_compression, Vec::new(), HashMap::new())
        })
    }

    pub fn append_extra_index(&mut self, index: XP3Index) {
        self.index_set.extra_mut().push(index);
    }

    pub fn current_pos(&mut self) -> u64 {
        self.stream.seek(SeekFrom::Current(0)).unwrap()
    }

    /// Enter file entry start
    pub fn enter_file(
        &mut self,
        protection: IndexInfoFlag,
        name: String,
        timestamp: Option<u64>
    ) -> EntryWriter<'_, T> {
        EntryWriter::new(self, protection, name, timestamp)
    }

    /// Append indexes and finish file
    pub fn finish(mut self) -> Result<(VirtualXP3, T), XP3Error> {
        let current = self.stream.seek(SeekFrom::Current(0))?;

        self.stream.seek(SeekFrom::Start(self.index_pos))?;
        // Write index offset
        self.stream.write_u64::<LittleEndian>(current - self.start_pos)?;

        self.stream.seek(SeekFrom::Start(current))?;

        // Write indexes
        self.index_set.write_bytes(&mut self.stream)?;

        Ok((VirtualXP3::new(self.header, self.index_set), self.stream))
    }

}

pub struct EntryWriter<'a, T: Write + Seek> {

    writer: &'a mut XP3Writer<T>,

    protection: IndexInfoFlag,
    name: String,
    timestamp: Option<u64>,

    saved_size: u64,
    original_size: u64,
    adler: RollingAdler32,

    default_flag: IndexSegmentFlag,

    written_segments: Vec<XP3FileIndexSegment>

}

impl<'a, T: Write + Seek> EntryWriter<'a, T> {

    pub fn new(
        writer: &'a mut XP3Writer<T>,
        protection: IndexInfoFlag,
        name: String,
        timestamp: Option<u64>
    ) -> Self {
        Self {
            writer,
            protection,
            name,
            timestamp,
            saved_size: 0,
            original_size: 0,
            adler: RollingAdler32::new(),
            default_flag: IndexSegmentFlag::UnCompressed,
            written_segments: Vec::new()
        }
    }

    pub fn default_flag(&self) -> IndexSegmentFlag {
        self.default_flag
    }

    pub fn set_default_flag(&mut self, flag: IndexSegmentFlag) {
        self.default_flag = flag;
    }

    /// Write one segment from stream.
    /// Returns write size.
    pub fn write_segment(&mut self, flag: IndexSegmentFlag, data: &[u8]) -> Result<u64, io::Error> {
        self.adler.update_buffer(&data);

        let original_size: u64 = data.len() as u64;
        let saved_size: u64;
        match flag {
            IndexSegmentFlag::UnCompressed => {
                self.writer.stream.write_all(data)?;

                saved_size = data.len() as u64;
                self.saved_size += saved_size;
            },

            IndexSegmentFlag::Compressed => {
                let mut compressed = Vec::<u8>::new();
                let mut encoder = ZlibEncoder::new(&data[..], Compression::fast());

                encoder.read_to_end(&mut compressed)?;

                self.writer.stream.write_all(&mut compressed)?;
                saved_size = compressed.len() as u64;
                self.saved_size += saved_size;
            }
        }
        self.original_size += original_size;

        self.written_segments.push(XP3FileIndexSegment::new(
            flag,
            self.writer.current_pos() - saved_size - self.writer.start_pos,
            original_size,
            saved_size
        ));

        Ok(saved_size)
    }

    /// Write with default flag
    pub fn write_default_segment(&mut self, data: &[u8]) -> Result<u64, io::Error> {
        self.write_segment(self.default_flag, data)
    }

    /// Finish file entry
    pub fn finish(self) {
        let time = if self.timestamp.is_some() {
            Some(XP3FileIndexTime::new(self.timestamp.unwrap()))
        } else {
            None
        };

        let index = XP3FileIndex::new(
            XP3FileIndexInfo::new(
                self.protection,
                self.original_size,
                self.saved_size,
                self.name.clone()
            ),
            self.written_segments.clone(),
            XP3FileIndexAdler::new(self.adler.hash()),
            time
        );

        self.writer.index_set.file_map_mut().insert(index.info().name().clone(), index);
    }

}


impl<'a, T: Write + Seek> Write for EntryWriter<'a, T> {

    /// Write with default flag
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(self.write_segment(self.default_flag, buf)? as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

}

/// Preventing creating more entry before previous entry finish.
impl<'a, T: Write + Seek> Drop for EntryWriter<'a, T> {

    fn drop(&mut self) { }

}