/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod xp3;

mod tests {
    use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom, Write}};

    use crate::xp3::reader::XP3Reader;

    #[test]
    fn xp3_test() {
        let stream = File::open("data.xp3").unwrap();
        let mut copy = File::create("data.copied.xp3").unwrap();
    
        let mut xp3 = XP3Reader::read_archive(BufReader::new(stream)).unwrap();

        xp3.index_set().write_bytes(&mut copy).unwrap();
        
        for index in xp3.index_set().indices() {
            println!("file: {:?}", index);
        }

        //xp3.read_test().unwrap();
    }
}
