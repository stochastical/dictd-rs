use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

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

        match Command::try_from(command_line) {
            Ok(Command::Quit) => {
                stream.write_all(StatusResponse::Quit.response_text().as_bytes())?;
                return Ok(());
            }
            Ok(Command::Help) => {
                stream.write_all(StatusResponse::Help.response_text().as_bytes())?;
                stream.write_all(HELP_LINES.join("\r\n").as_bytes())?;
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
                unimplemented!()
            }
            Ok(Command::OptionMIME) => {
                unimplemented!()
            }
            Ok(Command::Auth { .. }) | Ok(Command::SASLAuth { .. }) => {
                unimplemented!()
            }
            Err(ParseError::InvalidCommand) => {
                stream.write_all(StatusResponse::SyntaxErrorCommandNotRecognised.response_text().as_bytes())?;
            }
            Err(ParseError::InvalidArguments) => {
                stream.write_all(StatusResponse::SyntaxErrorIllegalParameters.response_text().as_bytes())?;
            }

            // Err(status @ StatusResponse::SyntaxErrorIllegalParameters) => {
            //     stream.write(status.response_text().as_bytes())?;
            // },
            _ => unimplemented!(),
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

// https://curl.se/rfc/rfc2229.txt
//
// 2.3 Commands
//
// Commands consist of a command word followed by zero or more
// parameters.  Commands with parameters must separate the parameters
// from each other and from the command by one or more space or tab
// characters.  Command lines must be complete with all required
// parameters, and may not contain more than one command.
//
//    Each command line must be terminated by a CRLF.
//
//    The grammar for commands is:
//
//              command     = cmd-word *<WS cmd-param>
//              cmd-word    = atom
//              cmd-param   = database / strategy / word
//              database    = atom
//              strategy    = atom
//
//    Commands are not case sensitive.
//
// Command lines MUST NOT exceed 1024 characters in length, counting all
// characters including spaces, separators, punctuation, and the
// trailing CRLF.  There is no provision for the continuation of command
// lines.  Since UTF-8 may encode a character using up to 6 octets, the
// command line buffer MUST be able to accept up to 6144 octets.

#[derive(Debug)]
enum StatusResponse {
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
    WordFound {
        n_definitions: usize,
    },
    /// * 151 word database name - text follows      
    WordDefinition {
        word:       String,
        database:   Database,
        definition: String,
    },
    /// * 152 n matches found - text follows   
    WordsMatched,

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

    NoCommandParsed,
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
            status @ DatabaseInformation => todo!(),
            status @ Help => format!("{} help text follows\r\n", status.status_code()),
            status @ ServerInformation => todo!(),
            status @ SASLChallengeFollows => todo!(),
            status @ WordFound { n_definitions } => format!(
                "{} {n_definitions} definitions retrieved \r\n",
                status.status_code()
            ),
            status @ WordDefinition {
                word,
                database,
                definition,
            } => format!(
                "{} {word} {} {}\r\n{}\r\n",
                status.status_code(),
                database.name,
                database.database_info,
                definition
            ),
            status @ WordsMatched => todo!(),
            status @ Status => todo!(),
            status @ ClientIPAllowed => format!("{} DUMMY_REQUEST_ID\r\n", status.status_code()),
            status @ Quit => format!("{} bye\r\n", status.status_code()),
            status @ AuthenticationSuccessful => todo!(),
            status @ Ok => format!("{} ok\r\n", status.status_code()),
            status @ SASLSendResponse => todo!(),
            status @ ServerTemporarilyUnavailable => todo!(),
            status @ ServerShutdownOperatorRequest => todo!(),
            status @ SyntaxErrorCommandNotRecognised => {
                format!("{} unknown command\r\n", status.status_code())
            }
            status @ SyntaxErrorIllegalParameters => todo!(),
            status @ CommandNotImplemented => todo!(),
            status @ CommandParameterNotImplemented => todo!(),
            status @ AccessDeniedIPBlocked => todo!(),
            status @ AccessDenied => todo!(),
            status @ SASLUnknownMechanism => todo!(),
            status @ InvalidDatabase => todo!(),
            status @ InvalidStrategy => todo!(),
            status @ NoMatch => format!("{} no match\r\n", status.status_code()),
            status @ NoDatabasesPresent => todo!(),
            status @ NoStrategiesAvailable => todo!(),

            status @ NoCommandParsed => unimplemented!(),
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
            WordsMatched                    => 152,
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
            NoStrategiesAvailable           => 555,

            NoCommandParsed => todo!(),
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
                    name: db.clone(),
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

#[derive(Debug)]
enum Command {
    /// DEFINE database word
    Define {
        database: DatabaseLookupStrategy,
        word:     String,
    },

    /// MATCH database strategy word
    Match {
        database: DatabaseLookupStrategy,
        strategy: SearchStrategy,
        word:     String,
    },

    /// SHOW DB or SHOW DATABASES
    /// SHOW STRAT or SHOW STRATEGIES
    /// SHOW INFO database
    /// SHOW SERVER
    Show(ShowArgument),

    ///  CLIENT text
    Client { text: String },

    /// STATUS
    Status,

    /// HELP
    Help,

    /// QUIT
    Quit,

    /// OPTION MIME
    OptionMIME,

    /// AUTH username authentication-string
    Auth {
        username:              String,
        authentication_string: String,
    },

    /// SASLAUTH mechanism initial-response
    /// SASLRESP response
    SASLAuth {
        mechanism:        String,
        initial_response: Option<String>,
    },
}

// Plan: define custom Parse error types,
// and map those to the appropriate StatusResponse in the main loop.
#[derive(Debug)]
enum ParseError {
    InvalidCommand,
    InvalidArguments,
}

impl TryFrom<&str> for Command {
    type Error = ParseError;

    // QUESTION: Should the parser validate the command line length and UTF-8 encoding before trying to parse the command?
    // TODO: Commands and arguments can be quoted! The tokeniser needs to handle quoted strings
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens: Vec<&str> = dbg!(value.split_ascii_whitespace().collect());

        match dbg!(tokens[0].to_uppercase().as_str()) {
            "DEFINE" => {
                match tokens[1..] {
                    [database_token, word_token] => {
                        Ok(Command::Define {
                            database: match database_token {
                                "!" => DatabaseLookupStrategy::First,
                                "*" => DatabaseLookupStrategy::All,
                                // TODO: need to validate that the database name is valid (and exists?)
                                db_name => unimplemented!(),
                            },
                            word:     word_token.to_string(),
                        })
                    }
                    _ => Err(ParseError::InvalidArguments),
                }
            }
            "MATCH" => unimplemented!(),
            "SHOW" => {
                match tokens[1].to_uppercase().as_str() {
                    "DB" | "DATABASES" => Ok(Command::Show(ShowArgument::Databases)),
                    "STRAT" | "STRATEGIES" => Ok(Command::Show(ShowArgument::Strategies)),
                    "SERVER" => Ok(Command::Show(ShowArgument::Server)),
                    "INFO" => {
                        // TODO: I need to validate there's exactly one more token, and that it's a valid database name (and exists?)
                        // Should I be resolving and validating the database name in the parser, or should I just pass it through and let the handler deal with it?
                        // or do I have an impl Database::new that does the validation?
                        unimplemented!()
                    }
                    _ => Err(ParseError::InvalidArguments),
                }
            }
            "CLIENT" => Ok(Command::Client {
                text: dbg!(tokens[1..].join(" ")),
            }),
            "STATUS" => Ok(Command::Status),
            "HELP" => Ok(Command::Help),
            "QUIT" => Ok(Command::Quit),
            "OPTION" => match dbg!(tokens[1].to_uppercase().as_str()) {
                "MIME" => Ok(Command::OptionMIME),
                _ => Err(ParseError::InvalidArguments),
            },
            "AUTH" => unimplemented!(),
            "SASLAUTH" | "SASLRESP" | _ => Err(ParseError::InvalidCommand),
        }
    }
}

#[derive(Debug)]
enum ShowArgument {
    Info { database: Database },
    Databases,
    Strategies,
    Server,
}

#[derive(Debug)]
struct Database {
    name:          String,
    database_info: String,
}

#[derive(Debug)]
enum DatabaseLookupStrategy {
    Named(String), // specific database
    First,           // '!'
    All,             // '*
}

/// Unsupported variants include:
/// Substring, Suffix, Regex,
/// Soundex, Levenshtein
#[derive(Debug)]
enum SearchStrategy {
    Exact,
    Prefix,
    Default, // '.'
}

#[derive(Debug)]
struct Definition {
    database:   Database,
    head_word:  String,
    definition: String,
}

#[cfg(test)]
mod test;
