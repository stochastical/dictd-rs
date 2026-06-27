use std::fmt::{self, Display};

use uuid::Uuid;

pub const HELP_LINES: &[&str] = &[
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

#[derive(Debug)]
pub enum StatusResponse {
    // 1yz - Positive Preliminary reply
    /// * 110 n databases present - text follows
    DatabasesPresent { n_databases: usize },
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
        headword: String,
        db_name: String,
        db_info: String,
        definition: String,
    },
    /// * 152 n matches found - text follows   
    WordsMatched { n_matches: usize },

    // 2yz - Positive Completion reply
    /// 210 (optional timing and statistical information here)
    Status,
    /// * 220 text msg-id            
    ClientIPAllowed { text: String, msg_id: Uuid },
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
            ClientIPAllowed { .. }          => 220,
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
            DatabasesPresent { n_databases } => write!(
                f,
                "{code} {n_databases} databases present - text follows\r\n"
            ),
            StrategiesAvailable { n_strategies } => {
                write!(f, "{code} {n_strategies} strategies present\r\n")
            }
            DatabaseInformation => write!(f, "{code} database information follows\r\n"),
            Help => write!(f, "{code} help text follows\r\n"),
            ServerInformation => write!(f, "{code} server information follows\r\n"),

            SASLChallengeFollows => {
                write!(f, "{code} challenge follows\r\n")
            }
            WordFound { n_definitions } => {
                write!(f, "{code} {n_definitions} definitions retrieved \r\n")
            }
            WordDefinition {
                headword,
                db_name,
                db_info,
                definition,
            } => write!(
                f,
                "{code} \"{headword}\" {db_name} \"{db_info}\"\r\n{definition}\r\n"
            ),
            WordsMatched { n_matches } => {
                write!(f, "{code} {n_matches} matches found - text follows\r\n")
            }
            Status => write!(f, "{code} status <DUMMY_STATUS>\r\n"),
            ClientIPAllowed { text, msg_id } => write!(f, "{code} {msg_id} {text}\r\n"),
            Quit => write!(f, "{code} bye\r\n"),
            AuthenticationSuccessful => write!(f, "{code} authentication successful\r\n"),
            Ok => write!(f, "{code} ok\r\n"),
            SASLSendResponse => write!(f, "{code} send response\r\n"),
            ServerTemporarilyUnavailable => write!(f, "{code} server temporarily unavailable\r\n"),
            ServerShutdownOperatorRequest => {
                write!(f, "{code} server shutting down at operator request\r\n")
            }
            SyntaxErrorCommandNotRecognised => write!(f, "{code} unknown command\r\n"),
            SyntaxErrorIllegalParameters => {
                write!(f, "{code} syntax error, illegal parameters\r\n")
            }

            CommandNotImplemented => write!(f, "{code} command not implemented\r\n"),
            CommandParameterNotImplemented => {
                write!(f, "{code} command parameter not implemented\r\n")
            }
            AccessDeniedIPBlocked => write!(f, "{} access denied\r\n", code),
            AccessDenied => write!(
                f,
                "{code} access denied, use \"SHOW INFO\" for server information\r\n"
            ),
            SASLUnknownMechanism => write!(f, "{code} access denied, unknown mechanism\r\n"),
            InvalidDatabase => write!(
                f,
                "{code} invalid database, use \"SHOW DB\" for list of databases\r\n"
            ),
            InvalidStrategy => write!(
                f,
                "{code} invalid strategy, use \"SHOW STRAT\" for a list of strategies\r\n"
            ),
            NoMatch => write!(f, "{code} no match\r\n"),
            NoDatabasesPresent => write!(f, "{code} no databases present\r\n"),
            NoStrategiesAvailable => write!(f, "{code} no strategies available\r\n"),
        }
    }
}

#[derive(Debug)]
pub enum DatabaseLookupStrategy {
    Named(String), // specific database
    First,         // '!'
    All,           // '*
}

/// Unsupported variants include: Substring, Suffix, Regex, Soundex, Levenshtein
#[derive(Debug, Default, Clone, Copy)]
pub enum SearchStrategy {
    #[default]
    /// '.'
    Exact,
    Prefix,
}

impl SearchStrategy {
    /// TODO: It would be nice to be able to do this automatically with reflection at compile time
    pub const VARIANTS: [SearchStrategy; 2] = [SearchStrategy::Exact, SearchStrategy::Prefix];
}

impl Display for SearchStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchStrategy::Exact => write!(f, "exact \"Match headwords exactly\""),
            SearchStrategy::Prefix => write!(f, "prefix \"Match prefixes\""),
        }
    }
}
