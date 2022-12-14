use std::fs::File;
use std::io::{BufRead, BufReader};
use core::fmt::Debug;
use nom::{
    branch::*,
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    sequence::*,
    combinator,
};

use crate::state::*;
use crate::lambda::*;
use crate::parsing::*;
use crate::process_line;
use crate::Mode;


#[derive(Clone,Debug,Default)]
struct AlphaEqCommand { first: String, second: String }

#[derive(Clone,Debug,Default)]
struct BuiltinsCommand;

#[derive(Clone,Debug,Default)]
struct CommentCommand { comment: String }

#[derive(Clone,Debug,Default)]
struct DeBruijnCommand { term: Option<LambdaNode> }

#[derive(Clone,Debug,Default)]
struct EchoCommand { msg: String }

#[derive(Clone,Debug,Default)]
struct HistoryCommand;

#[derive(Clone,Debug,Default)]
struct InfoCommand;

#[derive(Clone,Debug,Default)]
struct ModeCommand { mode: Mode }

#[derive(Clone,Debug,Default)]
struct NormalEqCommand { first: String, second: String }

#[derive(Clone,Debug)]
struct NormalizeCommand { term: Option<LambdaNode> }

#[derive(Clone, Debug)]
struct NormalizeVariableCommand { var: String }

#[derive(Clone,Debug,Default)]
struct PrintCommand { var: String }

#[derive(Clone,Debug)]
struct ReduceCommand { term: Option<LambdaNode> }

#[derive(Clone,Debug)]
struct ReduceVariableCommand { var: String }

#[derive(Clone,Debug,Default)]
struct SourceCommand { filename: String }

#[derive(Clone,Debug,Default)]
struct StepsCommand;

#[derive(Clone,Debug,Default)]
struct StoreCommand { name: String }

#[derive(Clone,Debug,Default)]
struct StrategyCommand { strategy: ReductionStrategy }

#[derive(Clone,Debug,Default)]
struct VariablesCommand;


pub trait Command: Debug {
    fn execute(&self, state: &mut State) -> Result<String, String>;
    fn clone_to_box(&self) -> Box<dyn Command>;
    fn keyword() -> &'static str where Self: Sized;
    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> where Self: Sized;

    fn match_keyword(s: Span) -> IResult<()> where Self: Sized{
        return map(tag(Self::keyword()), |_| ())(s);
    }

    fn match_command(s: Span) -> IResult<Box<dyn Command>> where Self: Sized {
        let (rest, _) = with_err(tag(Self::keyword())(s), s,
                                 format!("unknown command '{}'", s))?;
        let (rest, _) = with_err(alt((recognize(space1), recognize(pair(space0,eof))))(rest), rest,
                                 format!("unknown command"))?;
        return Self::match_arguments(rest);
    }
}

impl Command for AlphaEqCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let (first, _, _) = match state.builtins.get(self.first.as_str()) {
            Some(term) => term,
            None => match state.variables.get(&self.first) {
                Some(term) => term,
                None => return Err(format!("variable '{}' not found", self.first)),
            }
        }.resolve_vars(&state.builtins, &state.variables);
        let (second, _, _) = match state.builtins.get(self.second.as_str()) {
            Some(term) => term,
            None => match state.variables.get(&self.second) {
                Some(term) => term,
                None => return Err(format!("variable '{}' not found", self.second)),
            }
        }.resolve_vars(&state.builtins, &state.variables);
        return Ok(format!("{}", first == second));
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let msg = "command takes exactly two arguments";
        let (rest, first) = with_err(match_variable_name(s), s, msg.to_owned())?;
        let (rest, _) = with_err(space1(rest), rest, msg.to_owned())?;
        let (rest, second) = with_err(match_variable_name(rest), rest, msg.to_owned())?;
        let (rest, _) = with_err(space0(rest), rest, msg.to_owned())?;
        let (rest, _) = with_err(eof(rest), rest, msg.to_owned())?;
        return Ok((rest, Box::new(AlphaEqCommand { first: (*first).to_owned(), second: (*second).to_owned() })));
    }

    fn keyword() -> &'static str {
        return "alpha";
    }
}


impl Command for BuiltinsCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        Ok(state.builtins.iter().map(|(k, v)| format!("{}\t= {}", k, v.to_string()))
            .collect::<Vec<String>>().join("\n"))
    }

    fn keyword() -> &'static str {
        return "builtins";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        return match_no_arguments::<Self>()(s);
    }
}

impl Command for CommentCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, _state: &mut State) -> Result<String, String> {
        return Ok("".to_owned());
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, output) = combinator::rest(s)?;
        return Ok((rest, Box::new(CommentCommand { comment: (*output).to_owned() })));
    }

    fn keyword() -> &'static str {
        return ":";
    }
}

