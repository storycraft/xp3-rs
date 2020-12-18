/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

//! A XP3(krkr) archive library for rust.
//! ## Examples
//! See `examples` directory for various code examples.

pub mod archive;
pub mod reader;
pub mod writer;

pub mod header;
pub mod index;
pub mod index_set;

pub use reader::XP3Reader;
pub use writer::XP3Writer;

use std::{error::Error, io};

use self::{header::XP3Header, index_set::XP3IndexSet};

pub const XP3_MAGIC: [u8; 10] = [ 0x58_u8, 0x50, 0x33, 0x0D, 0x0A, 0x20, 0x0A, 0x1A, 0x8B, 0x67 ];

pub const XP3_CURRENT_VER_IDENTIFIER: u64 = 0x17;

pub const XP3_VERSION_IDENTIFIER: u8 = 128;

pub const XP3_INDEX_CONTINUE: u8 = 0x80;

pub const XP3_INDEX_FILE_IDENTIFIER: u32 = 1701603654; // File

pub const XP3_INDEX_INFO_IDENTIFIER: u32 = 1868983913; // info
pub const XP3_INDEX_SEGM_IDENTIFIER: u32 = 1835492723; // segm
pub const XP3_INDEX_ADLR_IDENTIFIER: u32 = 1919706209; // adlr
pub const XP3_INDEX_TIME_IDENTIFIER: u32 = 1701669236; // time

#[derive(Debug)]
pub struct XP3Error {

    kind: XP3ErrorKind,
    error: Option<Box<dyn Error>>

}

impl XP3Error {

    pub fn new(kind: XP3ErrorKind, error: Option<Box<dyn Error>>) -> Self {
        Self {
            kind, error
        }
    }

    pub fn kind(&self) -> &XP3ErrorKind {
        &self.kind
    }

    pub fn error(&self) -> &Option<Box<dyn Error>> {
        &self.error
    }

}

impl From<io::Error> for XP3Error {

    fn from(err: io::Error) -> Self {
        XP3Error::new(XP3ErrorKind::Io(err), None)
    }

}

#[derive(Debug)]
pub enum XP3ErrorKind {

    Io(io::Error),
    InvalidFile,
    InvalidHeader,
    InvalidFileIndexHeader,
    InvalidFileIndex,
    InvalidFileIndexFlag,

    FileNotFound

}

/// Virtual XP3 container containing XP3 file information.
#[derive(Debug)]
pub struct VirtualXP3 {

    header: XP3Header,
    index_set: XP3IndexSet

}

impl VirtualXP3 {

    pub fn new(
        header: XP3Header,
        index_set: XP3IndexSet
    ) -> Self {
        Self {
            header,
            index_set
        }
    }

    pub fn header(&self) -> XP3Header {
        self.header
    }

    pub fn set_header(&mut self, header: XP3Header) {
        self.header = header;
    }

    pub fn index_set(&self) -> &XP3IndexSet {
        &self.index_set
    }

    pub fn set_index_set(&mut self, index_set: XP3IndexSet) {
        self.index_set = index_set;
    }

}
