use std::fmt;

#[derive(Debug)]
pub struct Database {
    pub(crate) name:          String,
    pub(crate) database_info: String,
}

#[derive(Debug)]
pub enum DatabaseLookupStrategy {
    Named(String), // specific database
    First,         // '!'
    All,           // '*
}

/// Unsupported variants include:
/// Substring, Suffix, Regex,
/// Soundex, Levenshtein
#[derive(Debug)]
pub enum SearchStrategy {
    Exact,
    Prefix,
    Default, // '.'
}

#[derive(Debug)]
pub struct Definition {
    pub database:   Database,
    pub head_word:  String,
    pub definition: String,
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
    StrategiesAvailable { n_strategies: usize },
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
    #[rustfmt::skip]
    pub const fn code(&self) -> u16 {
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

/// Format: status code <optional params> explanatory text
impl fmt::Display for StatusResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StatusResponse::*;
        let code = self.code();
        match self {
            DatabasesPresent { n_databases, text } => write!(
                f,
                "{} {n_databases} databases present - text follows\r\n",
                code
            ),
            StrategiesAvailable { n_strategies } => {
                write!(f, "{} {n_strategies} strategies present\r\n", code)
            }
            DatabaseInformation => {
                write!(f, "{} database information follows\r\n", code)
            }
            Help => write!(f, "{} help text follows\r\n", code),
            ServerInformation => {
                write!(f, "{} server information follows\r\n", code)
            }
            SASLChallengeFollows => {
                write!(f, "{} challenge follows\r\n", code)
            }
            WordFound { n_definitions } => {
                write!(f, "{} {n_definitions} definitions retrieved \r\n", code)
            }
            WordDefinition {
                word,
                database,
                definition,
            } => write!(
                f,
                "{} \"{word}\" {} \"{}\"\r\n{}\r\n",
                code, database.name, database.database_info, definition
            ),
            WordsMatched { n_matches } => {
                write!(f, "{} {n_matches} matches found - text follows\r\n", code)
            }
            Status => write!(f, "{} status <DUMMY_STATUS>\r\n", code),
            ClientIPAllowed => write!(f, "{} <DUMMY_REQUEST_ID>\r\n", code),
            Quit => write!(f, "{} bye\r\n", code),
            AuthenticationSuccessful => {
                write!(f, "{} authentication successful\r\n", code)
            }
            Ok => write!(f, "{} ok\r\n", code),
            SASLSendResponse => write!(f, "{} send response\r\n", code),
            ServerTemporarilyUnavailable => {
                write!(f, "{} server temporarily unavailable\r\n", code)
            }
            ServerShutdownOperatorRequest => {
                write!(f, "{} server shutting down at operator request\r\n", code)
            }
            SyntaxErrorCommandNotRecognised => {
                write!(f, "{} unknown command\r\n", code)
            }
            SyntaxErrorIllegalParameters => {
                write!(f, "{} syntax error, illegal parameters\r\n", code)
            }
            CommandNotImplemented => {
                write!(f, "{} command not implemented\r\n", code)
            }
            CommandParameterNotImplemented => {
                write!(f, "{} command parameter not implemented\r\n", code)
            }
            AccessDeniedIPBlocked => write!(f, "{} access denied\r\n", code),
            AccessDenied => write!(
                f,
                "{} access denied, use \"SHOW INFO\" for server information\r\n",
                code
            ),
            SASLUnknownMechanism => write!(f, "{} access denied, unknown mechanism\r\n", code),
            InvalidDatabase => write!(
                f,
                "{} invalid database, use \"SHOW DB\" for list of databases\r\n",
                code
            ),
            InvalidStrategy => write!(
                f,
                "{} invalid strategy, use \"SHOW STRAT\" for a list of strategies\r\n",
                code
            ),
            NoMatch => write!(f, "{} no match\r\n", code),
            NoDatabasesPresent => {
                write!(f, "{} no databases present\r\n", code)
            }
            NoStrategiesAvailable => {
                write!(f, "{} no strategies available\r\n", code)
            }
        }
    }
}
