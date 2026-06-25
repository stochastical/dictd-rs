use std::{fs::File, io::BufReader, path::Path};

use crate::{
    dict::Dictionary,
    index::{Index, ParseError},
    protocol::{
        DatabaseLookupStrategy, SearchStrategy,
        StatusResponse::{self, WordDefinition},
    },
};

#[derive(Debug)]
pub struct Database {
    pub name: String,
    pub description: String,
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
            name: "gcide".into(),
            description: "test_db_gcide".into(),
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
                    db_info: self.description.clone(),
                    definition: self.dict.read(index).unwrap(),
                });
            }
        }
        definitions
    }
}

#[derive(Debug)]
pub struct DatabaseEngine {
    /// A DatabaseEngine owns a Database which owns a Dictionary & Index
    pub dbs: Vec<Database>,
}

/// TODO: Technically the Engine should be agnostic to the strategy
/// and different DB _implementations_ could support different stategies
impl DatabaseEngine {
    pub fn lookup(
        &mut self,
        word: &str,
        lookup_strat: DatabaseLookupStrategy,
        search_strat: SearchStrategy,
    ) -> Vec<StatusResponse> {
        // TODO: We need to if self.dbs.len == 0 { return Err(NoDatabasesPresent); }
        match lookup_strat {
            DatabaseLookupStrategy::Named(name) => {
                eprintln!(
                    "Looking up word '{word}' in database '{name}' using match strategy '{:?}'",
                    search_strat
                );
                // TODO: we need to validate (parse?) db name else return InvalidDatabase

                self.dbs
                    .iter_mut()
                    .filter(|db| db.name == name)
                    .flat_map(|db| db.lookup(&word, search_strat))
                    .collect()
            }
            DatabaseLookupStrategy::First => {
                eprintln!(
                    "Looking up word '{word}' in first available database using match strategy '{:?}'",
                    search_strat
                );
                self.dbs[0].lookup(&word, search_strat)
            }
            DatabaseLookupStrategy::All => {
                eprintln!(
                    "Looking up word '{word}' in all available databases using match strategy '{:?}'",
                    search_strat
                );
                self.dbs
                    .iter_mut()
                    .flat_map(|db| db.lookup(&word, search_strat))
                    .collect()
            }
        }
    }
}
