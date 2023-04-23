use std::fmt::Display;

use clap::ArgEnum;

use crate::{lambda::*, interpreter::Interpreter};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Macro {
    Debug,
    Nop,
    NewLine,
    Normalize,
    VNormalize,
    Reduce,
    VReduce,
}

impl Macro {
    pub fn get(name: &str) -> Option<Self> {
        clap::ValueEnum::from_str(name, false).ok()
    }

    pub fn apply(self, interpreter: &Interpreter, term: LambdaTree) -> LambdaTree {
        use Macro::*;
        match self {
            Debug => { println!("{}", term); term },
            NewLine => { println!(); term },
            Nop => term,
            Normalize => interpreter.strategy().normalize(term.clone(), false),
            VNormalize => interpreter.strategy().normalize(term.clone(), true),
            Reduce => if let Some(reduced) = interpreter.strategy().reduce(term.clone(), false) { reduced } else { term },
            VReduce => if let Some(reduced) = interpreter.strategy().reduce(term.clone(), true) { reduced } else { term },
        }
    }
}

impl Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}
