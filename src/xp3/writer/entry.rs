/*
 * Created on Wed Dec 16 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{Read, Seek};

use crate::xp3::{XP3Error, index::file::{IndexInfoFlag, IndexSegmentFlag, XP3FileIndexSegment}, reader::XP3Reader};

pub trait WriteInput {

    fn flag(&self) -> IndexSegmentFlag;

    fn write_data(&mut self) -> Result<Vec<u8>, XP3Error>;

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

    pub fn segments_mut(&mut self) -> &mut Vec<Box<dyn WriteInput>> {
        &mut self.segments
    }
}

pub struct StreamInput<T: Read> {

    flag: IndexSegmentFlag,
    stream: T

}

impl<T: Read> StreamInput<T> {

    pub fn new(
        flag: IndexSegmentFlag,
        stream: T
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

impl<T: Read> WriteInput for StreamInput<T> {

    fn flag(&self) -> IndexSegmentFlag {
        self.flag
    }

    fn write_data(&mut self) -> Result<Vec<u8>, XP3Error> {
        let mut data = Vec::new();
        
        self.stream.read_to_end(&mut data)?;

        Ok(data)
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

    fn write_data(&mut self) -> Result<Vec<u8>, XP3Error> {
        let mut data = Vec::with_capacity(self.segment.original_size() as usize);

        XP3Reader::read_segment(self.segment, self.stream, &mut data)?;

        Ok(data)
    }

}