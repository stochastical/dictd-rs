use std::{fs::File, io::BufReader, path::Path};

use crate::{
    dict::Dictionary,
    index::{Index, ParseError},
    protocol::{
        SearchStrategy,
        StatusResponse::{self, WordDefinition},
    },
};

#[derive(Debug)]
pub struct Database {
    pub name: String,
    pub info: String,
    pub dict: Dictionary,
    pub index: Index,
}

// TODO: Fix unwraps
impl Database {
    pub fn new(index_path: &Path, dict_path: &Path) -> Result<Self, ParseError> {
        let index = Index::parse(BufReader::new(File::open(index_path).unwrap()))?;
        let dict = Dictionary::new(dict_path).unwrap();
        // TODO: resolve headers into metadata?
        // TODO: maybe it would be header for headers to be a simple key-value map?
        // that way I can get values out of it easier than matching on an enum...
        Ok(Database {
            name: "test".into(),
            info: "test_db".into(),
            dict,
            index,
        })
    }

    /// Resolves matches from the index to the corresponding dict entries for definitions
    /// TODO: strange we only return 1 variant?
    pub fn lookup(&mut self, word: &str, strategy: SearchStrategy) -> Vec<StatusResponse> {
        let matches = self.index.lookup(word, strategy);
        let mut definitions = Vec::new();

        for (headword, indices) in matches {
            for index in indices {
                definitions.push(WordDefinition {
                    headword: headword.into(),
                    db_name: self.name.clone(),
                    db_info: self.info.clone(),
                    definition: self.dict.read(index).unwrap(),
                });
            }
        }
        definitions
    }
}
