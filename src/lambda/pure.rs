use nom::{
    branch::*,
    character::complete::*,
};

use crate::lambda::*;
use crate::parsing::*;

// @TODO this parser does not seem to handle applications well (try "(asdf jkl)")
//
// Syntax:
// lambda      := variable | abstraction | application
// abstraction := (\variable-name . lambda)
// application := (lambda lambda)

pub fn match_lambda(s: Span) -> IResult<LambdaNode> {
    return alt((match_variable, match_abstraction, match_application))(s);
}

fn match_variable(s: Span) -> IResult<LambdaNode> {
    let (rest, name) = match_variable_name(s)?;
    return Ok((rest, LambdaNode::Variable(name.to_owned())));
}

fn match_abstraction(s: Span) -> IResult<LambdaNode> {
    let (rest, _) = char('(')(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = match_lambda_sign(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, variable_name) = match_variable_name(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char('.')(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, term) = match_lambda(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char(')')(rest)?;
    return Ok((rest, LambdaNode::Abstraction(variable_name.to_owned(), Box::new(term))));
}

fn match_application(s: Span) -> IResult<LambdaNode> {
    let (rest, _) = char('(')(s)?;
    let (rest, _) = space0(rest)?;
    let (rest, term1) = match_lambda(rest)?;
    let (rest, _) = space1(rest)?;
    let (rest, term2) = match_lambda(rest)?;
    let (rest, _) = space0(rest)?;
    let (rest, _) = char(')')(rest)?;
    return Ok((rest, LambdaNode::Application(Box::new(term1), Box::new(term2))));
}
