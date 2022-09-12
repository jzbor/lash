use nom::{
    branch::*,
    character::complete::*,
    bytes::complete::*,
    combinator,
    IResult
};

use crate::state::*;


#[derive(Debug,Clone)]
pub enum Command {
    Echo(String),
    History(),
    Info(),
}

impl Command {
    pub fn execute(&self, state: &State) -> String {
        match self {
            Command::Echo(s) => format!(" {} ", s),
            Command::History() => history(state),
            Command::Info() => info(state),
        }
    }
}

pub fn history(state: &State) -> String {
    if state.history.is_empty() {
        return format!("no history entry found");
    } else {
        let items: Vec<String> = state.history.iter()
            .map(|e| e.to_string()).collect();
        return items.join("\n");
    }
}

pub fn info(state: &State) -> String {
    let option = state.last_lambda();
    match option {
        Some(e) => format!(
"input: {}
output: {}
beta reductions: {}
builtin substitutions: {}",
                           e.input, e.output, e.nbeta, e.bi_subs),
        None => format!("no history entry found"),
    }
}

pub fn match_command(s: &str) -> IResult<&str, Command> {
    let (rest, _) = char(':')(s)?;
    let (rest, _) = space0(rest)?;
    let command_matchers = (
        match_echo,
        match_hist,
        match_info,
    );
    return alt(command_matchers)(rest);
}

fn match_echo(s: &str) -> IResult<&str, Command> {
    let (rest, _) = tag("echo")(s)?;
    let (rest, _) = space1(rest)?;
    let (rest, output) = combinator::rest(rest)?;
    return Ok((rest, Command::Echo(output.to_owned())));
}

fn match_hist(s: &str) -> IResult<&str, Command> {
    let (rest, _) = tag("hist")(s)?;
    let (rest, _) = space0(rest)?;
    return Ok((rest, Command::History()));
}

fn match_info(s: &str) -> IResult<&str, Command> {
    let (rest, _) = tag("info")(s)?;
    let (rest, _) = space0(rest)?;
    return Ok((rest, Command::Info()));
}
