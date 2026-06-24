/// Implements a dictionary reader for the (uncompressed) .dict format
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

use crate::index::IndexEntry;

#[derive(Debug)]
/// TODO: Should this just be a type alias, or should we store metadata here?
/// And if so, should we use a constructor?
pub struct Dictionary {
    pub dict: File,
}

impl Dictionary {
    pub fn new(path: &Path) -> io::Result<Self> {
        Ok(Dictionary {
            dict: File::open(path)?,
        })
    }

    pub fn read(&mut self, entry: &IndexEntry) -> Result<String, Box<dyn Error>> {
        let mut buf = String::with_capacity(entry.length);

        self.dict.seek(SeekFrom::Start(entry.offset as u64))?;
        self.dict
            .by_ref()
            .take(entry.length as u64)
            .read_to_string(&mut buf)?;

        Ok(buf)
    }
}
