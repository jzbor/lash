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


#[derive(Debug,Copy,Clone,Default,clap::ArgEnum)]
pub enum Mode {
    #[default]
    Normalize, Reduce, Validate,
}

fn handle_assignment(state: &mut State, input: String, name: String, term: LambdaNode) -> HistoryEntry {
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    match state.add_variable(name.clone(), term.clone()) {
        Ok(term) => hist_entry.parsed = LineType::Assignment(name, term),
        Err(_) => {
            let msg = "Error: overwriting builtins is not allowed";
            println!("{}", msg);
            hist_entry.parsed = LineType::Error(msg.to_owned());
            hist_entry.output = msg.to_owned();
        },
    };
    return hist_entry;
}

fn handle_command(state: &mut State, input: String, command: Box<dyn Command>) -> HistoryEntry {
    let mut hist_entry = HistoryEntry::default();
    let output = match command.execute(state) {
        Ok(answer) => {
            hist_entry.parsed = LineType::Command(command);
            answer
        },
        Err(msg) => {
            let output = format!("Error: {}", msg);
            hist_entry.parsed = LineType::Error(output.clone());
            output
        },
    };

    println!("{}", output);
    hist_entry.input = input;
    hist_entry.output = output;
    return hist_entry;
}

fn replace_builtins_and_variables(state: &State, tree: &LambdaNode) -> (LambdaNode, u32, u32) {
    let mut var_subs = 0;
    let variables_borrowed = state.variables.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let mut new_tree = tree.clone();
    loop {
        let (t, vs) = new_tree.substitute(&variables_borrowed);
        if vs == 0 { break; }
        new_tree = t;
        var_subs += vs;
    }

    let mut bi_subs = 0;
    let builtins_borrowed = state.builtins.iter().map(|(k, v)| (*k, v)).collect();
    loop {
        let (t, bs) = new_tree.substitute(&builtins_borrowed);
        if bs == 0 { break; }
        new_tree = t;
        bi_subs += bs;
    }

    return (new_tree, bi_subs, var_subs);
}

fn handle_lambda(state: &State, input: String, tree: LambdaNode) -> HistoryEntry {
    let (tree, bsubs, vsubs) = replace_builtins_and_variables(state, &tree);
    let result = match state.config.mode {
        Mode::Normalize => normalize(&tree, state.config.strategy),
        Mode::Reduce => reduce(&tree, state.config.strategy),
        Mode::Validate => Ok((tree, 0)),
    };

    return match result {
        Ok((tree, nbeta)) => {
            let output = format!("{}", tree.to_string());
            println!("{}", output);
            HistoryEntry {
                input: input,
                parsed: LineType::Lambda(tree),
                output: output,
                nbeta: nbeta,
                var_subs: vsubs,
                bi_subs: bsubs,
            }
        },
        Err(msg) => {
            let output = format!("Error: {}", &msg);
            println!("{}", output);
            HistoryEntry {
                input: input,
                parsed: LineType::Error(msg),
                output: output,
                nbeta: 0,
                var_subs: vsubs,
                bi_subs: bsubs,
            }
        },
    }
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

fn normalize(tree: &LambdaNode, strategy: ReductionStrategy) -> Result<(LambdaNode, u32), String> {
    return Ok(tree.normalize(strategy));
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

fn reduce(tree: &LambdaNode, strategy: ReductionStrategy) -> Result<(LambdaNode, u32), String> {
    let tree = tree.reduce(strategy);
    return Ok((tree, 1));
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
            Err(ReadlineError::Interrupted) => {
                println!();
            },
            Err(ReadlineError::Eof) => {
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