impl Command for DeBruijnCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let tree = term_or_last_lambda(&self.term, state)?;
        let (tree, _, _) = tree.resolve_vars(&state.builtins, &state.variables);
        return Ok(tree.to_debrujin().to_string());
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, _) = space0(s)?;
        if eof::<Span,()>(s).is_ok() {
            return Ok((rest, Box::new(DeBruijnCommand { term: None })));
        } else {
            let (rest, tree) = match_complete_lambda(s)?;
            return Ok((rest, Box::new(DeBruijnCommand { term: Some(tree) })));
        }
    }

    fn keyword() -> &'static str {
        return "debruijn";
    }
}

impl Command for EchoCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, _state: &mut State) -> Result<String, String> {
        return Ok(format!(" {} ", self.msg));
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, output) = combinator::rest(s)?;
        return Ok((rest, Box::new(EchoCommand { msg: (*output).to_owned() })));
    }

    fn keyword() -> &'static str {
        return "echo";
    }
}

impl Command for HistoryCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        if state.history.is_empty() {
            return Err(format!("no history entry found"));
        } else {
            let items: Vec<String> = state.history.iter()
                .map(|e| e.to_string()).collect();
            return Ok(items.join("\n"));
        }
    }

    fn keyword() -> &'static str {
        return "hist";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        return match_no_arguments::<Self>()(s);
    }
}

impl Command for InfoCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let option = state.last_lambda();
        match option {
            Some(e) => Ok(format!("beta reductions: {}
builtin substitutions: {}
variable substitutions: {}",
                    e.nbeta, e.bi_subs, e. var_subs)),
            None => Err(format!("no history entry found")),
        }
    }

    fn keyword() -> &'static str {
        return "info";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        return match_no_arguments::<Self>()(s);
    }
}

impl Command for ModeCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        if !state.interactive {
            return Err("this setting can only be changed in interactive shells".to_owned());
        }

        state.config.mode = self.mode;

        return Ok("Changed mode".to_owned());
    }

    fn keyword() -> &'static str {
        return "mode";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, mode_str) = with_err(recognize(
                                            alt((
                                                tag("normalize"),
                                                tag("reduce"),
                                                tag("validate"),
                                            ))
                                        )(s), s, "unknown mode".to_owned())?;
        let mode = match *mode_str {
            "normalize" => Mode::Normalize,
            "reduce" => Mode::Reduce,
            "validate" => Mode::Validate,
            &_ => return Err(nom::Err::Error(ParseError::new("unknown mode".to_owned(), rest))),
        };
        return Ok((rest, Box::new(ModeCommand { mode: mode })));
    }
}

impl Command for NormalEqCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let (first, _, _) = match state.builtins.get(self.first.as_str()) {
            Some(term) => term,
            None => match state.variables.get(&self.first) {
                Some(term) => term,
                None => return Err(format!("variable '{}' not found", self.first)),
            }
        }.resolve_vars(&state.builtins, &state.variables);
        let (second, _, _) = match state.builtins.get(self.second.as_str()) {
            Some(term) => term,
            None => match state.variables.get(&self.second) {
                Some(term) => term,
                None => return Err(format!("variable '{}' not found", self.second)),
            }
        }.resolve_vars(&state.builtins, &state.variables);
        let (first, _) = first.normalize(state.config.strategy);
        let (second, _) = second.normalize(state.config.strategy);
        return Ok(format!("{}", first == second));
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let msg = "command takes exactly two arguments";
        let (rest, first) = with_err(match_variable_name(s), s, msg.to_owned())?;
        let (rest, _) = with_err(space1(rest), rest, msg.to_owned())?;
        let (rest, second) = with_err(match_variable_name(rest), rest, msg.to_owned())?;
        let (rest, _) = with_err(space0(rest), rest, msg.to_owned())?;
        let (rest, _) = with_err(eof(rest), rest, msg.to_owned())?;
        return Ok((rest, Box::new(NormalEqCommand { first: (*first).to_owned(), second: (*second).to_owned() })));
    }

    fn keyword() -> &'static str {
        return "eq";
    }
}

impl Command for NormalizeCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let term = term_or_last_lambda(&self.term, state)?;
        let (term, _, _) = term.resolve_vars(&state.builtins, &state.variables);
        let (normal, nbeta) = term.normalize(state.config.strategy);
        return Ok(format!("Normal form: {}\nBeta reductions: {}", normal.to_string(), nbeta));
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, _) = space0(s)?;
        if eof::<Span,()>(s).is_ok() {
            return Ok((rest, Box::new(NormalizeCommand { term: None })));
        } else {
            let (rest, tree) = match_complete_lambda(s)?;
            return Ok((rest, Box::new(NormalizeCommand { term: Some(tree) })));
        }
    }

    fn keyword() -> &'static str {
        return "normal";
    }
}

