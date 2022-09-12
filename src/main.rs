use std::io::*;
use nom::{
    branch::*,
    combinator::*,
    character::complete::*,
    IResult,
};

use lambda::*;
use commands::*;
use state::*;

mod lambda;
mod commands;
mod state;
mod builtins;

#[derive(Debug,Clone)]
pub enum LineType {
    Command(Command),
    Lambda(LambdaNode),
    Error(String),
    EOF(),
    Nop(),
}


fn handle_command(state: &State, input: String, command: Command) -> HistoryEntry {
    let output = command.execute(state);
    println!("{}", output);
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    hist_entry.parsed = LineType::Command(command);
    hist_entry.output = output;
    return hist_entry;
}

fn handle_lambda(state: &State, input: String, tree: LambdaNode) -> HistoryEntry {
    let variable_substitutions = 0;
    let (tree, builtin_substitutions) = tree.with_vars(&state.builtins, state.config.parser);
    let (normal, nbeta) = tree.normalize(ReductionStrategy::Normal);
    println!("{}", normal.to_string());

    return HistoryEntry {
        input: input,
        parsed: LineType::Lambda(tree),
        output: normal.to_string(),
        nbeta: nbeta,
        var_subs: variable_substitutions,
        bi_subs: builtin_substitutions,
    };
}

fn handle_error(_state: &State, input: String, msg: &str) -> HistoryEntry {
    let output = format!("An error occured: {}", msg);
    println!("{}", output);
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    hist_entry.parsed = LineType::Error(msg.to_string());
    hist_entry.output = output;
    return hist_entry;
}

fn match_nop(s: &str) -> IResult<&str, LineType> {
    return space0(s).map(|(rest, _)| (rest, LineType::Nop()));
}

fn match_eof(s: &str) -> IResult<&str, LineType> {
    return eof(s).map(|(rest, _)| (rest, LineType::EOF()));
}


fn match_wrapper(config: Config, s: &str) -> IResult<&str, LineType> {
    let to_command = |c| LineType::Command(c);
    let to_lambda = |l| LineType::Lambda(l);

    return alt((map(lambda_matcher(config.parser), to_lambda),
                map(match_command, to_command),
                match_nop, match_eof))(s);
}

fn parse_line(input: String, state: &mut State) -> bool {
    let parsed_option = match_wrapper(state.config, &input);

    let hist_entry = if let Ok((rest, parsed)) = parsed_option {
        if rest == "" {
            match parsed {
                LineType::Command(c) => handle_command(state, input, c),
                LineType::EOF() => return true,
                LineType::Error(e) => handle_error(state, input, &e),
                LineType::Lambda(t) => handle_lambda(state, input, t),
                LineType::Nop() => return false,
            }
        } else {
            let msg = "Unable to parse (unparsed input remaining)";
            handle_error(state, input, msg)
        }
    } else {
        let msg = "Unable to parse (parser returned error)";
        handle_error(state, input, msg)
    };

    state.history.push(hist_entry);
    return false;
}

fn print_prompt() {
    print!("\n-> ");
    stdout().flush().unwrap();
}

// @TODO test "asdf jkl" on pure parser

fn main() {
    // lambda_info("(((\\x . (\\y . x)) x) ((\\x . (x x)) (\\x . (x x))))");
    let mut state = State::init();
    let mut input = String::new();
    print_prompt();

    while stdin().read_line(&mut input).unwrap() != 0 {
        input = input.trim().to_string();
        parse_line(input, &mut state);
        print_prompt();
        input = String::new();
    }
    println!();
}
