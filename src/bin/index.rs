/// Implements a dictionary reader for the (uncompressed) .dict + .index format
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use thiserror::Error;

const NUM_FIELDS: usize = 3;
const FIELD_DELIMITER: char = '\t';
const HEADER_PREFIX: &str = "00-database-";

#[derive(Error, Debug)]
enum ParseError {
    #[error("unknown header {0}")]
    UnknownHeader(String),  // TODO: I'm catching these headers as Uknown instead. Parse don't validate?
    #[error("missing field")]
    MissingField,
    #[error("invalid base64 char {0}")]
    InvalidBase64Char(char),
    #[error("extra field present")]
    ExtraField,
}

#[derive(Debug)]
struct IndexEntry {
    offset: usize,
    length: usize,
}

/// TODO: naming conventions
#[derive(Debug)]
enum DatabaseHeader {
    Info(IndexEntry),
    ShortName(IndexEntry),
    URL(IndexEntry),
    Alphabet(IndexEntry),
    UTF8(IndexEntry),
    DefaultStrategy(IndexEntry),
    AllChars(IndexEntry),
    CaseSensitive(IndexEntry),

    Unknown(String, IndexEntry)
}

type Headword = String;

#[derive(Debug)]
struct Index {
    headers: Vec<DatabaseHeader>,
    entries: Vec<(Headword, IndexEntry)>,
}

/// https://datatracker.ietf.org/doc/html/rfc1421#section-4.3.2.4
/// TODO: pad
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

fn parse_index(reader: BufReader<File>) -> Result<Index, ParseError> {
    // I wish we could use reflection to do Vec::with_capacity(DatabaseHeader.num_variants)
    // Should be able to get the approx entries length from the index_raw, though
    let mut index = Index {
        headers: Vec::new(),
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

        let index_entry = IndexEntry {
            offset: offset as usize,
            length: length as usize,
        };

        if key.starts_with(HEADER_PREFIX) {
            let header_type =
                key.strip_prefix(HEADER_PREFIX).unwrap();

            index.headers.push(match header_type {
                "alphabet" => DatabaseHeader::Alphabet(index_entry),
                "info" => DatabaseHeader::Info(index_entry),
                "short" => DatabaseHeader::ShortName(index_entry),
                "url" => DatabaseHeader::URL(index_entry),
                "utf8" => DatabaseHeader::UTF8(index_entry),
                "defaultstrategy" => DatabaseHeader::DefaultStrategy(index_entry),
                "allchars" => DatabaseHeader::AllChars(index_entry),
                "casesensitive" => DatabaseHeader::CaseSensitive(index_entry),
                _ => DatabaseHeader::Unknown(key.into(), index_entry)
            });
        } else {
            index.entries.push((key.into(), index_entry));
        }
    }
    Ok(index)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("Please pass in a .index file");
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let index = parse_index(reader)?;
    dbg!(&index);
    
    Ok(())
}
