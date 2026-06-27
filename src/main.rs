use std::env;
use std::io::{BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use dictd::database::{Database, DatabaseEngine};
use dictd::parser::{Command, ShowArgument};
use dictd::protocol::{HELP_LINES, SearchStrategy, StatusResponse};
use uuid::Uuid;

// TODO: define server struct (with assoc. constants + list of clients + )
// TODO: define client struct (with UUID + stream)

const DICT_SERVER_PORT: u16 = 2628;
const LINE_BUFFER_MAX_CHARS: usize = 1024;
const LINE_BUFFER_MAX_BYTES: usize = 6144; // 1024 * 6
const MIME_HEADER: &[&str] = &[
    "Content-type: text/plain; charset=utf-8",
    "Content-transfer-encoding: 8bit",
];
const SERVER_INFO: &str = "A server project by abstractnonsense.xyz <hello@abstractnonsense.xyz>";

/// TODO: it'd be nice to return a StatusResponse maybe?
/// TODO: Should there be a timeout on the connection?
/// TODO: What's the best way to do dependency injection (i.e. we need to know about all databases etc...)
fn handle_connection(mut stream: TcpStream, dbs: &mut DatabaseEngine) -> std::io::Result<()> {
    eprintln!("New client connection: {:#?}", &stream);
    let mut reader = BufReader::new(stream.try_clone()?);

    // Send server banner
    write!(
        stream,
        "{}\r\n",
        StatusResponse::ClientIPAllowed {
            text: "dictd-rs".into(),
            msg_id: Uuid::new_v4(),
        }
    )?;

    let mut line = String::with_capacity(LINE_BUFFER_MAX_CHARS);

    loop {
        line.clear();
        let bytes_read: usize = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            eprintln!("EOF or client closed connection");
            break;
        }

        let command_line = line.trim_end_matches(['\r', '\n']);
        if command_line.chars().count() > LINE_BUFFER_MAX_CHARS {
            write!(stream, "{}", StatusResponse::SyntaxErrorIllegalParameters)?;
            continue;
        }
        // TODO: blank lines should just continue, I don't think I need a 500 unknown command returned

        match dbg!(Command::try_from(command_line)) {
            Ok(Command::Quit) => {
                write!(stream, "{}", StatusResponse::Quit)?;
                return Ok(());
            }
            Ok(Command::Help) => {
                write!(stream, "{}", StatusResponse::Help)?;
                write!(stream, "{}", HELP_LINES.join("\r\n"))?;
                write!(stream, "\r\n.\r\n")?;
                write!(stream, "{}", StatusResponse::Ok)?;
            }
            Ok(Command::Client { .. }) => {
                write!(stream, "{}", StatusResponse::Ok)?;
            }
            Ok(Command::Define { lookup, word }) => {
                let definitions = dbs.lookup(&word, lookup, SearchStrategy::default());

                if definitions.is_empty() {
                    write!(stream, "{}", StatusResponse::NoMatch)?;
                } else {
                    write!(
                        stream,
                        "{}",
                        StatusResponse::WordFound {
                            n_definitions: definitions.len(),
                        }
                    )?;
                    for definition in definitions {
                        write!(stream, "{definition}",)?;
                        write!(stream, ".\r\n")?;
                    }
                    write!(stream, "{}", StatusResponse::Ok)?;
                }
            }
            Ok(Command::Match {
                lookup_strat,
                search_strat,
                word,
            }) => {
                let matches = dbs.find_matches(&word, lookup_strat, search_strat);
                if matches.is_empty() {
                    write!(stream, "{}", StatusResponse::NoMatch)?;
                } else {
                    write!(
                        stream,
                        "{}",
                        StatusResponse::WordsMatched {
                            n_matches: matches.len()
                        }
                    )?;
                    for (db_name, headword) in &matches {
                        write!(stream, "{db_name} {headword}\r\n")?;
                    }
                    write!(stream, ".\r\n")?;
                    write!(stream, "{}", StatusResponse::Ok)?;
                }
            }
            Ok(Command::Show(arg)) => {
                match arg {
                    ShowArgument::Info { database } => {
                        if let Some(db) = dbs.dbs.iter().find(|db| db.name == database) {
                            write!(stream, "{}", StatusResponse::DatabaseInformation)?;
                            write!(stream, "{}", db.description)?;
                            write!(stream, ".\r\n")?;
                        } else {
                            write!(stream, "{}", StatusResponse::InvalidDatabase)?;
                        };
                    }
                    ShowArgument::Databases => {
                        write!(
                            stream,
                            "{}",
                            StatusResponse::DatabasesPresent {
                                n_databases: dbs.dbs.len()
                            }
                        )?;

                        for db in &dbs.dbs {
                            write!(stream, "{} {}\r\n", db.name, db.description)?;
                        }
                        write!(stream, ".\r\n")?;
                    }
                    ShowArgument::Strategies => {
                        // I'm not bothering to implement 555 No strategies available
                        // By the spec, I thought compliant servers must implement Exact & Prefix strategies anyways...
                        write!(
                            stream,
                            "{}",
                            StatusResponse::StrategiesAvailable {
                                n_strategies: SearchStrategy::VARIANTS.len()
                            }
                        )?;
                        for strat in SearchStrategy::VARIANTS {
                            write!(stream, "{strat}\r\n")?;
                        }
                        write!(stream, ".\r\n")?;
                    }
                    ShowArgument::Server => {
                        write!(stream, "{}", StatusResponse::ServerInformation)?;
                        write!(stream, "{SERVER_INFO}.\r\n")?;
                    }
                }
                write!(stream, "{}", StatusResponse::Ok)?;
            }
            Ok(Command::Status) => {
                write!(stream, "{}", StatusResponse::Status)?;
            }
            Ok(Command::OptionMIME) => {
                // WARNING: dictd uses "250 ok - using MIME headers\r\n"
                write!(
                    stream,
                    "{} ok - using MIME headers\r\n",
                    StatusResponse::Ok.code()
                )?;
                todo!();
            }
            // TODO
            Ok(Command::Auth { .. }) => {
                write!(stream, "{}", StatusResponse::CommandNotImplemented)?;
            }
            Err(status_response) => {
                write!(stream, "{status_response}")?;
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let index_path = env::args().nth(1).expect("Please pass in a .index file");
    let dict_path = env::args().nth(2).expect("Please pass in a .dict file");

    let db = Database::new(Path::new(&index_path), Path::new(&dict_path)).unwrap();
    let mut dbs = DatabaseEngine { dbs: vec![db] };

    let listener = TcpListener::bind(format!("127.0.0.1:{DICT_SERVER_PORT}"))?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match handle_connection(stream, &mut dbs) {
                Ok(_handled) => {
                    eprintln!("Client connection completed.");
                }
                Err(e) => {
                    eprintln!("Client disconnected {e}.")
                }
            },
            Err(e) => {
                eprintln!("{e}")
            }
        }
    }
    Ok(())
}
