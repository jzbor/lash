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


#[derive(Clone,Debug,Default)]
struct BuiltinsCommand;

#[derive(Clone,Debug,Default)]
struct DeBruijnCommand { term: Option<LambdaNode> }

#[derive(Clone,Debug,Default)]
struct EchoCommand { msg: String }

#[derive(Clone,Debug,Default)]
struct HistoryCommand;

#[derive(Clone,Debug,Default)]
struct InfoCommand;

#[derive(Clone,Debug)]
struct NormalizeCommand { term: LambdaNode }

#[derive(Clone,Debug,Default)]
struct PrintCommand { var: String }

#[derive(Clone,Debug)]
struct ReduceCommand { term: LambdaNode }

#[derive(Clone,Debug,Default)]
struct StepsCommand;

#[derive(Clone,Debug,Default)]
struct StoreCommand { name: String }

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

impl Command for DeBruijnCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        match &self.term {
            Some(term) => Ok(term.to_debrujin().to_string()),
            None => if let Some(hist_entry) = state.last_lambda() {
                if let LineType::Lambda(tree) = &hist_entry.parsed {
                    Ok(tree.to_debrujin().to_string())
                } else {
                    panic!("last_lambda() didn't return lambda entry");
                }
            } else {
                return Err("no history entry found".to_owned());
            }
        }
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

impl Command for NormalizeCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> Result<String, String> {
        let (normal, nbeta) = self.term.normalize(state.config.strategy);
        return Ok(format!("Normal form: {}\nBeta reductions: {}", normal.to_string(), nbeta));
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, tree) = match_complete_lambda(s)?;
        return Ok((rest, Box::new(NormalizeCommand { term: tree })));
    }

    fn keyword() -> &'static str {
        return "normal";
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
        match self.term.next_redex(state.config.strategy) {
            Some((redex, _depth)) => {
                let reduced = self.term.reduce(state.config.strategy);
                return Ok(format!("Redex: {}\n => {}", redex.to_string(), reduced.to_string()));
            },
            None => return Ok("This term already has normal form".to_owned()),
        }
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        let (rest, tree) = match_complete_lambda(s)?;
        return Ok((rest, Box::new(ReduceCommand { term: tree })));
    }

    fn keyword() -> &'static str {
        return "reduce";
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


pub fn match_command(s: Span) -> IResult<Box<dyn Command>> {
    let (rest, _) = char(':')(s)?;
    let (rest, _) = space0(rest)?;

    let command_matchers = (
        BuiltinsCommand::match_command,
        DeBruijnCommand::match_command,
        EchoCommand::match_command,
        HistoryCommand::match_command,
        InfoCommand::match_command,
        NormalizeCommand::match_command,
        PrintCommand::match_command,
        ReduceCommand::match_command,
        StepsCommand::match_command,
        StoreCommand::match_command,
        VariablesCommand::match_command,
    );

    return alt(command_matchers)(rest);
}
