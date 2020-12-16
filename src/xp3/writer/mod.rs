/*
 * Created on Mon Dec 14 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

pub mod entry;

use std::{collections::HashMap, io::{Read, Seek, SeekFrom, Write}};

use adler32::RollingAdler32;
use byteorder::{WriteBytesExt, LittleEndian};
use flate2::{Compression, read::ZlibEncoder};

use self::entry::WriteEntry;

use super::{VirtualXP3, XP3Error, XP3_MAGIC, header::{XP3Header, XP3HeaderVersion}, index::{XP3Index, file::{IndexSegmentFlag, XP3FileIndex, XP3FileIndexAdler, XP3FileIndexInfo, XP3FileIndexSegment, XP3FileIndexTime}}, index_set::{XP3IndexCompression, XP3IndexSet}};

pub struct XP3Writer {

    version: XP3HeaderVersion,

    index_compression: XP3IndexCompression,

    extra_indexes: Vec<XP3Index>,
    entries: Vec<WriteEntry>

}

impl XP3Writer {

    pub fn new(
        version: XP3HeaderVersion,
        index_compression: XP3IndexCompression
    ) -> Self {
        Self {
            version,
            index_compression,
            extra_indexes: Vec::new(),
            entries: Vec::new()
        }
    }

    pub fn version(&self) -> XP3HeaderVersion {
        self.version
    }

    pub fn set_version(&mut self, version: XP3HeaderVersion) {
        self.version = version;
    }

    pub fn index_compression(&self) -> XP3IndexCompression {
        self.index_compression
    }

    pub fn set_index_compression(&mut self, index_compression: XP3IndexCompression) {
        self.index_compression = index_compression;
    }

    pub fn entries(&self) -> &Vec<WriteEntry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<WriteEntry> {
        &mut self.entries
    }

    pub fn extra_indexes(&self) -> &Vec<XP3Index> {
        &self.extra_indexes
    }

    pub fn extra_indexes_mut(&mut self) -> &mut Vec<XP3Index> {
        &mut self.extra_indexes
    }

    /// Create archive and build into XP3Archive
    pub fn build<T: Seek + Write>(&mut self, stream: &mut T) -> Result<VirtualXP3, XP3Error> {
        let header = XP3Header::new(self.version);

        let mut index_map: HashMap<String, XP3FileIndex> = HashMap::with_capacity(self.entries.len());

        let start_position = stream.seek(SeekFrom::Current(0))?;

        // Write magic
        stream.write_all(&XP3_MAGIC)?;
        // Fixed version
        stream.write_u8(1)?;
        // Write header
        header.write_bytes(stream)?;
        // Write index offset
        // Save position
        let index_offset_pos = stream.seek(SeekFrom::Current(0))?;
        // Rewrite after data stream written.
        stream.write_u64::<LittleEndian>(0)?;

        // Write segments stream and build index_set
        for entry in &mut self.entries {
            let mut saved_file_size = 0_u64;
            let mut file_size = 0_u64;
            let mut segments = Vec::<XP3FileIndexSegment>::new();
            let mut adler = RollingAdler32::new();

            for entry_segment in entry.segments_mut() {
                let mut data = entry_segment.write_data()?;
                let current = stream.seek(SeekFrom::Current(0))?;

                let original_size = data.len() as u64;
                let written_size: u64;
                match entry_segment.flag() {
                    IndexSegmentFlag::UnCompressed => {
                        stream.write_all(&mut data)?;

                        written_size = data.len() as u64;
                    },

                    IndexSegmentFlag::Compressed => {
                        let mut compressed = Vec::<u8>::new();
                        let mut encoder = ZlibEncoder::new(&data[..], Compression::fast());

                        encoder.read_to_end(&mut compressed)?;

                        stream.write_all(&mut compressed)?;
                        written_size = compressed.len() as u64;
                    }
                }

                adler.update_buffer(&data);

                let segment = XP3FileIndexSegment::new(
                    entry_segment.flag(),
                    current,
                    written_size,
                    original_size
                );

                segments.push(segment);

                saved_file_size += written_size;
                file_size += original_size;
            }

            let time = if entry.timestamp().is_some() {
                Some(XP3FileIndexTime::new(entry.timestamp().unwrap()))
            } else {
                None
            };

            let index = XP3FileIndex::new(
                XP3FileIndexInfo::new(
                    entry.protection(),
                    file_size,
                    saved_file_size,
                    entry.name().clone()
                ),
                segments,
                XP3FileIndexAdler::new(adler.hash()),
                time
            );

            index_map.insert(index.info().name().clone(), index);
        }

        let mut index_set = XP3IndexSet::new(
            self.index_compression,
            self.extra_indexes.clone(),
            index_map
        );

        {
            let current = stream.seek(SeekFrom::Current(0))?;

            stream.seek(SeekFrom::Start(index_offset_pos))?;
            stream.write_u64::<LittleEndian>(current - start_position)?;

            stream.seek(SeekFrom::Start(current))?;
        }
        
        // Write indexes
        index_set.write_bytes(stream)?;

        Ok(VirtualXP3::new(
            header,
            index_set
        ))
    }

}