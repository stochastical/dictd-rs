use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

mod parser;
mod types;

use parser::{Command};
use types::{Database, DatabaseLookupStrategy, Definition, SearchStrategy};

const DICT_SERVER_PORT: u16 = 2628;
const LINE_BUFFER_MAX_CHARS: usize = 1024;
const LINE_BUFFER_MAX_BYTES: usize = 6144; // 1024 * 6
const MIME_HEADER: &'static str =
    "Content-type: text/plain; charset=utf-8\n\rContent-transfer-encoding: 8bit";

const HELP_LINES: &[&str] = &[
    "DEFINE database word         -- look up word in database",
    "MATCH database strategy word -- match word in database using strategy",
    "SHOW DB                      -- list all accessible databases",
    "SHOW DATABASES               -- list all accessible databases",
    "SHOW STRAT                   -- list available matching strategies",
    "SHOW STRATEGIES              -- list available matching strategies",
    "SHOW INFO database           -- provide information about the database",
    "SHOW SERVER                  -- provide site-specific information",
    "OPTION MIME                  -- use MIME headers",
    "CLIENT info                  -- identify client to server",
    "AUTH user string             -- provide authentication information",
    "STATUS                       -- display timing information",
    "HELP                         -- display this help information",
    "QUIT                         -- terminate connection",
];

