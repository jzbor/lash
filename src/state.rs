use std::collections::HashMap;
use crate::LineType;
use crate::lambda::*;
use crate::builtins::*;

#[derive(Debug,Clone)]
pub struct HistoryEntry {
    pub input: String,
    pub parsed: LineType,
    pub output: String,
    pub nbeta: u32,
    pub var_subs: u32,
    pub bi_subs: u32,
}

#[derive(Debug,Copy,Clone)]
pub struct Config {
    pub parser: Parser,
    pub strategy: ReductionStrategy,
    pub interactive: bool,
}

pub struct State {
    pub history: Vec<HistoryEntry>,
    pub config: Config,
    pub builtins: HashMap<&'static str, &'static str>,
    pub variables: HashMap<String, String>,
}

impl State {
    pub fn init() -> State {
        return State {
            history: Vec::new(),
            config: Config::default(),
            builtins: get_builtins(),
            variables: HashMap::new(),
        }
    }

    pub fn add_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
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
            parser: Parser::Default,
            strategy: ReductionStrategy::Normal,
            interactive: true,
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
            LineType::Lambda(_) => format!("{}\n    => {}", self.input, self.output),
            LineType::Error(_) => format!("{} [{}]", self.input, self.output),
            _ => format!("{}", self.input),
        }
    }
}
