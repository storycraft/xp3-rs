/*
 * Created on Wed Dec 16 2020
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{fs::{self, File}, io::{BufReader, BufWriter}, path::Path};

use xp3::reader::XP3Reader;

pub fn main() {
    let stream = BufReader::new(File::open("sample.xp3").unwrap());
            
    let xp3 = XP3Reader::open_archive(stream).unwrap();

    for (name, _) in xp3.entries() {
        println!("Extracting: {}", name);
        
        let path_str = format!("xp3_test/{}", name);
        let path = Path::new(&path_str);
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        xp3.unpack(&name.into(), &mut BufWriter::new(File::create(path).unwrap())).unwrap();
    }
}