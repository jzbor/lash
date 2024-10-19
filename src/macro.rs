use humantime::format_duration;
use std::fmt::Display;
use std::time::Duration;

use clap::ValueEnum;

use crate::{debruijn::DeBruijnNode, error::{LashError, LashResult}, interpreter::Interpreter, lambda::*};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Macro {
    CN,
    CNormalize,
    Dbg,
    DeBruijn,
    Debug,
    Macros,
    N,
    Normalize,
    R,
    Reduce,
    Resolve,
    Time,
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
            println!("{: <8} \t{}", m.to_string(), m.help());
        }
    }

    pub fn apply(self, interpreter: &Interpreter, terms: Vec<LambdaTree>, duration: Duration) -> LashResult<LambdaTree> {
        use Macro::*;

        if terms.len() != self.nargs() {
            return Err(LashError::new_macro_arg_error(self, self.nargs(), terms.len()));
        }

        let term = match self {
            CNormalize | CN => {
                let (term, count) = interpreter.strategy().normalize(terms[0].clone(), false);
                println!("Number of reductions: {}", count);
                term
            },
            DeBruijn => {
                println!("{}", DeBruijnNode::from(terms[0].clone()).to_string());
                terms[0].clone()
            },
            Debug | Dbg => {
                println!("{}", terms[0].clone());
                terms[0].clone()
            },
            Macros => { Self::print_all(); LambdaTree::new_macro(self, terms) },
            Normalize | N => interpreter.strategy().normalize(terms[0].clone(), false).0,
            Reduce | R => if let Some(reduced) = interpreter.strategy().reduce(terms[0].clone(), false) {
                reduced
            } else {
                terms[0].clone()
            },
            Resolve => terms[0].resolve(),
            Time => {
                println!("Time elapsed: {}", format_duration(Duration::from_millis(duration.as_millis() as u64)));
                terms[0].clone()
            },
            VNormalize | VN => interpreter.strategy().normalize(terms[0].clone(), true).0,
            VReduce | VR => if let Some(reduced) = interpreter.strategy().reduce(terms[0].clone(), true) {
                reduced
            } else {
                terms[0].clone()
            },
        };

        Ok(term)
    }

    fn help(&self) -> &str {
        use Macro::*;
        match self {
            CN => "shortcut for cnormalize",
            CNormalize => "normalize and show number of reductions performed",
            Dbg => "shortcut for debug",
            DeBruijn => "print out DeBruijn form",
            Debug => "print out current term (useful in non-interactive mode)",
            Macros => "print available macros",
            N => "shortcut for normalize",
            Normalize => "normalize the given term",
            R => "shortcut for reduce",
            Reduce => "reduce the given term",
            Resolve => "resolve all named terms",
            Time => "time the execution of the macros contained inside the term",
            VN => "shortcut for vnormalize",
            VNormalize => "visually normalize the given term",
            VR => "shortcut for vreduce",
            VReduce => "visually reduce the given term",
        }
    }

    pub fn nargs(&self) -> usize {
        use Macro::*;
        match self {
            CNormalize | CN => 1,
            DeBruijn => 1,
            Debug | Dbg => 1,
            Macros => 0,
            Normalize | N => 1,
            Reduce | R => 1,
            Resolve => 1,
            Time => 1,
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
