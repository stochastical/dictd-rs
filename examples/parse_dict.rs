use std::{env, fs::File};

use server::{dict::read_definition_from_dict, index::IndexEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("Please pass in a .dict file");
    let mut file = File::open(&path)?;

    read_definition_from_dict(
        &mut file,
        &IndexEntry {
            offset: 5884,
            length: 100,
        },
    )?;

    Ok(())
}
