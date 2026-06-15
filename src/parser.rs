use std::{env::Args, option};

use crate::types::{DatabaseLookupStrategy, SearchStrategy};

// PLAN: define custom Parse error types, and map those to the appropriate StatusResponse in the main loop.
#[derive(Debug)]
pub enum ParseError {
    InvalidCommand,
    InvalidArguments,
    InvalidStrategy,
    InvalidDatabase,
}

#[derive(Debug)]
pub enum ShowArgument {
    Info { database: DatabaseLookupStrategy },
    Databases,
    Strategies,
    Server,
}

/// https://curl.se/rfc/rfc2229.txt
///
/// 2.3 Commands
/// Commands consist of a command word followed by zero or more
/// parameters.  Commands with parameters must separate the parameters
/// from each other and from the command by one or more space or tab
/// characters.  Command lines must be complete with all required
/// parameters, and may not contain more than one command.
///
/// Each command line must be terminated by a CRLF.
///
/// The grammar for commands is:
///
///              command     = cmd-word *<WS cmd-param>
///              cmd-word    = atom
///              cmd-param   = database / strategy / word
///              database    = atom
///              strategy    = atom
/// Commands are not case sensitive.
/// Command lines MUST NOT exceed 1024 characters in length, counting all
/// characters including spaces, separators, punctuation, and the
/// trailing CRLF.  There is no provision for the continuation of command
/// lines.  Since UTF-8 may encode a character using up to 6 octets, the
/// command line buffer MUST be able to accept up to 6144 octets.
#[derive(Debug)]
pub enum Command {
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
    /// SHOW DB | DATABASES
    /// SHOW STRAT | STRATEGIES
    /// SHOW SERVER
    /// SHOW INFO database
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
    // SASLAUTH mechanism initial-response
    // SASLRESP response
    // SASLAuth {
    //     mechanism:        String,
    //     initial_response: Option<String>,
    // },
}

/// As an invariant, we assume that the command line has already been validated for UTF-8 encoding and max length.
impl TryFrom<&str> for Command {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // TODO: We need to support quoted atoms
        let tokens: Vec<&str> = value.split_ascii_whitespace().collect();
        dbg!(&tokens);

        let Some(cmd) = tokens.first() else {
            return Err(ParseError::InvalidCommand);
        };

        match (cmd.to_uppercase().as_str(), &tokens[1..]) {
            // DEFINE database word
            ("DEFINE", [database, word]) => Ok(Command::Define {
                database: match *database {
                    "!" => DatabaseLookupStrategy::First,
                    "*" => DatabaseLookupStrategy::All,
                    db_name => DatabaseLookupStrategy::Named(db_name.to_string()),
                },
                word:     word.to_string(),
            }),
            ("DEFINE", _) => Err(ParseError::InvalidArguments),

            // MATCH database strategy word
            ("MATCH", [database, strategy, word]) => Ok(Command::Match {
                database: match *database {
                    "!" => DatabaseLookupStrategy::First,
                    "*" => DatabaseLookupStrategy::All,
                    db_name => DatabaseLookupStrategy::Named(db_name.to_string()),
                },
                strategy: match strategy.to_uppercase().as_str() {
                    "." => SearchStrategy::Default,
                    "EXACT" => SearchStrategy::Exact,
                    "PREFIX" => SearchStrategy::Prefix,
                    _ => return Err(ParseError::InvalidStrategy), // TODO: is this considered validation?
                },
                word:     word.to_string(),
            }),
            ("MATCH", _) => Err(ParseError::InvalidArguments),

            // SHOW DB | DATABASES
            // SHOW STRAT | STRATEGIES
            // SHOW SERVER
            // SHOW INFO database
            ("SHOW", [arg, rest @ ..]) => match arg.to_uppercase().as_str() {
                "DB" | "DATABASES" => Ok(Command::Show(ShowArgument::Databases)),
                "STRAT" | "STRATEGIES" => Ok(Command::Show(ShowArgument::Strategies)),
                "SERVER" => Ok(Command::Show(ShowArgument::Server)),

                "INFO" => match rest {
                    [db_name] => Ok(Command::Show(ShowArgument::Info {
                        database: DatabaseLookupStrategy::Named(db_name.to_string()),
                    })),
                    _ => Err(ParseError::InvalidArguments),
                },

                _ => Err(ParseError::InvalidArguments),
            },

            //  CLIENT text
            ("CLIENT", [info @ ..]) => Ok(Command::Client {
                text: info.join(" ").to_string(),
            }),

            // STATUS
            ("STATUS", []) => Ok(Command::Status),

            // HELP
            ("HELP", []) => Ok(Command::Help),

            // QUIT
            ("QUIT", []) => Ok(Command::Quit),

            // OPTION MIME
            ("OPTION", [opt]) if opt.eq_ignore_ascii_case("MIME") => Ok(Command::OptionMIME),
            ("OPTION", _) => Err(ParseError::InvalidArguments),

            // AUTH username authentication-string
            ("AUTH", [username, authentication_string]) => Ok(Command::Auth {
                username:              username.to_string(),
                authentication_string: authentication_string.to_string(),
            }),
            ("AUTH", _) => Err(ParseError::InvalidArguments),

            _ => Err(ParseError::InvalidCommand),
        }
    }
}