impl Command for NormalizeVariableCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        match state.builtins.get(self.var.as_str()) {
            Some(_term) => Err(format!("Cannot normalize builtins")),
            None => match state.variables.get(&self.var) {
                Some(term) => {
                    let (normal, nbeta) = term.normalize(state.config.strategy);
                    state.variables.insert(self.var.clone(), normal);
                    Ok(format!("Normalized variable '{}'\nBeta reductions: {}", self.var, nbeta))
                },
                None => Err(format!("variable '{}' not found", self.var)),
            },
        }
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let msg = "command takes exactly one variable name as argument";
        let (rest, var) = with_err(match_variable_name(s), s, msg.to_owned())?;
        let (rest, _) = with_err(space0(rest), rest, msg.to_owned())?;
        let (rest, _) = with_err(eof(rest), rest, msg.to_owned())?;
        return Ok((rest, Box::new(NormalizeVariableCommand { var: (*var).to_owned() })));
    }

    fn keyword() -> &'static str {
        return "normalize";
    }
}

impl Command for PrintCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        match state.builtins.get(self.var.as_str()) {
            Some(term) => Ok(term.to_string()),
            None => match state.variables.get(&self.var) {
                Some(term) => Ok(term.to_string()),
                None => Err(format!("variable '{}' not found", self.var)),
            },
        }
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let msg = "print takes exactly one argument";
        let (rest, var) = with_err(match_variable_name(s), s, msg.to_owned())?;
        let (rest, _) = with_err(space0(rest), rest, msg.to_owned())?;
        let (rest, _) = with_err(eof(rest), rest, msg.to_owned())?;
        return Ok((rest, Box::new(PrintCommand { var: (*var).to_owned() })));
    }

    fn keyword() -> &'static str {
        return "print";
    }
}

impl Command for ReduceCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let term = term_or_last_lambda(&self.term, state)?;
        let (term, _, _) = term.resolve_vars(&state.builtins, &state.variables);
        match term.next_redex(state.config.strategy) {
            Some((redex, _depth)) => {
                let reduced = term.reduce(state.config.strategy);
                return Ok(format!("Redex: {}\n => {}", redex.to_string(), reduced.to_string()));
            },
            None => return Ok("This term already has normal form".to_owned()),
        }
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, _) = space0(s)?;
        if eof::<Span,()>(s).is_ok() {
            return Ok((rest, Box::new(ReduceCommand { term: None })));
        } else {
            let (rest, tree) = match_complete_lambda(s)?;
            return Ok((rest, Box::new(ReduceCommand { term: Some(tree) })));
        }
    }

    fn keyword() -> &'static str {
        return "reduced";
    }
}

impl Command for ReduceVariableCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        match state.builtins.get(self.var.as_str()) {
            Some(_term) => Err(format!("Cannot normalize builtins")),
            None => match state.variables.get(&self.var) {
                Some(term) => match term.next_redex(state.config.strategy) {
                    Some((redex, _depth)) => {
                        let reduced = term.reduce(state.config.strategy);
                        state.variables.insert(self.var.clone(), reduced.clone());
                        Ok(format!("Reduced variable '{}' to {}\nRedex: {}",
                                   self.var, reduced.to_string(), redex.to_string()))
                    },
                    None => Ok("This term already has normal form".to_owned()),
                },
                None => Err(format!("variable '{}' not found", self.var)),
            },
        }
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let msg = "command takes exactly one variable name as argument";
        let (rest, var) = with_err(match_variable_name(s), s, msg.to_owned())?;
        let (rest, _) = with_err(space0(rest), rest, msg.to_owned())?;
        let (rest, _) = with_err(eof(rest), rest, msg.to_owned())?;
        return Ok((rest, Box::new(ReduceVariableCommand { var: (*var).to_owned() })));
    }

    fn keyword() -> &'static str {
        return "reduce";
    }
}

