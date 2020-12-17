/*
 * Created on Wed Dec 16 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{fs::File, io::{BufWriter, Write}};

use xp3::{header::XP3HeaderVersion, index::file::IndexInfoFlag, index_set::XP3IndexCompression, writer::XP3Writer};

pub fn main() {
    let sample_xp3 = BufWriter::new(File::create("sample.xp3").unwrap());

        let mut writer = XP3Writer::start(
            sample_xp3,
            XP3HeaderVersion::Current {
                minor_version: 1,
                index_size_offset: 0
            },
            XP3IndexCompression::Compressed
        ).unwrap();

        let mut entry = writer.enter_file(
            IndexInfoFlag::Protected,
            "sample.txt".into(),
            Some(0)
        );

        entry.write_all("Hello world!".as_bytes()).unwrap();
        entry.finish();

        let (build, _) = writer.finish().unwrap();

        println!("Written: {:?}", build);
}