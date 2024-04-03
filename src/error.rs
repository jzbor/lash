use std::{fmt::Display, path::PathBuf};

use crate::{parsing, r#macro::Macro};

pub type LashResult<T> = Result<T, LashError>;

#[derive(Debug,Clone)]
pub struct LashError {
    error_type: LashErrorType,
    message: String,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug,Clone)]
pub enum LashErrorType {
    ChurchNumError,
    FileError,
    MacroArgError,
    SetKeyError,
    SetValueError,
    SyntaxError,
}

impl LashError {
    pub fn new_church_num_error() -> Self {
        LashError {
            error_type: LashErrorType::ChurchNumError,
            message: "church numerals disabled".to_owned(),
        }
    }

    pub fn new_file_error(file: PathBuf, error: Option<std::io::Error>) -> Self {
        let error_msg = match error {
            Some(e) => format!("({})", e),
            None => String::new(),
        };
        LashError {
            error_type: LashErrorType::FileError,
            message: format!("unable to open file '{}' {}", file.to_string_lossy(), error_msg),
        }
    }

    pub fn new_macro_arg_error(m: Macro, args_given: usize, args_expected: usize) -> Self {
        LashError {
            error_type: LashErrorType::MacroArgError,
            message: format!("macro {} expects {} arguments, but {} were given", m, args_given, args_expected),
        }
    }

    pub fn new_set_key_error(key: &str) -> Self {
        LashError {
            error_type: LashErrorType::SetKeyError,
            message: format!("unknown key '{}'", key),
        }
    }

    pub fn new_set_value_error(value: &str) -> Self {
        LashError {
            error_type: LashErrorType::SetValueError,
            message: format!("unknown value '{}'", value),
        }
    }

    pub fn resolve(&self) {
        eprintln!("{}", self);
        std::process::exit(1);
    }
}

impl Display for LashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LashErrorType::*;
        let prefix = match self.error_type {
            ChurchNumError => "Church Numeral Error",
            FileError => "File Error",
            MacroArgError => "Macro Argument Error",
            SetKeyError => "Set Key Error",
            SetValueError => "Set Value Error",
            SyntaxError => "Syntax Error",
        };
        write!(f, "{}: {}", prefix, self.message)
    }
}

impl From<nom::Err<parsing::ParseError<'_>>> for LashError {
    fn from(value: nom::Err<parsing::ParseError<'_>>) -> Self {
        use nom::Err::*;
        let message = match value {
            Incomplete(_) => "incomplete data".to_owned(),
            Error(e) => format!("{}", e),
            Failure(e) => format!("{}", e),
        };
        LashError {
            error_type: LashErrorType::SyntaxError,
            message,
        }
    }
}