impl Command for SourceCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let file = match File::open(&self.filename) {
            Ok(f) => f,
            Err(e) => return Err(format!("Unable to open file ({})", e.to_string())),
        };
        let mut reader = BufReader::new(file);
        let interactive = state.interactive;
        state.interactive = false;
        let mut result = Ok("".to_owned());

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(n) => if n == 0 {
                    // @TODO redundant? process_line() has eof flag
                    break;
                },
                Err(e) => {
                    result = Err(format!("Fatal IO Error: {}", e.to_string()));
                    break;
                },
            }

            let line = line.trim().to_owned();

            let (answer, eof) = process_line(line, state);
            if eof {
                break;
            }
            if let Err(_) = answer {
                result = Err(format!("A fatal error occurred while parsing '{}'", self.filename));
                break;
            }
        }

        state.interactive = interactive;
        return result;
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, output) = combinator::rest(s)?;
        return Ok((rest, Box::new(SourceCommand { filename: (*output).to_owned() })));
    }

    fn keyword() -> &'static str {
        return "source";
    }
}

impl Command for StepsCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
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
                return Ok(lines.join("\n"));
            } else {
                panic!("last_lambda() didn't return lambda entry");
            }
        } else {
            return Err("no history entry found".to_owned());
        }
    }

    fn keyword() -> &'static str {
        return "steps";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        return match_no_arguments::<Self>()(s);
    }
}

impl Command for StoreCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        if let Some(hist_entry) = state.last_lambda() {
            if let LineType::Lambda(tree) = &hist_entry.parsed {
                let result = state.add_variable(self.name.clone(), tree.clone());
                return match result {
                    Ok(_) => Ok(format!("Added variable mapping for '{}'", self.name)),
                    Err(()) => Err(format!("unable to overwrite builtin '{}'", self.name)),
                }
            } else {
                panic!("Malformed history entry");
            }
        } else {
            return Err("no history entry found".to_owned());
        }
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, name) = match_variable_name(s)?;
        return Ok((rest, Box::new(StoreCommand { name: name.to_owned() })));
    }

    fn keyword() -> &'static str {
        return "store";
    }
}

impl Command for StrategyCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        if !state.interactive {
            return Err("this setting can only be changed in interactive shells".to_owned());
        }

        state.config.strategy = self.strategy;

        return Ok("Changed strategy".to_owned());
    }

    fn keyword() -> &'static str {
        return "strategy";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, strategy_str) = with_err(recognize(
                                            alt((
                                                tag("normal"),
                                                tag("applicative"),
                                            ))
                                        )(s), s, "unknown strategy".to_owned())?;
        let strategy = match *strategy_str {
            "normal" => ReductionStrategy::Normal,
            "applicative" => ReductionStrategy::Applicative,
            &_ => return Err(nom::Err::Error(ParseError::new("unknown strategy".to_owned(), rest))),
        };
        return Ok((rest, Box::new(StrategyCommand { strategy: strategy })));
    }
}

impl Command for VariablesCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        Ok(state.variables.iter().map(|(k, v)| format!("{}\t= {}", k, v.to_string()))
            .collect::<Vec<String>>().join("\n"))
    }

    fn keyword() -> &'static str {
        return "vars";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        return match_no_arguments::<Self>()(s);
    }
}

fn match_no_arguments<T>() -> impl FnMut(Span) -> IResult<Box<dyn Command>>
        where T: Clone + Command + Default + 'static {
    return |s| {
        let default: Box<dyn Command> = Box::new(T::default());
        Ok((s, default))
            .conclude(|_| format!("command '{}' does not take any arguments", T::keyword()))
    };
}

fn term_or_last_lambda(optional: &Option<LambdaNode>, state: &State) -> Result<LambdaNode, String> {
    match optional {
        Some(term) => Ok(term.clone()),
        None => if let Some(hist_entry) = state.last_lambda() {
            if let LineType::Lambda(term) = &hist_entry.parsed {
                Ok(term.clone())
            } else {
                panic!("last_lambda() didn't return lambda entry");
            }
        } else {
            Err("no history entry found".to_owned())
        }
    }
}


pub fn match_command(s: Span) -> IResult<Box<dyn Command>> {
    let (rest, _) = char(':')(s)?;
    let (rest, _) = space0(rest)?;

    let command_matchers = (
        AlphaEqCommand::match_command,
        BuiltinsCommand::match_command,
        CommentCommand::match_command,
        DeBruijnCommand::match_command,
        EchoCommand::match_command,
        HistoryCommand::match_command,
        InfoCommand::match_command,
        ModeCommand::match_command,
        NormalEqCommand::match_command,
        NormalizeCommand::match_command,
        NormalizeVariableCommand::match_command,
        PrintCommand::match_command,
        ReduceCommand::match_command,
        ReduceVariableCommand::match_command,
        SourceCommand::match_command,
        StepsCommand::match_command,
        StoreCommand::match_command,
        StrategyCommand::match_command,
        VariablesCommand::match_command,
    );

    return alt(command_matchers)(rest);
}
