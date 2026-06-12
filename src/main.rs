use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

const DICT_SERVER_PORT: u16 = 2628; // TODO what type for port numbers?
const LINE_BUFFER_MAX_CHARS: usize = 1024;
const LINE_BUFFER_MAX_BYTES: usize = 6144; // 1024 * 6

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    eprintln!("New client connection: {:#?}", &stream);
    let mut buffer = [0; LINE_BUFFER_MAX_BYTES];
    let bytes_read: usize = stream.read(&mut buffer)?;

    if let Ok(command) = str::from_utf8(&buffer[..bytes_read]) {
        assert!(command.len() <= LINE_BUFFER_MAX_CHARS);
        dbg!(command);
        // let status = StatusResponse::ServerTemporarilyUnavailable as u16;
        let status = StatusResponse::ServerTemporarilyUnavailable.discriminant();
        stream.write(format!("{status}\r\n").as_bytes())?;
    } else { // TODO require UTF-8 input??
        eprintln!("Client sent non-UTF-8 input.");
        // let status = StatusResponse::SyntaxErrorCommandNotRecognised as u16;
        let status = StatusResponse::SyntaxErrorCommandNotRecognised.discriminant();
        stream.write(format!("{status}\r\n").as_bytes())?;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{DICT_SERVER_PORT}"))?;


    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                match handle_connection(stream) {
                    Ok(_handled) => { eprintln!("Client connection completed.") }
                    Err(e) => { eprintln!("Client disconnected {e}.")}
                }
            }
            Err(e) => { eprintln!("{e}")}
        }
    }
    Ok(())
}

// https://curl.se/rfc/rfc2229.txt

// 2.3 Commands

// Commands consist of a command word followed by zero or more
// parameters.  Commands with parameters must separate the parameters
// from each other and from the command by one or more space or tab
// characters.  Command lines must be complete with all required
// parameters, and may not contain more than one command.

//    Each command line must be terminated by a CRLF.

//    The grammar for commands is:

//              command     = cmd-word *<WS cmd-param>
//              cmd-word    = atom
//              cmd-param   = database / strategy / word
//              database    = atom
//              strategy    = atom

//    Commands are not case sensitive.

// Command lines MUST NOT exceed 1024 characters in length, counting all
// characters including spaces, separators, punctuation, and the
// trailing CRLF.  There is no provision for the continuation of command
// lines.  Since UTF-8 may encode a character using up to 6 octets, the
// command line buffer MUST be able to accept up to 6144 octets.

const MIME_HEADER: &'static str =
    "Content-type: text/plain; charset=utf-8\n\rContent-transfer-encoding: 8bit";

#[derive(Debug)]
#[repr(u16)]
enum StatusResponse {
    // 1yz - Positive Preliminary reply
    DatabasesPresent(u8)               = 110, // * 110 n databases present - text follows
    StrategiesAvailable             = 111, // * 111 n strategies available - text follows
    DatabaseInformation             = 112, // 112 database information follows
    Help                            = 113, // 113 help text follows
    ServerInformation               = 114, // 114 server information follows
    SASLChallengeFollows            = 130, // 130 challenge follows
    WordFound                       = 150, // * 150 n definitions retrieved - definitions follow
    WordDefinition                  = 151, // * 151 word database name - text follows
    WordsMatched                    = 152, // * 152 n matches found - text follows

    // 2yz - Positive Completion reply
    Status                          = 210, // 210 (optional timing and statistical information here)
    ClientIPAllowed                 = 220, // * 220 text msg-id
    // ^ or ConnectionPermitted or Banner ?
    Quit                            = 221, // 221 Closing Connection
    AuthenticationSuccessful        = 230, // 230 Authentication successful
    Ok                              = 250, // 250 ok (optional timing information here)

    //  3yz - Positive Intermediate reply
    SASLSendResponse                = 330, // 330 send response

    // 4yz - Transient Negative Completion reply
    ServerTemporarilyUnavailable    = 420, // 420 Server temporarily unavailable
    ServerShutdownOperatorRequest   = 421, // 421 Server shutting down at operator request

    // 5yz - Permanent Negative Completion reply
    SyntaxErrorCommandNotRecognised = 500, // 500 Syntax error, command not recognized
    SyntaxErrorIllegalParameters    = 501, // 501 Syntax error, illegal parameters
    CommandNotImplemented           = 502, // 502 Command not implemented
    CommandParameterNotImplemented  = 503, // 503 Command parameter not implemented
    AccessDeniedIPBlocked           = 530, // 530 Access denied
    AccessDenied                    = 531, // 531 Access denied, use "SHOW INFO" for server information
    SASLUnknownMechanism            = 532, // 532 Access denied, unknown mechanism
    InvalidDatabase                 = 550, // 550 Invalid database, use "SHOW DB" for list of databases
    InvalidStrategy                 = 551, // 551 Invalid strategy, use "SHOW STRAT" for a list of strategies
    NoMatch                         = 552, // 552 No match
    NoDatabasesPresent              = 554, // 554 No databases present,
    NoStrategiesAvailable           = 555, // 555 No strategies available
}

impl StatusResponse {
    fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

// #[derive(Debug)]
// // #[repr(u8)]
// enum StatusResponse {
//     DatabasesPresent(u8, u8), // * 110 n databases present - text follows
//     DatabaseInformation(u8),  // 112 database information follows
// }

impl TryFrom<&str> for Command {
    type Error = StatusResponse;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens: Vec<&str> = value.split_ascii_whitespace().collect();
        match tokens[0].to_uppercase().as_str() {
            "QUIT" => Ok(Command::Quit),
            _ => Err(StatusResponse::SyntaxErrorCommandNotRecognised),
        }
    } // TODO
}

// 3.3.  The MATCH Command

// MATCH database strategy word

#[derive(Debug)]
enum Command {
    // DEFINE database word
    Define {
        database: Database,
        word:     String,
    },

    // MATCH database strategy word
    Match {
        database: Database,
        strategy: SearchStrategy,
        word:     Word,
    },

    // SHOW DB or SHOW DATABASES
    // SHOW STRAT or SHOW STRATEGIES
    // SHOW INFO database
    // SHOW SERVER
    Show(ShowArgument),

    //  CLIENT text
    Client {
        text: String,
    },

    Status,

    Help,

    Quit,

    OptionMIME,

    // AUTH username authentication-string
    Auth {
        username:              String,
        authentication_string: String,
    },

    // SASLAUTH mechanism initial-response
    // SASLRESP response
    SASLAuth {
        mechanism:        String,
        initial_response: Option<String>,
    },
}

#[derive(Debug)]
enum ShowArgument {
    Info { database: Database },
    Databases,
    Strategies,
    Server,
}

#[derive(Debug)]
enum Database {
    Named(String), // specific database
    First,         // '!'
    All,           // '*
}

#[derive(Debug)]
enum SearchStrategy {
    Exact,
    Prefix,
    Default, // '.'

    // Unsupported
    // Substring,
    // Suffix,
    // Regex,
    // Soundex,
    // Levenshtein
}

#[derive(Debug)]
struct Word {
    definitions: Vec<String>,
}
