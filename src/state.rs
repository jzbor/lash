use std::collections::HashMap;
use crate::commands::*;
use crate::lambda::*;
use crate::builtins::*;
use crate::Mode;

#[derive(Debug)]
pub enum LineType {
    Assignment(String, LambdaNode),
    Command(Box<dyn Command>),
    EOF(),
    Error(String),
    Lambda(LambdaNode),
    Nop(),
}

impl Clone for LineType {
    fn clone(&self) -> LineType {
        match self {
            LineType::Assignment(name, value) => LineType::Assignment(name.clone(), value.clone()),
            LineType::Command(command) => LineType::Command(command.clone_to_box()),
            LineType::EOF() => LineType::EOF(),
            LineType::Error(msg) => LineType::Error(msg.clone()),
            LineType::Lambda(tree) => LineType::Lambda(tree.clone()),
            LineType::Nop() => LineType::Nop(),
        }
    }
}

#[derive(Debug,Clone)]
pub struct HistoryEntry {
    pub input: String,
    pub parsed: LineType,
    pub output: String,
    pub nbeta: u32,
    pub var_subs: u32,
    pub bi_subs: u32,
}

#[derive(Debug,Clone,clap::Parser)]
pub struct Config {
    #[clap(short, long, arg_enum, value_parser, default_value_t)]
    pub strategy: ReductionStrategy,

    #[clap(short, long, value_parser)]
    pub file: Option<String>,

    #[clap(short, long, arg_enum, value_parser, default_value_t)]
    pub mode: Mode,
}

pub struct State {
    pub history: Vec<HistoryEntry>,
    pub config: Config,
    pub builtins: HashMap<&'static str, LambdaNode>,
    pub variables: HashMap<String, LambdaNode>,
    pub interactive: bool,
}

impl State {
    pub fn init(config: Config) -> State {
        let builtins = get_builtins();
        return State {
            history: Vec::new(),
            config: config,
            builtins: builtins,
            variables: HashMap::new(),
            interactive: true,
        }
    }

    pub fn add_variable(&mut self, name: String, term: LambdaNode) -> Result<LambdaNode, ()> {
        if self.builtins.contains_key(name.as_str()) {
            return Err(());
        } else if let Some(tree) = self.variables.get(&name) {
            let sigma = HashMap::from([(name.as_str(), tree)]);
            let (term, _) = term.substitute(&sigma);
            self.variables.insert(name, term.clone());
            return Ok(term);
        } else {
            self.variables.insert(name, term.clone());
            return Ok(term);
        }
    }

    pub fn last_lambda(&self) -> Option<&HistoryEntry> {
        if self.history.len() > 0 {
            let last_index = self.history.len() - 1;
            for i in 0..self.history.len() {
                let entry = &self.history[last_index - i];
                if let LineType::Lambda(_) = &entry.parsed {
                    return Some(entry);
                }
            }
        }
        return None;
    }
}

impl Default for Config {
    fn default() -> Config {
        return Config {
            strategy: ReductionStrategy::Normal,
            file: None,
            mode: Mode::Normalize,
        };
    }
}

impl Default for HistoryEntry {
    fn default() -> HistoryEntry {
        return HistoryEntry {
            input: String::new(),
            parsed: LineType::Error("Empty entry".to_string()),
            output: String::new(),
            nbeta: 0,
            var_subs: 0,
            bi_subs: 0,
        }
    }
}

impl HistoryEntry {
    pub fn to_string(&self) -> String {
        match self.parsed {
            LineType::Lambda(_) => format!("{}\n  => {}", self.input, self.output),
            LineType::Error(_) => format!("{} [{}]", self.input, self.output),
            _ => format!("{}", self.input),
        }
    }
}
