use std::{collections::VecDeque, fmt::Display};
use nom::{
    branch::*,
    bytes::complete::*,
    character::*,
    character::complete::*,
    combinator::*,
    multi::*,
};
use nom_locate::LocatedSpan;

use crate::{lambda::*, r#macro::Macro};


pub type Span<'a> = LocatedSpan<&'a str>;
pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;

#[derive(Debug, PartialEq)]
pub struct ParseError<'a> {
    span: Span<'a>,
    message: String,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(String, LambdaTree),
    Lambda(LambdaTree),
}


impl<'a> ParseError<'a> {
    pub fn new(message: String, span: Span<'a>) -> Self {
        Self { span, message }
    }

    pub fn line(&self) -> u32 {
        return self.span().location_line();
    }

    pub fn offset(&self) -> usize {
        return self.span().location_offset();
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn message(&self) -> String {
        self.message.clone()
    }
}


impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(kind.description().to_owned(), input)
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        // TODO: build stack trace
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("expected character '{}'", c), input)
    }

    fn or(self, other: Self) -> Self {
        if self.line() == other.line() {
            if self.offset() > other.offset() {
                self
            } else {
                other
            }
        } else if self.line() > other.line() {
            self
        } else {
            other
        }
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (line {})" , self.message, self.line())
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Statement::*;
        match self {
            Assignment(name, term) => write!(f, "{} := {}", name, term),
            Lambda(term) => write!(f, "{}", term),
        }
    }
}


fn match_abstraction(s: Span) -> IResult<LambdaTree> {
    let (rest, _) = match_lambda_sign(s)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, mut variables) = map(match_variable_list, VecDeque::from)(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = with_err(char('.')(rest), rest,
                             "expected '.' after abstraction variables".to_owned())?;
    let (rest, _) = multispace0(rest)?;
    let (rest, inner) = with_err(match_lambda(rest), rest,
                             "invalid or missing inner term on abstraction".to_owned())?;

    let mut current_abstraction = LambdaTree::new_abstraction(variables.pop_back().unwrap().to_owned(), inner);
    while let Some(variable) = variables.pop_back() {
        current_abstraction = LambdaTree::new_abstraction(variable.to_owned(), current_abstraction);
    }

    Ok((rest, current_abstraction))
}

fn match_application(s: Span) -> IResult<LambdaTree> {
    let (rest, terms) = separated_list1(multispace1, match_group)(s)?;
    let node = vec_to_application(terms);
    Ok((rest, node))
}

fn match_assignment(s: Span) -> IResult<(String, LambdaTree)> {
    let (rest, name) = match_variable_name(s)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = tag(":=")(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, lambda) = match_lambda(rest)?;
    Ok((rest, (name.to_owned(), lambda)))
}

fn match_bracketed(s: Span) -> IResult<LambdaTree> {
    let (rest, _) = char('(')(s)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, lambda) = match_lambda(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char(')')(rest)?;

    Ok((rest, lambda))
}

fn match_group(s: Span) -> IResult<LambdaTree> {
    alt((match_variable, match_bracketed))(s)
}

pub fn match_lambda(s: Span) -> IResult<LambdaTree> {
    alt((match_macro, match_abstraction, match_application))(s)
}

fn match_lambda_sign(s: Span) -> IResult<Span> {
    recognize(alt((char('\\'), char('λ'))))(s)
}

fn match_macro(s: Span) -> IResult<LambdaTree> {
    let (rest, _) = char('!')(s)?;
    let (rest, macro_name) = alphanumeric1(rest)?;

    let m = match Macro::get(&macro_name) {
        Some(m) => m,
        None => return Err(nom::Err::Error(ParseError::new(format!("unknown macro '{}'", macro_name), rest))),
    };

    let (rest, args) = opt(match_macro_args)(rest)?;
    let lambdas = match args {
        Some(lambdas) => lambdas,
        None => Vec::new(),
    };

    Ok((rest, LambdaTree::new_macro(m, lambdas)))
}

fn match_macro_args(s: Span) -> IResult<Vec<LambdaTree>> {
    let (rest, _) = multispace1(s)?;
    let (rest, lambdas) = separated_list1(multispace1, match_group)(rest)?;
    let (rest, _) = multispace0(rest)?;
    Ok((rest, lambdas))
}

pub fn match_statement(s: Span, with_semicolon: bool) -> IResult<Statement> {
    let (rest, statement) = alt((|x| match_assignment(x).map(|(r, (n, l))| (r, Statement::Assignment(n, l))),
                                 |x| match_lambda(x).map(|(r, l)| (r, Statement::Lambda(l)))))(s)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = if with_semicolon {
        (char(';')(rest)?.0, 0)
    } else {
        (opt(char(';'))(rest)?.0, 0)
    };
    let (rest, _) = multispace0(rest)?;
    Ok((rest, statement))
}

pub fn match_statements(s: Span) -> IResult<Vec<Statement>> {
    let mut rest = s;
    let mut statements = Vec::new();

    loop {
        let (r, statement) = match_statement(rest, true)?;
        rest = r;
        statements.push(statement);
        if rest.is_empty() {
            break;
        }
    }

    if eof::<&str, ()>(*rest).is_ok() {
        Ok((rest, statements))
    } else {
        return Err(nom::Err::Error(ParseError::new("expected end of file".to_owned(), rest)));
    }
}

pub fn match_variable(s: Span) -> IResult<LambdaTree> {
    let (rest, name) = match_variable_name(s)?;
    Ok((rest, LambdaTree::new_variable(name.to_owned())))
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
        (rest, _) = multispace0(rest)?;
        if let Ok((r, name)) = match_variable_name(rest) {
            variables.push(name);
            rest = r;
        } else {
            break;
        }
    }

    Ok((rest, variables))
}
pub fn match_variable_name<'a>(s: Span<'a>) -> IResult<&'a str> {
    let (rest, name) = take_while1(|x| is_alphanumeric(x as u8) || x == '-' || x == '_' || x == '\'')(s)?;
    Ok((rest, *name))
}

fn vec_to_application(mut terms: Vec<LambdaTree>) -> LambdaTree {
    if terms.is_empty() {
        panic!("Invalid number of input terms for application");
    } else if terms.len() == 1 {
        return terms.pop().unwrap();
    } else {
        let right = terms.pop().unwrap();
        let left = vec_to_application(terms);
        return LambdaTree::new_application(left, right);
    }
}

pub fn with_err<'a, O>(result: IResult<'a, O>, s: Span<'a>, msg: String) -> IResult<'a, O> {
    result.map_err(|_| nom::Err::Error(ParseError::new(msg, s)))
}

