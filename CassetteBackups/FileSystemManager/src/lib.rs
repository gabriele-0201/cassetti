use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
};

pub struct FileSystemManager;

impl FileSystemManager {
    // For now NO stuff will be specified
    // hardcoded input and output name
    pub fn read() -> Result<Vec<u8>, ()> {
        let in_file_name = "in_test.org";

        let f = File::open(in_file_name).map_err(|_| ())?;

        Ok(BufReader::new(f)
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| ())?)
    }

    pub fn write(bytes: Vec<u8>) -> Result<(), &'static str> {
        let out_file_name = "out_test.org";
        let f = File::create(out_file_name).map_err(|_| "Impossible Create file")?;

        BufWriter::new(f)
            .write(&bytes)
            .map_err(|_| "Impossible write inside file buffer")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_and_write_test() {
        FileSystemManager::write(FileSystemManager::read().expect("Impossible read"))
            .expect("Impossible write")
    }
}
