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
    pub fn lookup(&mut self, word: &str, strategy: SearchStrategy) -> Vec<StatusResponse> {
        let matches = self.index.lookup(word, strategy);
        let mut definitions = Vec::new();

        for (headword, indices) in matches {
            dbg!(headword);
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

// TEST: mock dictionary lookup
// pub fn define_word(
//     word: &str,
//     database_lookup_strategy: DatabaseLookupStrategy,
// ) -> Vec<Definition> {
//     let test_db_1 = Database {
//         name: "testdb1".to_string(),
//         database_info: "This is a test database".to_string(),
//     };

//     let test_db_2 = Database {
//         name: "testdb2".to_string(),
//         database_info: "This is another test database".to_string(),
//     };

//     match database_lookup_strategy {
//         DatabaseLookupStrategy::Named(db) => {
//             eprintln!("Looking up word '{}' in database '{}'", word, db);
//             vec![Definition {
//                 database: Database {
//                     name: db.clone(),
//                     database_info: format!("This is the {} database", db),
//                 },
//                 head_word: word.to_string(),
//                 definition: format!("Dummy definition of {word}"),
//             }]
//         }
//         DatabaseLookupStrategy::First => {
//             eprintln!("Looking up word '{}' in first available database", word);
//             vec![Definition {
//                 database: test_db_1,
//                 head_word: word.to_string(),
//                 definition: format!("Dummy definition of {word}"),
//             }]
//         }
//         DatabaseLookupStrategy::All => {
//             eprintln!("Looking up word '{}' in all available databases", word);
//             vec![
//                 Definition {
//                     database: test_db_1,
//                     head_word: word.to_string(),
//                     definition: format!("Dummy definition 1 of {word}"),
//                 },
//                 Definition {
//                     database: test_db_2,
//                     head_word: word.to_string(),
//                     definition: format!("Dummy definition 2 of {word}"),
//                 },
//             ]
//         }
//     }
// }
