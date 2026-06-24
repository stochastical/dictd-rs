use crate::protocol::DatabaseLookupStrategy;

#[derive(Debug)]
pub struct Database {
    pub name:          String,
    pub database_info: String,
}

#[derive(Debug)]
pub struct Definition {
    pub database:   Database,
    pub head_word:  String,
    pub definition: String,
}

/// TEST: mock dictionary lookup
pub fn define_word(
    word: &str,
    database_lookup_strategy: DatabaseLookupStrategy,
) -> Vec<Definition> {
    let test_db_1 = Database {
        name: "testdb1".to_string(),
        database_info: "This is a test database".to_string(),
    };

    let test_db_2 = Database {
        name: "testdb2".to_string(),
        database_info: "This is another test database".to_string(),
    };

    match database_lookup_strategy {
        DatabaseLookupStrategy::Named(db) => {
            eprintln!("Looking up word '{}' in database '{}'", word, db);
            vec![Definition {
                database: Database {
                    name: db.clone(),
                    database_info: format!("This is the {} database", db),
                },
                head_word: word.to_string(),
                definition: format!("Dummy definition of {word}"),
            }]
        }
        DatabaseLookupStrategy::First => {
            eprintln!("Looking up word '{}' in first available database", word);
            vec![Definition {
                database: test_db_1,
                head_word: word.to_string(),
                definition: format!("Dummy definition of {word}"),
            }]
        }
        DatabaseLookupStrategy::All => {
            eprintln!("Looking up word '{}' in all available databases", word);
            vec![
                Definition {
                    database: test_db_1,
                    head_word: word.to_string(),
                    definition: format!("Dummy definition 1 of {word}"),
                },
                Definition {
                    database: test_db_2,
                    head_word: word.to_string(),
                    definition: format!("Dummy definition 2 of {word}"),
                },
            ]
        }
    }
}