/// I think we can bubble errors through here (e.g. client disconnects, and let the caller process it)
/// TODO: it'd be nice to return a StatusResponse maybe?
fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    eprintln!("New client connection: {:#?}", &stream);
    stream.write_all(StatusResponse::ClientIPAllowed.response_text().as_bytes())?;

    loop {
        let mut buffer = [0; LINE_BUFFER_MAX_BYTES];
        let bytes_read: usize = stream.read(&mut buffer)?;

        // Validate bytes read and UTF-8 encoding before trying to parse the command.
        if bytes_read == 0 {
            eprintln!("Client dropped connection. Connection closed.");
            break;
        }
        let Ok(command_line) = str::from_utf8(&buffer[..bytes_read]) else {
            eprintln!("Client sent invalid UTF-8.");
            stream.write_all(
                StatusResponse::SyntaxErrorCommandNotRecognised
                    .response_text()
                    .as_bytes(),
            )?;
            continue;
        };
        if command_line.chars().count() > LINE_BUFFER_MAX_CHARS {
            stream.write_all(
                StatusResponse::SyntaxErrorIllegalParameters
                    .response_text()
                    .as_bytes(),
            )?;
            continue;
        }

        match dbg!(Command::try_from(command_line)) {
            Ok(Command::Quit) => {
                stream.write_all(StatusResponse::Quit.response_text().as_bytes())?;
                return Ok(());
            }
            Ok(Command::Help) => {
                stream.write_all(StatusResponse::Help.response_text().as_bytes())?;
                stream.write_all(HELP_LINES.join("\r\n").as_bytes())?;
                stream.write_all("\r\n.\r\n".as_bytes())?;
                stream.write_all(StatusResponse::Ok.response_text().as_bytes())?;
            }
            Ok(Command::Client { .. }) => {
                stream.write_all(StatusResponse::Ok.response_text().as_bytes())?;
            }
            Ok(Command::Define { database, word }) => {
                let definitions = define_word(word, database);
                if definitions.is_empty() {
                    stream.write_all(StatusResponse::NoMatch.response_text().as_bytes())?;
                } else {
                    stream.write_all(
                        StatusResponse::WordFound {
                            n_definitions: definitions.len(),
                        }
                        .response_text()
                        .as_bytes(),
                    )?;
                    for definition in definitions {
                        stream.write_all(
                            StatusResponse::WordDefinition {
                                word:       definition.head_word,
                                database:   definition.database,
                                definition: definition.definition,
                            }
                            .response_text()
                            .as_bytes(),
                        )?;
                        stream.write_all(".\r\n".as_bytes())?;
                    }
                    stream.write_all(StatusResponse::Ok.response_text().as_bytes())?;
                }
            }
            Ok(Command::Match {
                database,
                strategy,
                word,
            }) => {
                unimplemented!()
            }
            Ok(Command::Show(_)) => {
                unimplemented!()
            }

            Ok(Command::Status) => {
                stream.write_all(StatusResponse::Status.response_text().as_bytes())?;
            }
            Ok(Command::OptionMIME) => {
                // WARNING: dictd uses "250 ok - using MIME headers\r\n"
                stream.write_all(
                    format!(
                        "{} ok - using MIME headers\r\n{}",
                        StatusResponse::Ok.status_code(),
                        MIME_HEADER
                    )
                    .as_bytes(),
                )?;
                unimplemented!();
            }
            Ok(Command::Auth { .. }) => {
                stream.write_all(
                    StatusResponse::CommandNotImplemented
                        .response_text()
                        .as_bytes(),
                )?;
                todo!()
            }
            Err(status_response) => {
                stream.write_all(
                    status_response
                        .response_text()
                        .as_bytes(),
                )?;
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{DICT_SERVER_PORT}"))?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => match handle_connection(stream) {
                Ok(_handled) => {
                    // let status_code = status.discriminant();
                    // stream.write(format!("{status_code}\r\n").as_bytes())?;
                    // ^ hmm, at this point we can't access the stream anymore!
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

#[derive(Debug)]
pub enum StatusResponse {
    // 1yz - Positive Preliminary reply
    /// * 110 n databases present - text follows
    DatabasesPresent {
        n_databases: usize,
        text:        String,
    },
    /// * 111 n strategies available - text follows
    StrategiesAvailable {
        n_strategies: usize,
        strategies:   Vec<SearchStrategy>,
    },
    /// 112 database information follows
    DatabaseInformation,
    /// 113 help text follows
    Help,
    /// 114 server information follows           
    ServerInformation,
    /// 130 challenge follows
    SASLChallengeFollows,
    /// * 150 n definitions retrieved - definitions follow
    WordFound { n_definitions: usize },
    /// * 151 word database name - text follows      
    WordDefinition {
        word:       String,
        database:   Database,
        definition: String,
    },
    /// * 152 n matches found - text follows   
    WordsMatched { n_matches: usize },

    // 2yz - Positive Completion reply
    /// 210 (optional timing and statistical information here)
    Status,
    /// * 220 text msg-id            
    ClientIPAllowed,
    /// 221 Closing Connection   
    Quit,
    /// 230 Authentication successful     
    AuthenticationSuccessful,
    /// 250 ok (optional timing information here)
    Ok,

    //  3yz - Positive Intermediate reply
    /// 330 send response
    SASLSendResponse,

    // 4yz - Transient Negative Completion reply
    /// 420 Server temporarily unavailable
    ServerTemporarilyUnavailable,
    /// 421 Server shutting down at operator request
    ServerShutdownOperatorRequest,

    // 5yz - Permanent Negative Completion reply
    /// 500 Syntax error, command not recognized
    SyntaxErrorCommandNotRecognised,
    /// 501 Syntax error, illegal parameters
    SyntaxErrorIllegalParameters,
    /// 502 Command not implemented
    CommandNotImplemented,
    /// 503 Command parameter not implemented
    CommandParameterNotImplemented,
    /// 530 Access denied
    AccessDeniedIPBlocked,
    /// 531 Access denied, use "SHOW INFO" for server information
    AccessDenied,
    /// 532 Access denied, unknown mechanism
    SASLUnknownMechanism,
    /// 550 Invalid database, use "SHOW DB" for list of databases
    InvalidDatabase,
    /// 551 Invalid strategy, use "SHOW STRAT" for a list of strategies
    InvalidStrategy,
    /// 552 No match
    NoMatch,
    /// 554 No databases present,
    NoDatabasesPresent,
    /// 555 No strategies available
    NoStrategiesAvailable,
}

impl StatusResponse {
    /// REFACTOR: I think this should set out the template, which includes the
    /// unless, is there a way to return a template, and let it be filled in later?
    /// should the e.g. HELP text be stored in the enum variant itself? Or is it better to have a separate function that generates the text based on the variant and its params? Should the databases and strategies be stored in the enum variant itself, or should they be passed in as params to the function that generates the text?
    /// Format: status code <optional params> explanatory text
    fn response_text(&self) -> String {
        use StatusResponse::*;
        match self {
            status @ DatabasesPresent { n_databases, text } => format!(
                "{} {n_databases} databases present - text follows\r\n",
                status.status_code()
            ),
            status @ StrategiesAvailable {
                n_strategies,
                strategies,
            } => format!(
                "{} {n_strategies} strategies present\r\n",
                status.status_code()
            ),
            status @ DatabaseInformation => {
                format!("{} database information follows\r\n", status.status_code())
            }
            status @ Help => format!("{} help text follows\r\n", status.status_code()),
            status @ ServerInformation => {
                format!("{} server information follows\r\n", status.status_code())
            }
            status @ SASLChallengeFollows => {
                format!("{} challenge follows\r\n", status.status_code())
            }
            status @ WordFound { n_definitions } => format!(
                "{} {n_definitions} definitions retrieved \r\n",
                status.status_code()
            ),
            status @ WordDefinition {
                word,
                database,
                definition,
            } => format!(
                "{} \"{word}\" {} \"{}\"\r\n{}\r\n",
                status.status_code(),
                database.name,
                database.database_info,
                definition
            ),
            status @ WordsMatched { n_matches } => format!(
                "{} {n_matches} matches found - text follows\r\n",
                status.status_code()
            ),
            status @ Status => format!("{} status <DUMMY_STATUS>\r\n", status.status_code()),
            status @ ClientIPAllowed => format!("{} <DUMMY_REQUEST_ID>\r\n", status.status_code()),
            status @ Quit => format!("{} bye\r\n", status.status_code()),
            status @ AuthenticationSuccessful => {
                format!("{} authentication successful\r\n", status.status_code())
            }
            status @ Ok => format!("{} ok\r\n", status.status_code()),
            status @ SASLSendResponse => format!("{} send response\r\n", status.status_code()),
            status @ ServerTemporarilyUnavailable => format!(
                "{} server temporarily unavailable\r\n",
                status.status_code()
            ),
            status @ ServerShutdownOperatorRequest => format!(
                "{} server shutting down at operator request\r\n",
                status.status_code()
            ),
            status @ SyntaxErrorCommandNotRecognised => {
                format!("{} unknown command\r\n", status.status_code())
            }
            status @ SyntaxErrorIllegalParameters => {
                format!(
                    "{} syntax error, illegal parameters\r\n",
                    status.status_code()
                )
            }
            status @ CommandNotImplemented => {
                format!("{} command not implemented\r\n", status.status_code())
            }
            status @ CommandParameterNotImplemented => {
                format!(
                    "{} command parameter not implemented\r\n",
                    status.status_code()
                )
            }
            status @ AccessDeniedIPBlocked => format!("{} access denied\r\n", status.status_code()),
            status @ AccessDenied => format!(
                "{} access denied, use \"SHOW INFO\" for server information\r\n",
                status.status_code()
            ),
            status @ SASLUnknownMechanism => format!(
                "{} access denied, unknown mechanism\r\n",
                status.status_code()
            ),
            status @ InvalidDatabase => format!(
                "{} invalid database, use \"SHOW DB\" for list of databases\r\n",
                status.status_code()
            ),
            status @ InvalidStrategy => format!(
                "{} invalid strategy, use \"SHOW STRAT\" for a list of strategies\r\n",
                status.status_code()
            ),
            status @ NoMatch => format!("{} no match\r\n", status.status_code()),
            status @ NoDatabasesPresent => {
                format!("{} no databases present\r\n", status.status_code())
            }
            status @ NoStrategiesAvailable => {
                format!("{} no strategies available\r\n", status.status_code())
            }
        }
    }

    #[rustfmt::skip]
    const fn status_code(&self) -> u16 {
        use StatusResponse::*;
        match self {
            DatabasesPresent { .. }         => 110,
            StrategiesAvailable { .. }      => 111,
            DatabaseInformation             => 112,
            Help                            => 113,
            ServerInformation               => 114,
            SASLChallengeFollows            => 130,
            WordFound { .. }                => 150,
            WordDefinition { .. }           => 151,
            WordsMatched { .. }             => 152,
            Status                          => 210,
            ClientIPAllowed                 => 220,
            Quit                            => 221,
            AuthenticationSuccessful        => 230,
            Ok                              => 250,
            SASLSendResponse                => 330,
            ServerTemporarilyUnavailable    => 420,
            ServerShutdownOperatorRequest   => 421,
            SyntaxErrorCommandNotRecognised => 500,
            SyntaxErrorIllegalParameters    => 501,
            CommandNotImplemented           => 502,
            CommandParameterNotImplemented  => 503,
            AccessDeniedIPBlocked           => 530,
            AccessDenied                    => 531,
            SASLUnknownMechanism            => 532,
            InvalidDatabase                 => 550,
            InvalidStrategy                 => 551,
            NoMatch                         => 552,
            NoDatabasesPresent              => 554,
            NoStrategiesAvailable           => 555
        }
    }
}

/// TEST: mock dictionary lookup
fn define_word(word: String, database_lookup_strategy: DatabaseLookupStrategy) -> Vec<Definition> {
    let test_db_1 = Database {
        name:          "testdb1".to_string(),
        database_info: "This is a test database".to_string(),
    };

    let test_db_2 = Database {
        name:          "testdb2".to_string(),
        database_info: "This is another test database".to_string(),
    };

    match database_lookup_strategy {
        DatabaseLookupStrategy::Named(db) => {
            eprintln!("Looking up word '{}' in database '{}'", word, db);
            vec![Definition {
                database:   Database {
                    name:          db.clone(),
                    database_info: format!("This is the {} database", db),
                },
                head_word:  word.clone(),
                definition: format!("Dummy definition of {word}"),
            }]
        }
        DatabaseLookupStrategy::First => {
            eprintln!("Looking up word '{}' in first available database", word);
            vec![Definition {
                database:   test_db_1,
                head_word:  word.clone(),
                definition: format!("Dummy definition of {word}"),
            }]
        }
        DatabaseLookupStrategy::All => {
            eprintln!("Looking up word '{}' in all available databases", word);
            vec![
                Definition {
                    database:   test_db_1,
                    head_word:  word.clone(),
                    definition: format!("Dummy definition 1 of {word}"),
                },
                Definition {
                    database:   test_db_2,
                    head_word:  word.clone(),
                    definition: format!("Dummy definition 2 of {word}"),
                },
            ]
        }
    }
}

#[cfg(test)]
mod test;
