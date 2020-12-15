/*
 * Created on Tue Dec 15 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod xp3;

#[cfg(test)]
mod tests {
    use std::{fs::File};

    use crate::xp3::reader::XP3Reader;

    #[test]
    fn xp3_test() {
        let stream = File::open("data.xp3").unwrap();
    
        let xp3 = XP3Reader::read_archive(stream.try_clone().unwrap()).unwrap();
        for index in xp3.index_set().indices() {
            println!("file: {:?}", index.info().name());
        }

    }
}
