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

fn handle_assignment(state: &mut State, input: String, name: String, term: LambdaNode) -> (Result<String, String>, HistoryEntry) {
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    let result = match state.add_variable(name.clone(), term.clone()) {
        Ok(term) => {
            hist_entry.parsed = LineType::Assignment(name, term);
            Ok("".to_owned())
        },
        Err(_) => {
            let msg = "Error: overwriting builtins is not allowed";
            hist_entry.parsed = LineType::Error(msg.to_owned());
            hist_entry.output = msg.to_owned();
            Err(msg.to_owned())
        },
    };
    return (result, hist_entry);
}

fn handle_command(state: &mut State, input: String, command: Box<dyn Command>) -> (Result<String, String>, HistoryEntry) {
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    let result = match command.execute(state) {
        Ok(answer) => {
            hist_entry.parsed = LineType::Command(command);
            hist_entry.output = answer.clone();
            Ok(answer)
        },
        Err(msg) => {
            let output = format!("Error: {}", msg);
            hist_entry.parsed = LineType::Error(output.clone());
            hist_entry.output = output.clone();
            Err(output)
        },
    };

    return (result, hist_entry);
}

fn handle_lambda(state: &State, input: String, tree: LambdaNode) -> (Result<String, String>, HistoryEntry) {
    let (tree, bsubs, vsubs) = tree.resolve_vars(&state.builtins, &state.variables);
    let result = match state.config.mode {
        Mode::Normalize => normalize(&tree, state.config.strategy),
        Mode::Reduce => reduce(&tree, state.config.strategy),
        Mode::Validate => Ok((tree, 0)),
    };

    return match result {
        Ok((tree, nbeta)) => {
            let output = tree.to_string();
            let hist_entry = HistoryEntry {
                input: input,
                parsed: LineType::Lambda(tree),
                output: output.clone(),
                nbeta: nbeta,
                var_subs: vsubs,
                bi_subs: bsubs,
            };
            (Ok(output), hist_entry)
        },
        Err(msg) => {
            let output = format!("Error: {}", &msg);
            let hist_entry = HistoryEntry {
                input: input,
                parsed: LineType::Error(msg),
                output: output.clone(),
                nbeta: 0,
                var_subs: vsubs,
                bi_subs: bsubs,
            };
            (Ok(output), hist_entry)
        },
    }
}

fn on_parsing_error(_state: &State, input: String, msg: &str) -> (Result<String, String>, HistoryEntry) {
    let output = format!("Parsing Error: {}", msg);
    let mut hist_entry = HistoryEntry::default();
    hist_entry.input = input;
    hist_entry.parsed = LineType::Error(msg.to_string());
    hist_entry.output = output.clone();
    return (Err(output), hist_entry);
}

fn match_nop(s: Span) -> IResult<LineType> {
    return space0(s).map(|(rest, _)| (rest, LineType::Nop()))
        .conclude(|_| "".to_owned());
}

fn match_eof(s: Span) -> IResult<LineType> {
    return eof(s).map(|(rest, _)| (rest, LineType::EOF()));
}


fn match_wrapper(s: &str) -> Result<LineType, String> {
    let to_command = |c| LineType::Command(c);
    let to_lambda = |l| LineType::Lambda(l);
    let to_assignment = |(k, v)| LineType::Assignment(k, v);
    let (s, _) = space0::<&str, ()>(s).unwrap();

    match alt((map(match_assignment, to_assignment),
                map(match_complete_lambda, to_lambda),
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

fn process_line(input: String, state: &mut State) -> (Result<String, String>, bool) {
    let parser_result = match_wrapper(&input);

    let (result, hist_entry) = match parser_result {
        Ok(parsed) => match parsed {
            LineType::Assignment(k, v) => handle_assignment(state, input, k, v),
            LineType::Command(c) => handle_command(state, input, c),
            LineType::EOF() => return (Err("EOF".to_owned()), true),
            LineType::Error(e) => on_parsing_error(state, input, &e),
            LineType::Lambda(t) => handle_lambda(state, input, t),
            LineType::Nop() => return (Ok("".to_owned()), false),
        },
        Err(msg) => on_parsing_error(state, input, &msg),
    };

    state.history.push(hist_entry);

    return match result {
        Err(msg) => (Err(msg), false),
        ok => (ok, false),
    };
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

        println!("{}", line);
        let (result, eof) = process_line(line, state);
        if eof {
            break;
        } else {
            match result {
                Ok(output) => if output != "" {
                    println!(" | {}\n", output.replace("\n", "\n | "));
                },
                Err(msg) => {
                    println!("\n{}", msg);
                    println!("A fatal error occurred while parsing '{}'", filename);
                    break;
                },
            }
        }
    }
}

fn repl(state: &mut State) {
    let mut rl = Editor::<()>::new().unwrap();

    loop {
        let prompt = match state.config.mode {
            Mode::Normalize => "[N] ",
            Mode::Reduce => "[R] ",
            Mode::Validate => "[λ] ",
        };

        let line = rl.readline(prompt);
        match line {
            Ok(input) => {
                rl.add_history_entry(input.as_str());
                let (result, eof) = process_line(input, state);

                if eof {
                    break;
                } else {
                    match result {
                        Ok(output) => if output != "" {
                            println!("{}\n", output);
                        },
                        Err(msg) => if msg != "" {
                            println!("{}\n", msg)
                        },
                    }
                }
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
}
