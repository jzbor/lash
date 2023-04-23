use std::fmt::Display;

use crate::{lambda::*, interpreter::Interpreter};

#[derive(Debug, Clone, Copy)]
pub enum Macro {
    Debug,
    Nop,
    Normalize,
    NormalizeVerbose,
    Reduce,
    ReduceVerbose,
}

impl Macro {
    pub fn get(name: &str) -> Option<Self> {
        match name {
            "debug" => Some(Self::Debug),
            "nop" => Some(Self::Nop),
            "normalize" => Some(Self::Normalize),
            "reduce" => Some(Self::Reduce),
            "vnormalize" => Some(Self::NormalizeVerbose),
            "vreduce" => Some(Self::ReduceVerbose),
            _ => None,
        }
    }

    pub fn apply(self, interpreter: &Interpreter, term: LambdaTree) -> LambdaTree {
        use Macro::*;
        match self {
            Debug => { println!("{}", term); term },
            Nop => term,
            Normalize => interpreter.strategy().normalize(term.clone(), false),
            NormalizeVerbose => interpreter.strategy().normalize(term.clone(), true),
            Reduce => if let Some(reduced) = interpreter.strategy().reduce(term.clone(), false) { reduced } else { term },
            ReduceVerbose => if let Some(reduced) = interpreter.strategy().reduce(term.clone(), true) { reduced } else { term },
        }
    }
}

impl Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Macro::*;
        let s = match self {
            Debug => "debug",
            Nop => "nop",
            Normalize => "normalize",
            Reduce => "reduce",
            NormalizeVerbose => "vnormalize",
            ReduceVerbose => "vreduce",
        };
        write!(f, "{}", s)
    }
}
