/*
 * Created on Wed Dec 16 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{Read, Seek, Write};

use crate::xp3::{XP3Error, index::file::{IndexInfoFlag, IndexSegmentFlag, XP3FileIndexSegment}, reader::XP3Reader};

use byteorder::WriteBytesExt;

pub trait WriteInput {

    fn flag(&self) -> IndexSegmentFlag;

    /// Returns written size
    fn write_data(&mut self, stream: &mut Box<dyn Write>) -> Result<u64, XP3Error>;

}

pub struct WriteEntry {

    protection: IndexInfoFlag,
    name: String,

    timestamp: Option<u64>,

    segments: Vec<Box<dyn WriteInput>>

}

impl WriteEntry {

    pub fn new(
        protection: IndexInfoFlag,
        name: String,
        timestamp: Option<u64>,
        segments: Vec<Box<dyn WriteInput>>
    ) -> Self {
        Self {
            protection,
            name,
            timestamp,
            segments
        }
    }

    pub fn protection(&self) -> IndexInfoFlag {
        self.protection
    }

    pub fn set_protection(&mut self, protection: IndexInfoFlag) {
        self.protection = protection;
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn timestamp(&self) -> Option<u64> {
        self.timestamp
    }

    pub fn set_timestamp(&mut self, timestamp: Option<u64>) {
        self.timestamp = timestamp;
    } 

    pub fn segments(&self) -> &Vec<Box<dyn WriteInput>> {
        &self.segments
    }

}

pub struct StreamInput<'a, T: Read> {

    flag: IndexSegmentFlag,
    stream: &'a mut T

}

impl<'a, T: Read> StreamInput<'a, T> {

    pub fn new(
        flag: IndexSegmentFlag,
        stream: &'a mut T
    ) -> Self {
        Self {
            flag,
            stream
        }
    }

    pub fn flag(&self) -> IndexSegmentFlag {
        self.flag
    }

    pub fn set_flag(&mut self, flag: IndexSegmentFlag) {
        self.flag = flag;
    }

}

impl<'a, T: Read> WriteInput for StreamInput<'a, T> {

    fn flag(&self) -> IndexSegmentFlag {
        self.flag
    }

    fn write_data(&mut self, stream: &mut Box<dyn Write>) -> Result<u64, XP3Error> {
        let mut written = 0_u64;
        for byte in self.stream.bytes() {
            stream.write_u8(byte?)?;

            written += 1;
        }

        Ok(written)
    }
}

/// Reuse data from XP3 stream.
pub struct FileSegmentInput<'a, T: Seek + Read> {

    flag: IndexSegmentFlag,
    stream: &'a mut T,
    segment: &'a XP3FileIndexSegment

}

impl<'a, T: Seek + Read> FileSegmentInput<'a, T> {

    pub fn new(
        flag: IndexSegmentFlag,
        stream: &'a mut T,
        segment: &'a XP3FileIndexSegment
    ) -> Self {
        Self {
            flag, stream, segment
        }
    }

}

impl<'a, T: Seek + Read> WriteInput for FileSegmentInput<'a, T> {

    fn flag(&self) -> IndexSegmentFlag {
        self.flag
    }

    fn write_data(&mut self, stream: &mut Box<dyn Write>) -> Result<u64, XP3Error> {
        XP3Reader::read_segment(self.segment, self.stream, stream)?;

        Ok(self.segment.original_size())
    }

}