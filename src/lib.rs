/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod xp3;

#[cfg(test)]
mod tests {
    use std::{fs::{self, File}, io::BufReader, path::Path};

    use crate::xp3::reader::XP3Reader;

    #[test]
    fn xp3_test() {
        let stream = BufReader::new(File::open("data_r.xp3").unwrap());
    
        let xp3 = XP3Reader::read_archive(stream).unwrap();

        for (name, index) in xp3.entries() {
            let save_name = if name.len() < 50 {
                name
            } else {
                "Long name.txt"
            };

            println!("Extracting: {}", save_name);
                let path_str = format!("xp3_test/{}", save_name);
                let path = Path::new(&path_str);
                
                fs::create_dir_all(path.parent().unwrap()).unwrap();

                xp3.unpack(&name.into(), &mut File::create(path).unwrap()).unwrap();
        }

    }
}
