/*
 * Created on Mon Dec 14 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

pub mod entry;

use std::io::Write;

use self::entry::WriteEntry;

use super::{VirtualXP3, XP3Error, header::XP3HeaderVersion};

pub struct XP3Writer {

    version: XP3HeaderVersion,

    entries: Vec<WriteEntry>

}

impl XP3Writer {

    pub fn new(version: XP3HeaderVersion) -> Self {
        Self {
            version,
            entries: Vec::new()
        }
    }

    pub fn version(&self) -> XP3HeaderVersion {
        self.version
    }

    pub fn set_version(&mut self, version: XP3HeaderVersion) {
        self.version = version;
    }

    pub fn entries(&self) -> &Vec<WriteEntry> {
        &self.entries
    }

    pub fn entries_mut(&mut self) -> &mut Vec<WriteEntry> {
        &mut self.entries
    }

    /// Create archive and build into XP3Archive
    pub fn create<T: Write>(&mut self, ) -> Result<VirtualXP3, XP3Error> {
        
    }

}