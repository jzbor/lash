use std::fmt::Display;

use crate::{lambda::*, interpreter::Interpreter};

#[derive(Debug, Clone, Copy)]
pub enum Macro {
    Debug,
    Nop,
    Normalize,
    Reduce,
}

impl Macro {
    pub fn get(name: &str) -> Option<Self> {
        match name {
            "debug" => Some(Self::Debug),
            "nop" => Some(Self::Nop),
            "normalize" => Some(Self::Normalize),
            "reduce" => Some(Self::Reduce),
            _ => None,
        }
    }

    pub fn apply(self, interpreter: &Interpreter, term: LambdaTree) -> LambdaTree {
        use Macro::*;
        match self {
            Debug => { println!("{}", term); term },
            Nop => term,
            Normalize => interpreter.strategy().normalize(term.clone()),
            Reduce => if let Some((reduced, _)) = interpreter.strategy().reduce(term.clone()) { reduced } else { term },
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
        };
        write!(f, "{}", s)
    }
}
