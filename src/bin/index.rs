/// Implements a dictionary reader for the (uncompressed) .dict + .index format
use std::env;
use thiserror::Error;

const NUM_FIELDS: usize = 3;
const FIELD_DELIMITER: char = '\t';
const HEADER_PREFIX: &str = "00";

#[derive(Error, Debug)]
enum ParseError {
    #[error("unknown header")]
    UnknownHeader(String),
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

fn normalise_header_key(key: &str) -> Option<&str> {
    key.strip_prefix("00database")
        .or_else(|| key.strip_prefix("00-database-"))
}

fn parse_index(content: &str) -> Result<Index, ParseError> {
    // I wish we could use reflection to do Vec::with_capacity(DatabaseHeader.num_variants)
    // Should be able to get the approx entries length from the index_raw, though
    let mut index = Index {
        headers: Vec::new(),
        entries: Vec::new(),
    };

    for line in content.lines().filter(|l| !l.is_empty()) {
        let mut parts = line.splitn(NUM_FIELDS, FIELD_DELIMITER);

        let (Some(key), Some(offset), Some(length)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(ParseError::MissingField);
        };
        if parts.next().is_some() {
            return Err(ParseError::ExtraField);
        }

        let offset = decode_base64_int(offset)?;
        let length = decode_base64_int(length)?;

        let index_entry = IndexEntry {
            offset: offset as usize,
            length: length as usize,
        };

        if key.starts_with(HEADER_PREFIX) {
            let header_type =
                normalise_header_key(key).ok_or_else(|| ParseError::UnknownHeader(key.into()))?;

            index.headers.push(match header_type {
                "alphabet" => DatabaseHeader::Alphabet(index_entry),
                "info" => DatabaseHeader::Info(index_entry),
                "short" => DatabaseHeader::ShortName(index_entry),
                "url" => DatabaseHeader::URL(index_entry),
                "utf8" => DatabaseHeader::UTF8(index_entry),
                "defaultstrategy" => DatabaseHeader::DefaultStrategy(index_entry),
                "allchars" => DatabaseHeader::AllChars(index_entry),
                "casesensitive" => DatabaseHeader::CaseSensitive(index_entry),
                _ => return Err(ParseError::UnknownHeader(header_type.into())),
            });
        } else {
            index.entries.push((key.into(), index_entry));
        }
    }
    Ok(index)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let index_path = env::args().nth(1).expect("Please pass in a .index file");
    let content = std::fs::read_to_string(index_path)?;
    let index = parse_index(&content)?;
    println!("{:#?}", index);

    Ok(())
}
