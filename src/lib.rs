/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

pub mod xp3;

#[cfg(test)]
mod tests {
    use std::{fs::{self, File}, io::{BufReader, BufWriter}, path::Path};

    use crate::xp3::{header::XP3HeaderVersion, index::file::{IndexInfoFlag, IndexSegmentFlag, XP3FileIndexTime}, index_set::XP3IndexCompression, reader::XP3Reader, writer::XP3Writer};

    #[test]
    fn xp3_test() {
        let stream = BufReader::new(File::open("data_r.xp3").unwrap());
            
        let xp3 = XP3Reader::open_archive(stream).unwrap();

        let sample_xp3 = BufWriter::new(File::create("sample.xp3").unwrap());
        let mut sample = BufReader::new(File::open("Cargo.toml").unwrap());

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
            "Cargo.toml".into(),
            Some(0)
        );

        entry.write_segment(IndexSegmentFlag::UnCompressed, &mut sample).unwrap();
        entry.finish();

        let (build, _) = writer.finish().unwrap();

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
