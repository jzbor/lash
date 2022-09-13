use nom;
use nom::{
    combinator::*,
    character::complete::*,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;

pub fn with_err<'a, O>(result: IResult<'a, O>, s: Span<'a>, msg: String) -> IResult<'a, O> {
    return result.map_err(|_| nom::Err::Error(ParseError::new(msg, s)));
}


#[derive(Debug, PartialEq)]
pub struct ParseError<'a> {
    span: Span<'a>,
    message: String,
}

pub trait Conclude<'a, O> {
    fn conclude(self, msg: fn(&str) -> String) -> IResult<'a, O>;
}

impl<'a> ParseError<'a> {
    pub fn new(message: String, span: Span<'a>) -> Self {
        Self { span, message: message }
    }

    pub fn line(&self) -> u32 {
        return self.span().location_line();
    }

    pub fn offset(&self) -> usize {
        return self.span().location_offset();
    }

    pub fn span(&self) -> &Span {
        return &self.span;
    }

    pub fn message(&self) -> String {
        return self.message.clone();
    }
}

impl<'a, O> Conclude<'a, O> for IResult<'a, O> {
    fn conclude(self, msg: fn(&str) -> String) -> IResult<'a, O> {
        match self {
            IResult::Ok((rest, result)) => {
                let (rest, _) = space0(rest)?;

                if eof::<&str, ()>(*rest).is_ok() {
                    return Ok((rest, result));
                } else {
                    return Err(nom::Err::Error(ParseError::new(msg(*rest), rest)));
                }
            },
            error => error,
        }
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, _kind: nom::error::ErrorKind) -> Self {
        Self::new("unknown error".to_owned(), input)
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), input)
    }

    fn or(self, other: Self) -> Self {
        let chosen = if self.line() == other.line() {
            if self.offset() > other.offset() {
                self
            } else {
                other
            }
        } else if self.line() > other.line() {
            self
        } else {
            other
        };
        return chosen;
    }
}
