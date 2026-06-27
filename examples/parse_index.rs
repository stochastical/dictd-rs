use std::{env, fs::File, io::BufReader};

use dictd::index::Index;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("Please pass in a .index file");
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let index = Index::parse(reader)?;
    dbg!(&index.headers);
    dbg!(&index.entries.len());
    dbg!(&index.entries.iter().take(10).collect::<Vec<_>>());

    Ok(())
}
