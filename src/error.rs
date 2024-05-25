extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::Display;

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
    FormatError,
    MacroArgError,
    SetKeyError,
    SetValueError,
    SyntaxError,
    #[cfg(not(feature = "std"))]
    NotFoundError,
    #[cfg(not(feature = "std"))]
    NotSupportedError,
}

impl LashError {
    pub fn new_church_num_error() -> Self {
        LashError {
            error_type: LashErrorType::ChurchNumError,
            message: "church numerals disabled".to_owned(),
        }
    }

    #[cfg(feature = "std")]
    pub fn new_file_error(file: std::path::PathBuf, error: Option<std::io::Error>) -> Self {
        let error_msg = match error {
            Some(e) => format!("({})", e),
            None => String::new(),
        };
        LashError {
            error_type: LashErrorType::FileError,
            message: format!("unable to open file '{}' {}", file.to_string_lossy(), error_msg),
        }
    }

    #[cfg(not(feature = "std"))]
    pub fn new_not_found_error(file: &str) -> Self {
        LashError {
            error_type: LashErrorType::NotFoundError,
            message: format!("{}", file),
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

    #[cfg(not(feature = "std"))]
    pub fn new_not_supported_error(message: String) -> Self {
        LashError {
            error_type: LashErrorType::NotSupportedError,
            message,
        }
    }


    pub fn resolve(&self) {
        #[cfg(feature = "std")]
        {
            eprintln!("{}", self);
            std::process::exit(1);
        }
        #[cfg(not(feature = "std"))]
        Err(self).unwrap()
    }
}

impl Display for LashError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use LashErrorType::*;
        let prefix = match self.error_type {
            ChurchNumError => "Church Numeral Error",
            FileError => "File Error",
            FormatError => "Format Error",
            MacroArgError => "Macro Argument Error",
            SetKeyError => "Set Key Error",
            SetValueError => "Set Value Error",
            SyntaxError => "Syntax Error",
            #[cfg(not(feature = "std"))]
            NotFoundError => "Not Found",
            #[cfg(not(feature = "std"))]
            NotSupportedError => "Not supported",
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

impl From<core::fmt::Error> for LashError {
    fn from(value: core::fmt::Error) -> Self {
        LashError {
            error_type: LashErrorType::FormatError,
            message: value.to_string(),
        }
    }
}
