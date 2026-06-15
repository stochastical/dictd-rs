#[derive(Debug)]
pub struct Database {
    pub(crate) name:          String,
    pub(crate) database_info: String,
}

#[derive(Debug)]
pub enum DatabaseLookupStrategy {
    Named(String), // specific database
    First,         // '!'
    All,           // '*
}

/// Unsupported variants include:
/// Substring, Suffix, Regex,
/// Soundex, Levenshtein
#[derive(Debug)]
pub enum SearchStrategy {
    Exact,
    Prefix,
    Default, // '.'
}

#[derive(Debug)]
pub struct Definition {
    pub(crate) database:   Database,
    pub(crate) head_word:  String,
    pub(crate) definition: String,
}
