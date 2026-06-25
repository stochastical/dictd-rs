use std::env;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use server::database::{Database, DatabaseEngine};
use server::parser::Command;
use server::protocol::{HELP_LINES, SearchStrategy, StatusResponse};
use uuid::Uuid;

const DICT_SERVER_PORT: u16 = 2628;
const LINE_BUFFER_MAX_CHARS: usize = 1024;
const LINE_BUFFER_MAX_BYTES: usize = 6144; // 1024 * 6
const MIME_HEADER: &[&str] = &[
    "Content-type: text/plain; charset=utf-8",
    "Content-transfer-encoding: 8bit",
];

/// I think we can bubble errors through here (e.g. client disconnects, and let the caller process it)
/// TODO: it'd be nice to return a StatusResponse maybe?
/// QUESTION: does the spec allow for multiple commands in a single connection? If so, we need to loop and read until the client disconnects, rather than returning after handling one command. Should there be a timeout on the connection?
/// TODO: What's the best way to do dependency injection (i.e. we need to know about all databases etc...)
fn handle_connection(mut stream: TcpStream, dbs: &mut DatabaseEngine) -> std::io::Result<()> {
    eprintln!("New client connection: {:#?}", &stream);
    write!(
        stream,
        "{}\r\n",
        StatusResponse::ClientIPAllowed {
            text: "dictd-rs".into(),
            msg_id: Uuid::new_v4(),
        }
    )?;

    loop {
        let mut buffer = [0; LINE_BUFFER_MAX_BYTES];
        let bytes_read: usize = stream.read(&mut buffer)?;

        // Validate bytes read and UTF-8 encoding before trying to parse the command.
        if bytes_read == 0 {
            eprintln!("Client dropped connection. Connection closed.");
            break;
        }
        let Ok(command_line) = dbg!(str::from_utf8(&buffer[..bytes_read])) else {
            eprintln!("Client sent invalid UTF-8.");
            write!(
                stream,
                "{}",
                StatusResponse::SyntaxErrorCommandNotRecognised
            )?;
            continue;
        };
        if command_line.chars().count() > LINE_BUFFER_MAX_CHARS {
            write!(stream, "{}", StatusResponse::SyntaxErrorIllegalParameters)?;
            continue;
        }

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
                        write!(stream, "{definition}", )?;
                        write!(stream, ".\r\n")?;
                    }
                    write!(stream, "{}", StatusResponse::Ok)?;
                }
            }
            Ok(Command::Match {
                lookup,
                strategy,
                word,
            }) => {
                let definitions = dbs.lookup(&word, lookup, strategy);

                if definitions.is_empty() {
                    write!(stream, "{}", StatusResponse::NoMatch)?;
                } else {
                    write!(
                        stream,
                        "{}",
                        StatusResponse::WordsMatched {
                            n_matches: definitions.len(),
                        }
                    )?;
                    for definition in definitions {
                        write!(stream, "{definition}")?;
                        write!(stream, ".\r\n")?;
                    }
                    write!(stream, "{}", StatusResponse::Ok)?;
                }
            }
            Ok(Command::Show(_)) => {
                todo!()
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
            Ok(Command::Auth { .. }) => {
                write!(stream, "{}", StatusResponse::CommandNotImplemented)?;
                unimplemented!()
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

    let mut db = Database::new(Path::new(&index_path), Path::new(&dict_path)).unwrap();
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
