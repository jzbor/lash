use std::fmt::Display;

use clap::ArgEnum;

use crate::{lambda::*, interpreter::Interpreter, error::{LashResult, LashError}};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Macro {
    Debug,
    Normalize,
    VNormalize,
    Reduce,
    VReduce,
}

impl Macro {
    pub fn get(name: &str) -> Option<Self> {
        clap::ValueEnum::from_str(name, false).ok()
    }

    pub fn apply(self, interpreter: &Interpreter, terms: Vec<LambdaTree>) -> LashResult<LambdaTree> {
        use Macro::*;

        if terms.len() != self.nargs() {
            return Err(LashError::new_macro_arg_error(self, self.nargs(), terms.len()));
        }

        let term = match self {
            Debug => { println!("{}", terms[0].clone()); terms[0].clone() },
            Normalize => interpreter.strategy().normalize(terms[0].clone(), false),
            VNormalize => interpreter.strategy().normalize(terms[0].clone(), true),
            Reduce => if let Some(reduced) = interpreter.strategy().reduce(terms[0].clone(), false) { reduced } else { terms[0].clone() },
            VReduce => if let Some(reduced) = interpreter.strategy().reduce(terms[0].clone(), true) { reduced } else { terms[0].clone() },
        };

        Ok(term)
    }

    pub fn nargs(&self) -> usize {
        use Macro::*;
        match self {
            Debug => 1,
            Normalize => 1,
            VNormalize => 1,
            Reduce => 1,
            VReduce => 1,
        }
    }
}

impl Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{}", self.to_possible_value().unwrap().get_name())
    }
}
