use rustyline::{
    Editor,
    error::*,
};
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

fn handle_assignment(state: &mut State, input: String, name: String, value: String) -> HistoryEntry {
    state.add_variable(name, value);
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    return hist_entry;
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

fn replace_builtins_and_variables(state: &State, tree: &LambdaNode) -> (LambdaNode, u32, u32) {
    let mut variable_substitutions = 0;
    let variables_borrowed = state.variables.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let mut new_tree = tree.clone();
    loop {
        let (t, vs) = new_tree.with_vars(&variables_borrowed, state.config.parser);
        if vs == 0 {
            break;
        }
        new_tree = t;
        variable_substitutions += vs;
    }

    let (new_tree, builtin_substitutions) = new_tree.with_vars(&state.builtins, state.config.parser);

    return (new_tree, builtin_substitutions, variable_substitutions);
}

fn handle_lambda(state: &State, input: String, tree: LambdaNode) -> HistoryEntry {
    let (tree, bsubs, vsubs) = replace_builtins_and_variables(state, &tree);
    let (normal, nbeta) = tree.normalize(ReductionStrategy::Normal);
    println!("{}", normal.to_string());

    return HistoryEntry {
        input: input,
        parsed: LineType::Lambda(tree),
        output: normal.to_string(),
        nbeta: nbeta,
        var_subs: vsubs,
        bi_subs: bsubs,
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
    let to_assignment = |(k, v)| LineType::Assignment(k, v);

    return alt((map(assignment_matcher(config.parser), to_assignment),
                map(lambda_matcher(config.parser), to_lambda),
                map(match_command, to_command),
                match_nop, match_eof))(s);
}

fn parse_line(input: String, state: &mut State) -> bool {
    let parsed_option = match_wrapper(state.config, &input);

    let hist_entry = if let Ok((rest, parsed)) = parsed_option {
        if rest == "" {
            match parsed {
                LineType::Assignment(k, v) => handle_assignment(state, input, k, v),
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

fn main() {
    // lambda_info("(((\\x . (\\y . x)) x) ((\\x . (x x)) (\\x . (x x))))");
    let mut state = State::init();
    let mut rl = Editor::<()>::new().unwrap();
    loop {
        let line = rl.readline("[λ] ");
        match line {
            Ok(input) => {
                rl.add_history_entry(input.as_str());
                parse_line(input, &mut state);
                println!();
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    println!();
}
