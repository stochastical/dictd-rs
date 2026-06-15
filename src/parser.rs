use std::env::Args;

use crate::types::{DatabaseLookupStrategy, SearchStrategy};

// PLAN: define custom Parse error types, and map those to the appropriate StatusResponse in the main loop.
#[derive(Debug)]
pub enum ParseError {
    InvalidCommand,
    InvalidArguments,
}

#[derive(Debug)]
pub enum ShowArgument {
    Info { database: DatabaseLookupStrategy },
    Databases,
    Strategies,
    Server,
}

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

        match tokens.as_slice() {
            [] => Err(ParseError::InvalidCommand), // TODO: I don't want to 
            [cmd, database, word] if cmd.eq_ignore_ascii_case("DEFINE") => Ok(Command::Define {
                database: match *database {
                    "!" => DatabaseLookupStrategy::First,
                    "*" => DatabaseLookupStrategy::All,
                    db_name => DatabaseLookupStrategy::Named(db_name.to_string()),
                },
                word:     word.to_string(),
            }),
            [cmd, database, strategy, word] if cmd.eq_ignore_ascii_case("MATCH") => {
                Ok(Command::Match {
                    database: match *database {
                        "!" => DatabaseLookupStrategy::First,
                        "*" => DatabaseLookupStrategy::All,
                        db_name => DatabaseLookupStrategy::Named(db_name.to_string()),
                    },
                    strategy: match strategy.to_uppercase().as_str() {
                        "." => SearchStrategy::Default,
                        "EXACT" => SearchStrategy::Exact,
                        "PREFIX" => SearchStrategy::Prefix,
                        _ => return Err(ParseError::InvalidArguments),
                    },
                    word:     word.to_string(),
                })
            },
            [cmd, rest @ ..] if cmd.eq_ignore_ascii_case("SHOW") => match rest {
                [arg] => match arg.to_uppercase().as_ref() {
                    "DB" | "DATABASES" => Ok(Command::Show(ShowArgument::Databases)),
                    "STRAT" | "STRATEGIES" => Ok(Command::Show(ShowArgument::Strategies)),
                    "SERVER" => Ok(Command::Show(ShowArgument::Server)),
                    _ => Err(ParseError::InvalidArguments),
                },
                [arg, db_name] if arg.eq_ignore_ascii_case("INFO") => {
                    Ok(Command::Show(ShowArgument::Info {
                        database: DatabaseLookupStrategy::Named(db_name.to_string()),
                    }))
                }
                _ => Err(ParseError::InvalidArguments),
            },
            [cmd, "MIME"] if cmd.eq_ignore_ascii_case("OPTION") => Ok(Command::OptionMIME),
            [cmd, info] if cmd.eq_ignore_ascii_case("CLIENT") => Ok(Command::Client {
                text: info.to_string(),
            }),
            [cmd, user, auth_string] if cmd.eq_ignore_ascii_case("AUTH") => Ok(Command::Auth {
                username: user.to_string(),
                authentication_string: auth_string.to_string(),
            }),
            [cmd] if cmd.eq_ignore_ascii_case("STATUS") => Ok(Command::Status),
            [cmd] if cmd.eq_ignore_ascii_case("HELP") => Ok(Command::Help),
            [cmd] if cmd.eq_ignore_ascii_case("QUIT") => Ok(Command::Quit),
            _ => Err(ParseError::InvalidCommand),
        }
    }
}
