use std::collections::VecDeque;
use nom::{
    branch::*,
    multi::*,
    character::complete::*,
    IResult,
};

use crate::lambda::*;

pub fn match_lambda(s: &str) -> IResult<&str, LambdaNode> {
    return alt((match_abstraction, match_application))(s);
}

fn match_abstraction(s: &str) -> IResult<&str, LambdaNode> {
    let (rest, _) = match_lambda_sign(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, mut variables) = map(match_variable_list, |v| VecDeque::from(v))(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char('.')(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, inner) = match_lambda(rest)?;

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

fn match_application(s: &str) -> IResult<&str, LambdaNode> {
    let (rest, terms) = separated_list1(space1, match_group)(s)?;
    let node = vec_to_application(terms);
    return Ok((rest, node));
}

fn match_group(s: &str) -> IResult<&str, LambdaNode> {
    return alt((match_variable, match_bracketed))(s);
}

fn match_bracketed(s: &str) -> IResult<&str, LambdaNode> {
    let (rest, _) = char('(')(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, lambda) = match_lambda(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char(')')(rest)?;

    return Ok((rest, lambda));
}

fn match_variable(s: &str) -> IResult<&str, LambdaNode> {
    let (rest, name) = match_variable_name(s)?;
    return Ok((rest, LambdaNode::Variable(name.to_owned())));
}

fn match_variable_list(s: &str) -> IResult<&str, Vec<&str>> {
    let mut rest = s;
    let mut variables = Vec::new();

    if let Ok((r, name)) = match_variable_name(rest) {
        variables.push(name);
        rest = r
    } else {
        // @TODO
        panic!("Error handling for missing variables not implemented");
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
