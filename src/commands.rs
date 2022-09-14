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
struct EchoCommand { msg: String }

#[derive(Clone,Debug,Default)]
struct HistoryCommand;

#[derive(Clone,Debug,Default)]
struct InfoCommand;

#[derive(Clone,Debug,Default)]
struct PrintCommand { var: String }

#[derive(Clone,Debug,Default)]
struct StepsCommand;

#[derive(Clone,Debug,Default)]
struct StoreCommand { name: String }

#[derive(Clone,Debug,Default)]
struct VariablesCommand;


pub trait Command: Debug {
    fn execute(&self, state: &mut State) -> String;
    fn clone_to_box(&self) -> Box<dyn Command>;
    fn keyword() -> &'static str where Self: Sized;
    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> where Self: Sized;

    fn match_keyword(s: Span) -> IResult<()> where Self: Sized{
        return map(tag(Self::keyword()), |_| ())(s);
    }

    fn match_command(s: Span) -> IResult<Box<dyn Command>> where Self: Sized {
        let (rest, _) = with_err(tag(Self::keyword())(s), s,
                                 format!("unknown command '{}'", s))?;
        let (rest, _) = alt((recognize(space1), recognize(pair(space0,eof))))(rest)?;
        return Self::match_arguments(rest);
    }
}


impl Command for BuiltinsCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> String {
        state.builtins.iter().map(|(k, v)| format!("{}\t= {}", k, v.to_string()))
            .collect::<Vec<String>>().join("\n")
    }

    fn keyword() -> &'static str {
        return "builtins";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        return match_no_arguments::<Self>()(s);
    }
}

impl Command for EchoCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, _state: &mut State) -> String {
        return format!(" {} ", self.msg);
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

    fn execute(&self, state: &mut State) -> String {
        if state.history.is_empty() {
            return format!("no history entry found");
        } else {
            let items: Vec<String> = state.history.iter()
                .map(|e| e.to_string()).collect();
            return items.join("\n");
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

    fn execute(&self, state: &mut State) -> String {
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

    fn keyword() -> &'static str {
        return "info";
    }

    fn match_arguments(s: Span) -> IResult<Box<dyn Command>> {
        return match_no_arguments::<Self>()(s);
    }
}

impl Command for PrintCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> String {
        match state.builtins.get(self.var.as_str()) {
            Some(term) => term.to_string(),
            None => match state.variables.get(&self.var) {
                Some(term) => term.to_string(),
                None => format!("Variable '{}' not found", self.var),
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

impl Command for StepsCommand {
    fn clone_to_box(&self) -> Box<dyn Command> {
        return Box::new(self.clone());
    }

    fn execute(&self, state: &mut State) -> String {
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

    fn execute(&self, state: &mut State) -> String {
        if let Some(hist_entry) = state.last_lambda() {
            if let LineType::Lambda(tree) = &hist_entry.parsed {
                let result = state.add_variable(self.name.clone(), tree.clone());
                return match result {
                    Ok(_) => format!("Added variable mapping for '{}'", self.name),
                    Err(()) => format!("Unable to overwrite builtin"),
                }
            } else {
                panic!("Malformed history entry");
            }
        } else {
            return String::from("no history entry found");
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

    fn execute(&self, state: &mut State) -> String {
        state.variables.iter().map(|(k, v)| format!("{}\t= {}", k, v.to_string()))
            .collect::<Vec<String>>().join("\n")
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
        EchoCommand::match_command,
        HistoryCommand::match_command,
        InfoCommand::match_command,
        PrintCommand::match_command,
        StepsCommand::match_command,
        StoreCommand::match_command,
        VariablesCommand::match_command,
    );

    return alt(command_matchers)(rest);
}
