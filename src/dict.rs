/// Implements a dictionary reader for the (uncompressed) .dict format
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use crate::index::IndexEntry;

pub fn read_definition_from_dict(
    dict: &mut File,
    index: &IndexEntry,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::with_capacity(index.length);
    dbg!(dict.seek(SeekFrom::Start(index.offset as u64))?);
    dbg!(dict.take(index.length as u64).read_to_string(&mut buf)?);
    Ok(dbg!(buf))
}
