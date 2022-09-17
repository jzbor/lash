use std::collections::VecDeque;
use nom;
use nom::{
    branch::*,
    bytes::complete::*,
    character::*,
    character::complete::*,
    combinator::*,
    multi::*,
};
use nom_locate::LocatedSpan;

use crate::lambda::*;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;


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

pub fn match_complete_lambda(s: Span) -> IResult<LambdaNode> {
    return match_lambda(s)
            .conclude(|r| format!("unable to parse lambda expression ('{}')", r));
}

// @TODO remove pub
pub fn match_lambda_sign(s: Span) -> IResult<Span> {
    return recognize(char('\\'))(s);
}

pub fn match_variable_name<'a>(s: Span<'a>) -> IResult<&'a str> {
    let (rest, name) = take_while1(|x| is_alphanumeric(x as u8) || x == '-' || x == '_' || x == '\'')(s)?;
    return Ok((rest, *name));
}

pub fn with_err<'a, O>(result: IResult<'a, O>, s: Span<'a>, msg: String) -> IResult<'a, O> {
    return result.map_err(|_| nom::Err::Error(ParseError::new(msg, s)));
}

fn match_abstraction(s: Span) -> IResult<LambdaNode> {
    let (rest, _) = match_lambda_sign(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, mut variables) = map(match_variable_list, |v| VecDeque::from(v))(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = with_err(char('.')(rest), rest,
                             "expected '.' after abstraction variables".to_owned())?;
    let (rest, _) = space0(rest)?;
    let (rest, inner) = with_err(match_lambda(rest), rest,
                             "invalid or missing inner term on abstraction".to_owned())?;

    let mut current_abstraction = LambdaNode::Abstraction(variables.pop_back().unwrap()
                                                          .to_owned(), Box::new(inner));
    while let Some(variable) = variables.pop_back() {
        current_abstraction = LambdaNode::Abstraction(variable.to_owned(), Box::new(current_abstraction));
    }

    return Ok((rest, current_abstraction));
}

fn match_application(s: Span) -> IResult<LambdaNode> {
    let (rest, terms) = separated_list1(space1, match_group)(s)?;
    let node = vec_to_application(terms);
    return Ok((rest, node));
}

fn match_bracketed(s: Span) -> IResult<LambdaNode> {
    let (rest, _) = char('(')(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, lambda) = match_lambda(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char(')')(rest)?;

    return Ok((rest, lambda));
}

fn match_numeral(s: Span) -> IResult<LambdaNode> {
    let (rest, _) = char('$')(s)?;
    let (rest, n) = match_u32(rest)?;
    return Ok((rest, LambdaNode::church_numeral(n)));
}

fn match_group(s: Span) -> IResult<LambdaNode> {
    return alt((match_variable, match_bracketed, match_numeral))(s);
}

fn match_lambda(s: Span) -> IResult<LambdaNode> {
    return alt((match_abstraction, match_application))(s);
}

fn match_u32(s: Span) -> IResult<u32> {
    let (rest, digits) = recognize(digit1)(s)?;
    let uint = match str::parse(*digits) {
        Ok(ui) => ui,
        Err(_) => return Err(nom::Err::Error(ParseError::new("unable to parse number".to_owned(), s))),
    };
    return Ok((rest, uint));
}

fn match_variable(s: Span) -> IResult<LambdaNode> {
    let (rest, name) = match_variable_name(s)?;
    return Ok((rest, LambdaNode::Variable(name.to_owned())));
}

fn match_variable_list<'a>(s: Span<'a>) -> IResult<Vec<&'a str>> {
    let mut rest = s;
    let mut variables = Vec::new();

    if let Ok((r, name)) = match_variable_name(rest) {
        variables.push(name);
        rest = r
    } else {
        // @TODO
        return Err(nom::Err::Error(
                ParseError::new("variables missing for lambda abstraction".to_owned(), rest)));
    }

    loop {
        (rest, _) = space0(rest)?;
        if let Ok((r, name)) = match_variable_name(rest) {
            variables.push(name);
            rest = r;
        } else {
            break;
        }
    }

    return Ok((rest, variables));
}

fn vec_to_application(mut terms: Vec<LambdaNode>) -> LambdaNode {
    if terms.len() < 1 {
        panic!("Invalid number of input terms for application");
    } else if terms.len() == 1 {
        return terms.pop().unwrap();
    } else {
        let right = terms.pop().unwrap();
        let left = vec_to_application(terms);
        return LambdaNode::Application(Box::new(left), Box::new(right));
    }
}

