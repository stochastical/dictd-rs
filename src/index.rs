/// Implements a dictionary reader for the .index format
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use thiserror::Error;

const NUM_FIELDS: usize = 3 + 1;
const FIELD_DELIMITER: char = '\t';
const HEADER_PREFIX: &str = "00-";

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("unknown header {0}")]
    UnknownHeader(String), // TODO: I'm catching these headers as Uknown instead. Parse don't validate?
    #[error("missing field")]
    MissingField,
    #[error("invalid base64 char {0}")]
    InvalidBase64Char(char),
    #[error("extra field present")]
    ExtraField,
}

#[derive(Debug)]
pub struct IndexEntry {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug)]
pub enum HeaderKind {
    Info,
    ShortName,
    LongName,
    Url,
    Alphabet,
    Utf8,
    DefaultStrategy,
    AllChars,
    CaseSensitive,
    Unknown(String),
}

#[derive(Debug)]
pub struct DatabaseHeader {
    kind: HeaderKind,
    entry: IndexEntry,
}

pub type Headword = String;

#[derive(Debug)]
pub struct Index {
    headers: Vec<DatabaseHeader>,
    entries: Vec<(Headword, IndexEntry)>,
}

/// https://datatracker.ietf.org/doc/html/rfc1421#section-4.3.2.4
const fn decode_base64_digit(c: char) -> Result<u8, ParseError> {
    match c {
        'A'..='Z' => Ok((c as u8) - b'A'),
        'a'..='z' => Ok((c as u8) - b'a' + 26),
        '0'..='9' => Ok((c as u8) - b'0' + 52),
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(ParseError::InvalidBase64Char(c)),
    }
}

/// Decode a base64 positional integer
fn decode_base64_int(s: &str) -> Result<u64, ParseError> {
    let mut result = 0;
    for c in s.chars() {
        let digit = decode_base64_digit(c)?;
        result = result * 64 + (digit as u64);
    }
    Ok(result)
}

pub fn parse_index(reader: BufReader<File>) -> Result<Index, ParseError> {
    let mut index = Index {
        /// I wish we could use reflection to do Vec::with_capacity(DatabaseHeader.num_variants)
        /// https://doc.rust-lang.org/std/mem/fn.variant_count.html
        headers: Vec::new(),
        /// Should be able to get the approx entries length from the reader length, though?
        entries: Vec::new(),
    };

    for line in reader.lines().map(|l| l.unwrap()).filter(|l| !l.is_empty()) {
        let mut parts = line.splitn(NUM_FIELDS, FIELD_DELIMITER);

        let (Some(key), Some(offset), Some(length)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(ParseError::MissingField);
        };
        if key.is_empty() || offset.is_empty() || length.is_empty() {
            return Err(ParseError::MissingField);
        }
        if parts.next().is_some() {
            return Err(ParseError::ExtraField);
        }

        let offset = decode_base64_int(offset)?;
        let length = decode_base64_int(length)?;
        assert_ne!(length, 0);

        let entry = IndexEntry {
            offset: offset as usize,
            length: length as usize,
        };

        if let Some(header_type) = key.strip_prefix(HEADER_PREFIX) {
            let kind = match header_type {
                "database-alphabet" => HeaderKind::Alphabet,
                "database-info" => HeaderKind::Info,
                "database-short" => HeaderKind::ShortName,
                "database-long" => HeaderKind::LongName,
                "database-url" => HeaderKind::Url,
                "database-utf8" => HeaderKind::Utf8,
                "database-defaultstrategy" => HeaderKind::DefaultStrategy,
                "database-allchars" => HeaderKind::AllChars,
                "database-casesensitive" => HeaderKind::CaseSensitive,
                _ => HeaderKind::Unknown(key.into()),
            };
            index.headers.push(DatabaseHeader { kind, entry });
        } else {
            index.entries.push((key.into(), entry));
        }
    }
    Ok(index)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("Please pass in a .index file");
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let index = parse_index(reader)?;
    dbg!(&index.headers);
    dbg!(&index.entries.iter().take(10).collect::<Vec<_>>());

    Ok(())
}
