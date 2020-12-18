/*
 * Created on Sun Dec 13 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{cell::RefCell, collections::hash_map::Iter, io::{self, Cursor, Read, Seek, Write}, marker::PhantomData};

use super::{VirtualXP3, XP3Error, XP3ErrorKind, index::file::XP3FileIndex, reader::XP3Reader};

/// An XP3 archive with XP3 container.
/// Read only occur when user request.
#[derive(Debug)]
pub struct XP3Archive<T: Read + Seek> {

    container: VirtualXP3,

    stream: RefCell<T>

}

impl<T: Read + Seek> XP3Archive<T> {

    pub fn new(container: VirtualXP3, data: T) -> Self {
        Self {
            container,
            stream: RefCell::new(data)
        }
    }

    pub fn container(&self) -> &VirtualXP3 {
        &self.container
    }

    pub fn entries(&self) -> Iter<String, XP3FileIndex> {
        self.container.index_set().entries()
    }

    /// Unpack file to stream
    pub fn unpack(&self, name: &String, stream: &mut impl Write) -> Result<(), XP3Error> {
        let item = self.container.index_set().get(name);

        match item {
            Some(index) => {
                for segment in index.segments().iter() {
                    XP3Reader::read_segment(segment, self.stream.borrow_mut().by_ref(), stream)?;
                }

                Ok(())
            },

            None => Err(XP3Error::new(XP3ErrorKind::FileNotFound, None))
        }
    }

    /// Close xp3 archive
    pub fn close(self) -> (VirtualXP3, T) {
        (self.container, self.stream.into_inner())
    }

}

/// XP3 filter method. (buffer, hash)
pub trait XP3FilterMethod {

    /// Returns read size
    fn filter(buffer: &[u8], hash: u32, out: &mut impl Write) -> io::Result<usize>;

}

/// XP3 archive filter mainy used for encryption
pub struct XP3ArchiveFilter<T, F: XP3FilterMethod> {

    stream: T,
    hash: u32,

    phantom_method: PhantomData<F>

}

impl<T, F: XP3FilterMethod> XP3ArchiveFilter<T, F> {

    pub fn new(stream: T, hash: u32) -> Self {
        Self {
            stream, hash,
            phantom_method: PhantomData
        }
    }

    /// Unwrap stream
    pub fn into_inner(self) -> T {
        self.stream
    }

}

impl<T: Write, F: XP3FilterMethod> Write for XP3ArchiveFilter<T, F> {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        F::filter(buf, self.hash, &mut self.stream)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }

}

impl<T: Read, F: XP3FilterMethod> Read for XP3ArchiveFilter<T, F> {

    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut raw_buf = vec![0_u8; buf.len()];

        self.stream.read(&mut raw_buf)?;
        
        F::filter(&raw_buf, self.hash, &mut Cursor::new(buf))
    }
    
}