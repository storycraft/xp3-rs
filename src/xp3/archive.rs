/*
 * Created on Sun Dec 13 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{cell::RefCell, collections::hash_map::Iter, io::{self, Read, Seek, SeekFrom, Write}};

use flate2::read::ZlibDecoder;

use super::{XP3Error, XP3ErrorKind, header::XP3Header, index::file::{XP3FileIndex, IndexSegmentFlag}, index_set::XP3FileIndexSet};

/// An XP3 archive with header, index, data part stream.
/// Only contains header, index info and the read only occur when user request.

#[derive(Debug)]
pub struct XP3Archive<T: Read + Seek> {

    header: XP3Header,
    index_set: XP3FileIndexSet,

    stream: RefCell<XP3DataStream<T>>

}

impl<T: Read + Seek> XP3Archive<T> {

    pub fn new(header: XP3Header, index_set: XP3FileIndexSet, data: T) -> Self {
        Self {
            header,
            index_set,
            stream: RefCell::new(XP3DataStream::new(data))
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

    pub fn entries(&self) -> Iter<String, XP3FileIndex> {
        self.index_set.entries()
    }

    /// Unpack file to stream
    pub fn unpack<W: Write>(&self, name: &String, stream: &mut W) -> Result<(), XP3Error> {
        let item = self.index_set.get(name);

        match item {
            Some(index) => {
                for segment in index.segments().iter() {
                    let mut buffer = Vec::with_capacity(segment.saved_size() as usize);
                    self.stream.borrow_mut().read_sized(segment.data_offset(), segment.saved_size(), &mut buffer)?;

                    match segment.flag() {
                        IndexSegmentFlag::UnCompressed => stream.write_all(&mut buffer[..segment.saved_size() as usize])?,

                        IndexSegmentFlag::Compressed => {
                            let mut uncompressed = Vec::<u8>::with_capacity(segment.original_size() as usize);

                            let decoder = ZlibDecoder::new(&buffer[..]);

                            decoder.take(segment.original_size()).read_to_end(&mut uncompressed)?;

                            stream.write_all(&mut uncompressed)?;
                        }
                    }
                }

                Ok(())
            },

            None => Err(XP3Error::new(XP3ErrorKind::FileNotFound, None))
        }
    }

    /// Close target xp3 archive
    pub fn unwrap(self) -> T {
        self.stream.into_inner().into_inner()
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

    pub fn read_exact_pos_offset(&mut self, offset: u64, buf: &mut [u8]) -> io::Result<()> {
        let pos = self.data.seek(SeekFrom::Current(offset as i64))?;
        self.data.read_exact(buf)?;
        self.data.seek(SeekFrom::Start(pos - offset))?;

        Ok(())
    }

    pub fn read_sized(&mut self, offset: u64, size: u64, buf: &mut Vec<u8>) -> io::Result<()> {
        let pos = self.data.seek(SeekFrom::Current(offset as i64))?;
        self.data.by_ref().take(size).read_to_end(buf)?;
        self.data.seek(SeekFrom::Start(pos - offset))?;

        Ok(())
    }

    pub fn into_inner(self) -> T {
        self.data
    }

}
