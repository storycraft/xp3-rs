/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

pub mod xp3;

#[cfg(test)]
mod tests {
    use std::{fs::{self, File}, io::{BufReader, BufWriter}, path::Path};

    use crate::xp3::{header::XP3HeaderVersion, index::file::{IndexInfoFlag, IndexSegmentFlag, XP3FileIndexTime}, index_set::XP3IndexCompression, reader::XP3Reader, writer::{XP3Writer, entry::{StreamInput, WriteEntry, WriteInput}}};

    #[test]
    fn xp3_test() {
        let stream = BufReader::new(File::open("data_r.xp3").unwrap());
            
        let xp3 = XP3Reader::open_archive(stream).unwrap();

        let sample = BufReader::new(File::open("Cargo.toml").unwrap());

        let mut writer = XP3Writer::new(
            XP3HeaderVersion::Current {
                minor_version: 1,
                index_size_offset: 0
            },
            XP3IndexCompression::Compressed
        );

        let mut segments: Vec<Box<dyn WriteInput>> = Vec::new();

        segments.push(
            Box::new(
                StreamInput::new(IndexSegmentFlag::Compressed, sample)
            )
        );

        writer.entries_mut().push(
            WriteEntry::new(
                IndexInfoFlag::Protected, 
                "Cargo.toml".into(),
                Some(0),
                segments
            )
        );

        let build = writer.build(&mut BufWriter::new(File::create("sample.xp3").unwrap())).unwrap();

        println!("Written: {:?}", build);

        /*for (name, _) in xp3.entries() {
            let save_name = if name.len() < 50 {
                name
            } else {
                "Long name.txt"
            };

            println!("Extracting: {}", save_name);
            let path_str = format!("xp3_test/{}", save_name);
            let path = Path::new(&path_str);
            
            fs::create_dir_all(path.parent().unwrap()).unwrap();

            xp3.unpack(&name.into(), &mut BufWriter::new(File::create(path).unwrap())).unwrap();
        }*/

    }
}
