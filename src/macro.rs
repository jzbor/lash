use std::fmt::Display;

use clap::ArgEnum;

use crate::{lambda::*, interpreter::Interpreter, error::{LashResult, LashError}};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Macro {
    Dbg,
    Debug,
    N,
    Normalize,
    R,
    Reduce,
    Resolve,
    VN,
    VNormalize,
    VR,
    VReduce,
}

impl Macro {
    pub fn get(name: &str) -> Option<Self> {
        clap::ValueEnum::from_str(name, false).ok()
    }

    pub fn macros() -> &'static [Self] {
        clap::ValueEnum::value_variants()
    }

    pub fn print_all() {
        for m in Self::macros() {
            println!("{} \t\t{}", m, m.help());
        }
    }

    pub fn apply(self, interpreter: &Interpreter, terms: Vec<LambdaTree>) -> LashResult<LambdaTree> {
        use Macro::*;

        if terms.len() != self.nargs() {
            return Err(LashError::new_macro_arg_error(self, self.nargs(), terms.len()));
        }

        let term = match self {
            Debug | Dbg => { println!("{}", terms[0].clone()); terms[0].clone() },
            Normalize | N => interpreter.strategy().normalize(terms[0].clone(), false),
            Reduce | R => if let Some(reduced) = interpreter.strategy().reduce(terms[0].clone(), false) { reduced } else { terms[0].clone() },
            Resolve => terms[0].resolve(),
            VNormalize | VN => interpreter.strategy().normalize(terms[0].clone(), true),
            VReduce | VR => if let Some(reduced) = interpreter.strategy().reduce(terms[0].clone(), true) { reduced } else { terms[0].clone() },
        };

        Ok(term)
    }

    fn help(&self) -> &str {
        use Macro::*;
        match self {
            Dbg => "shortcut for debug",
            Debug => "print out current term (useful in non-interactive mode)",
            N => "shortcut for normalize",
            Normalize => "normalize the given term",
            R => "shortcut for reduce",
            Reduce => "reduce the given term",
            Resolve => "resolve all named terms",
            VN => "shortcut for vnormalize",
            VNormalize => "visually normalize the given term",
            VR => "shortcut for vreduce",
            VReduce => "visually reduce the given term",
        }
    }

    pub fn nargs(&self) -> usize {
        use Macro::*;
        match self {
            Debug | Dbg => 1,
            Normalize | N => 1,
            Reduce | R => 1,
            Resolve => 1,
            VNormalize | VN => 1,
            VReduce | VR => 1,
        }
    }

}

impl Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "!{}", self.to_possible_value().unwrap().get_name())
    }
}
