use std::fs::File;
use std::io::{BufRead, BufReader};
use rustyline::{
    Editor,
    error::*,
};
use nom::{
    branch::*,
    combinator::*,
    character::complete::*,
};

use lambda::*;
use commands::*;
use state::*;
use parsing::*;
use clap::Parser;

mod lambda;
mod commands;
mod state;
mod builtins;
mod parsing;

fn handle_assignment(state: &mut State, input: String, name: String, value: String) -> HistoryEntry {
    state.add_variable(name.clone(), value.clone());
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    hist_entry.parsed = LineType::Assignment(name, value);
    return hist_entry;
}

fn handle_command(state: &mut State, input: String, command: Box<dyn Command>) -> HistoryEntry {
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

fn on_parsing_error(_state: &State, input: String, msg: &str) -> HistoryEntry {
    let output = format!("Parsing Error: {}", msg);
    println!("{}", output);
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    hist_entry.parsed = LineType::Error(msg.to_string());
    hist_entry.output = output;
    return hist_entry;
}

fn match_nop(s: Span) -> IResult<LineType> {
    return space0(s).map(|(rest, _)| (rest, LineType::Nop()))
        .conclude(|_| "".to_owned());
}

fn match_eof(s: Span) -> IResult<LineType> {
    return eof(s).map(|(rest, _)| (rest, LineType::EOF()));
}


fn match_wrapper(config: &Config, s: &str) -> Result<LineType, String> {
    let to_command = |c| LineType::Command(c);
    let to_lambda = |l| LineType::Lambda(l);
    let to_assignment = |(k, v)| LineType::Assignment(k, v);
    let (s, _) = space0::<&str, ()>(s).unwrap();

    match alt((map(assignment_matcher(config.parser), to_assignment),
                map(lambda_matcher(config.parser), to_lambda),
                map(match_command, to_command),
                match_nop, match_eof))(Span::new(s)) {
        Ok((_, parsed)) => Ok(parsed),
        Err(err) => match err {
            nom::Err::Incomplete(_) => Err("parsing incomplete".to_owned()),
            nom::Err::Error(e) => Err(e.message()),
            nom::Err::Failure(e) => Err(e.message()),
        },
    }
}

fn parse_line(input: String, state: &mut State) -> Result<(), bool> {
    let parser_result = match_wrapper(&state.config, &input);

    let hist_entry = match parser_result {
        Ok(parsed) => match parsed {
            LineType::Assignment(k, v) => handle_assignment(state, input, k, v),
            LineType::Command(c) => handle_command(state, input, c),
            LineType::EOF() => return Err(true),
            LineType::Error(e) => on_parsing_error(state, input, &e),
            LineType::Lambda(t) => handle_lambda(state, input, t),
            LineType::Nop() => return Ok(()),
        },
        Err(msg) => on_parsing_error(state, input, &msg),
    };

    let is_error = match hist_entry.parsed {
        LineType::Error(_) => true,
        _ => false,
    };

    state.history.push(hist_entry);

    if is_error {
        return Err(false);
    } else {
        return Ok(());
    }
}

fn file(state: &mut State, filename: &str) {
    let file = File::open(filename).expect("Unable to open file");
    let mut reader = BufReader::new(file);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => if n == 0 {
                break;
            },
            Err(e) => {
                println!("Fatal IO Error: {}", e.to_string());
                break;
            },
        }

        let line = line.trim().to_owned();

        println!("> {}", line);
        let result = parse_line(line, state);
        if let Err(_) = result {
            println!("A fatal error occurred while parsing '{}'", filename);
            break;
        }
        println!();
    }
}

fn repl(state: &mut State) {
    let mut rl = Editor::<()>::new().unwrap();

    loop {
        let line = rl.readline("[λ] ");
        match line {
            Ok(input) => {
                rl.add_history_entry(input.as_str());
                let result = parse_line(input, state);
                if let Err(fatal) = result {
                    if fatal { break; }
                }
                println!();
            },
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Fatal Error: {:?}", err);
                break
            }
        }
    }
}

fn main() {
    // lambda_info("(((\\x . (\\y . x)) x) ((\\x . (x x)) (\\x . (x x))))");
    let config = Config::parse();
    let mut state = State::init(config);

    match &state.config.file {
        Some(path) => {
            let path = path.clone();
            file(&mut state, &path)
        },
        None => repl(&mut state),
    }
    println!();
}
