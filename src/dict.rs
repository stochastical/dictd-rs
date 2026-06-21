/// Implements a dictionary reader for the (uncompressed) .dict format
use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

fn read_definition_from_dict(
    dict: &mut File,
    index: IndexEntry,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut buf = String::with_capacity(index.length);
    dbg!(dict.seek(SeekFrom::Start(index.offset as u64))?);
    dbg!(dict.take(index.length as u64).read_to_string(&mut buf)?);
    Ok(dbg!(buf))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("Please pass in a .index file");
    let mut file = File::open(&path)?;

    read_definition_from_dict(
        &mut file,
        IndexEntry {
            offset: 5884,
            length: 100,
        },
    )?;

    Ok(())
}
