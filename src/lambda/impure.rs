use std::collections::VecDeque;
use nom::{
    branch::*,
    multi::*,
    character::complete::*,
};

use crate::lambda::*;
use crate::parsing::*;

pub fn match_lambda(s: Span) -> IResult<LambdaNode> {
    return alt((match_abstraction, match_application))(s);
}

fn match_abstraction(s: Span) -> IResult<LambdaNode> {
    let (rest, _) = match_lambda_sign(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, mut variables) = map(match_variable_list, |v| VecDeque::from(v))(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char('.')(rest)?;
    let match_inner = |s| {
        let (rest, _) = space0(rest)?;
        return match_lambda(rest);
    };
    let (rest, inner) = with_err(match_inner(rest), rest,
                             "missing inner term on abstraction".to_owned())?;

    let mut current_abstraction = LambdaNode::Abstraction(variables.pop_back().unwrap()
                                                          .to_owned(), Box::new(inner));
    while let Some(variable) = variables.pop_back() {
        current_abstraction = LambdaNode::Abstraction(variable.to_owned(), Box::new(current_abstraction));
    }

    return Ok((rest, current_abstraction));
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

fn match_application(s: Span) -> IResult<LambdaNode> {
    let (rest, terms) = separated_list1(space1, match_group)(s)?;
    let node = vec_to_application(terms);
    return Ok((rest, node));
}

fn match_group(s: Span) -> IResult<LambdaNode> {
    return alt((match_variable, match_bracketed))(s);
}

fn match_bracketed(s: Span) -> IResult<LambdaNode> {
    let (rest, _) = char('(')(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, lambda) = match_lambda(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char(')')(rest)?;

    return Ok((rest, lambda));
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
