extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Display;
use core::str::FromStr;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::error::{LashError, LashResult};
use crate::interpreter::InterpreterDirective;
use crate::lambda::LambdaTree;
use crate::r#macro::Macro;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct LambdaParser;

#[derive(Debug, Clone)]
pub enum Statement {
    Assignment(String, LambdaTree),
    Directive(InterpreterDirective),
    Lambda(LambdaTree),
}


impl Display for Statement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Statement::*;
        match self {
            Assignment(name, term) => write!(f, "{} := {}", name, term),
            Lambda(term) => term.fmt(f),
            Directive(directive) => directive.fmt(f),
        }
    }
}


fn parse_lambda(pair: Pair<Rule>) -> LashResult<LambdaTree> {
    use Rule::*;
    match pair.as_rule() {
        lambda => parse_lambda(pair.into_inner().next().unwrap()),
        abstraction => {
            let mut variables = Vec::new();
            let mut child = None;
            for sub_pair in pair.into_inner() {
                if sub_pair.as_rule() == variable {
                    variables.push(sub_pair.as_span().as_str().to_string());
                } else {
                    child = Some(parse_lambda(sub_pair)?);
                    break
                }
            }

            let mut current = child.unwrap();
            for var in variables.into_iter().rev() {
                current = LambdaTree::new_abstraction(var, current);
            }
            Ok(current)
        },
        application => {
            let mut children: Vec<_> = pair.into_inner()
                .map(|ip| parse_lambda(ip))
                .collect();
            if children.len() == 1 {
                Ok(children.pop().unwrap()?)
            } else {
                let mut iter = children.into_iter();
                let mut current = iter.next().unwrap()?;
                for child in iter {
                    current = LambdaTree::new_application(current, child?);
                }
                Ok(current)
            }
        },
        group => parse_lambda(pair.into_inner().next().unwrap()),
        variable => Ok(LambdaTree::new_variable(pair.as_span().as_str().to_string())),
        church => Ok(LambdaTree::new_church_num(pair.as_span().as_str()[1..].parse::<u32>().unwrap())),
        r#macro => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_span().as_str().to_string();
            let mut children = Vec::new();
            for child in inner.map(|ip| parse_lambda(ip)) {
                children.push(child?);
            }
            let m = Macro::from_str(&name)
                .map_err(|_| LashError::new_unknown_macro_error(&name))?;
            Ok(LambdaTree::new_macro(m, children))
        },
        _ => panic!("Unexpected token '{:?}'", pair.as_rule()),
    }
}

fn parse_single_statement(pair: Pair<Rule>) -> LashResult<Statement> {
    use Rule::*;
    match pair.as_rule() {
        statement => parse_single_statement(pair.into_inner().next().unwrap()),
        assignment => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str();
            let term = parse_lambda(inner.next().unwrap())?;
            Ok(Statement::Assignment(name.to_string(), term))
        },
        lambda => Ok(Statement::Lambda(parse_lambda(pair.into_inner().next().unwrap())?)),
        directive => {
            let mut inner = pair.into_inner();
            let dir_pair = inner.next().unwrap();
            match dir_pair.as_rule() {
                directive_usestd => Ok(Statement::Directive(InterpreterDirective::UseStd)),
                directive_set => {
                    let mut inner = dir_pair.into_inner();
                    let k = inner.next().unwrap().as_span().as_str().to_string();
                    let v = inner.next().unwrap().as_span().as_str().to_string();
                    Ok(Statement::Directive(InterpreterDirective::Set(k, v)))
                },
                directive_echo => {
                    let mut inner = dir_pair.into_inner();
                    let msg = inner.next().unwrap().as_span().as_str().to_string();
                    Ok(Statement::Directive(InterpreterDirective::Echo(msg)))
                }
                directive_include => {
                    let mut inner = dir_pair.into_inner();
                    let path = inner.next().unwrap().as_span().as_str().to_string();
                    Ok(Statement::Directive(InterpreterDirective::Include(path)))
                }
                _ => unreachable!(),
            }
        },
        _ => panic!("Unexpected token '{:?}'", pair.as_rule()),
    }
}

fn parse_multi_statement(pair: Pair<Rule>) -> LashResult<Vec<Statement>> {
    let mut statements = Vec::new();
    for result in pair.into_inner()
            .filter(|ip| ip.as_rule() != Rule::EOI)
            .map(|ip| parse_single_statement(ip)) {
        statements.push(result?);
    }
    Ok(statements)
}

pub fn parse_statement(input: &str) -> LashResult<Statement> {
    let parsed = LambdaParser::parse(Rule::statement, input)
        .map_err(|e| LashError::new_syntax_error(e))?
        .next().unwrap();
    parse_single_statement(parsed)
}

pub fn parse_statements(input: &str) -> LashResult<Vec<Statement>> {
    let parsed = LambdaParser::parse(Rule::statements, input)
        .map_err(|e| LashError::new_syntax_error(e))?
        .next().unwrap();
    parse_multi_statement(parsed)
}
