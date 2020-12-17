/*
 * Created on Sun Dec 13 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{cell::RefCell, collections::hash_map::Iter, io::{Read, Seek, Write}};

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
