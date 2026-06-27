use std::{env, fs::File};

use dictd::{dict::Dictionary, index::IndexEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("Please pass in a .dict file");
    let file = File::open(&path)?;

    let mut dict = Dictionary { dict: file };

    dict.read(&IndexEntry {
        offset: 5884,
        length: 100,
    })?;

    Ok(())
}
