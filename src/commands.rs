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
    Steps(),
}

impl Command {
    pub fn execute(&self, state: &State) -> String {
        match self {
            Command::Echo(s) => format!(" {} ", s),
            Command::History() => history(state),
            Command::Info() => info(state),
            Command::Steps() => steps(state),
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
"beta reductions: {}
builtin substitutions: {}
variable substitutions: {}",
                           e.nbeta, e.bi_subs, e. var_subs),
        None => format!("no history entry found"),
    }
}

pub fn steps(state: &State) -> String {
    if let Some(hist_entry) = state.last_lambda() {
        if let LineType::Lambda(initial_tree) = &hist_entry.parsed {
            let mut lines = Vec::new();
            let mut tree = initial_tree.clone();
            lines.push(hist_entry.input.clone());
            lines.push(format!("   = {}", tree.to_string()));
            loop {
                let t = tree.reduce(state.config.strategy);
                if t == tree {
                    break;
                } else {
                    lines.push(format!("  -> {}", t.to_string()));
                    tree = t;
                }
            }
            return lines.join("\n");
        } else {
            panic!("last_lambda() didn't return lambda entry");
        }
    } else {
        return String::from("no history entry found");
    }
}

pub fn match_command(s: &str) -> IResult<&str, Command> {
    let (rest, _) = char(':')(s)?;
    let (rest, _) = space0(rest)?;
    let command_matchers = (
        match_echo,
        argless_matcher("hist", Command::History()),
        argless_matcher("info", Command::Info()),
        argless_matcher("steps", Command::Steps()),
    );
    return alt(command_matchers)(rest);
}

fn match_echo(s: &str) -> IResult<&str, Command> {
    let (rest, _) = tag("echo")(s)?;
    let (rest, _) = space1(rest)?;
    let (rest, output) = combinator::rest(rest)?;
    return Ok((rest, Command::Echo(output.to_owned())));
}

fn argless_matcher(keyword: &str, command: Command) -> impl FnMut(&str) -> IResult<&str, Command> {
    let owned_keyword = keyword.to_owned();
    return move |s: &str| {
        let (rest, _) = tag(owned_keyword.as_str())(s)?;
        let (rest, _) = space0(rest)?;
        return Ok((rest, command.clone()));
    };
}
