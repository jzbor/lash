use std::fmt::Display;

use crate::parsing;

pub type LashResult<T> = Result<T, LashError>;

pub struct LashError {
    error_type: LashErrorType,
    message: String,
}

pub enum LashErrorType {
    SyntaxError,
}


impl LashError {
    pub fn resolve(&self) {
        eprintln!("{}", self);
        std::process::exit(1);
    }
}

impl Display for LashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LashErrorType::*;
        let prefix = match self.error_type {
            SyntaxError => "Syntax Error",
        };
        write!(f, "{}: {}", prefix, self.message)
    }
}

impl From<nom::Err<parsing::ParseError<'_>>> for LashError {
    fn from(value: nom::Err<parsing::ParseError<'_>>) -> Self {
        use nom::Err::*;
        let message = match value {
            Incomplete(_) => format!("incomplete data"),
            Error(e) => format!("{}", e),
            Failure(e) => format!("{}", e),
        };
        LashError {
            error_type: LashErrorType::SyntaxError,
            message,
        }
    }
}
