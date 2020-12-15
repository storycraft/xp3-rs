/*
 * Created on Sun Dec 13 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{self, Read, Seek, SeekFrom};

use super::{file_index::XP3FileIndexSet, header::XP3Header};

/// An XP3 archive with header, index, data part stream.
/// Only contains header, index info and the read only occur when user request.

#[derive(Debug)]
pub struct XP3Archive<T: Read + Seek> {

    header: XP3Header,
    index_set: XP3FileIndexSet,

    stream: XP3DataStream<T>

}

impl<T: Read + Seek> XP3Archive<T> {

    pub fn new(header: XP3Header, index_set: XP3FileIndexSet, data: T) -> Self {
        Self {
            header,
            index_set,
            stream: XP3DataStream::new(data)
        }
    }

    pub fn header(&self) -> XP3Header {
        self.header
    }

    pub fn set_header(&mut self, header: XP3Header) {
        self.header = header;
    }

    pub fn index_set(&self) -> &XP3FileIndexSet {
        &self.index_set
    }

    pub fn index_set_mut(&mut self) -> &mut XP3FileIndexSet {
        &mut self.index_set
    }

    pub fn stream(&self) -> &XP3DataStream<T> {
        &self.stream
    }

    pub fn stream_mut(&mut self) -> &mut XP3DataStream<T> {
        &mut self.stream
    }
}


#[derive(Debug)]
pub struct XP3DataStream<T: Read + Seek> {

    data: T

}

impl<T: Read + Seek> XP3DataStream<T> {

    pub fn new(data: T) -> Self {
        Self {
            data
        }
    }

    pub fn read_exact_offset(&mut self, offset: u64, buf: &mut [u8]) -> io::Result<()> {
        let pos = self.data.seek(SeekFrom::Current(offset as i64))?;
        self.data.read_exact(buf)?;
        self.data.seek(SeekFrom::Start(pos - offset))?;

        Ok(())
    }

    pub fn into_inner(self) -> T {
        self.data
    }

}