extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::{Display, Write};
use core::str::FromStr;
use core::time::Duration;

use crate::debruijn::DeBruijnNode;
use crate::environment::Environment;
use crate::error::{LashError, LashResult};
use crate::interpreter::Interpreter;
use crate::lambda::*;
use crate::typing;

// Improved version of
// https://stackoverflow.com/a/64678145/10854888
macro_rules! enum_with_values {
    ($(#[$derives:meta])* $(vis $visibility:vis)? enum $name:ident { $($(#[$nested_meta:meta])* $member:ident),* }) => {
        $(#[$derives])*
        $($visibility)? enum $name {
            $($(#[$nested_meta])* $member),*
        }
        impl $name {
            #[allow(dead_code)]
            pub const VALUES: &'static [$name; $crate::count!($($member)*)] = &[$($name::$member,)*];
            #[allow(dead_code)]
            pub const SIZE: usize = $crate::count!($($member)*);
        }
    };
}

#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

enum_with_values! {
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "std", derive(clap::ValueEnum))]
    #[cfg_attr(feature = "std", clap(rename_all = "lower"))]
    vis pub enum Macro {
        AlphaEq,
        CNormalize,
        DeBruijn,
        Debug,
        Macros,
        Normalize,
        Reduce,
        Resolve,
        Time,
        Type,
        VNormalize,
        VReduce
    }
}

impl Macro {
    pub fn macros() -> &'static [Self] {
        Self::VALUES
    }

    pub fn print_all(out: &mut impl Write) -> LashResult<()>{
        for m in Self::macros() {
            writeln!(out, "{: <8} \t{}", m.to_string(), m.help())?;
        }
        Ok(())
    }

    pub fn apply<E: Environment>(self, interpreter: &mut Interpreter<E>, terms: Vec<LambdaTree>, duration: Duration) -> LashResult<LambdaTree> {
        use Macro::*;

        if terms.len() != self.nargs() {
            return Err(LashError::new_macro_arg_error(self, self.nargs(), terms.len()));
        }

        let strategy = interpreter.strategy();
        let mut stdout = interpreter.env().stdout();
        let term = match self {
            AlphaEq => if terms[0].alpha_eq(&terms[1]) {
                writeln!(stdout, "Terms are alpha equivalent")?;
                LambdaTree::new_abstraction("x".to_owned(),
                    LambdaTree::new_abstraction("y".to_owned(),
                        LambdaTree::new_variable("x".to_owned())
                    )
                )
            } else {
                writeln!(stdout, "Terms are NOT alpha equivalent")?;
                LambdaTree::new_abstraction("x".to_owned(),
                    LambdaTree::new_abstraction("y".to_owned(),
                        LambdaTree::new_variable("y".to_owned())
                    )
                )
            },
            CNormalize => {
                let (term, count) = strategy.normalize(terms[0].clone(), false, &mut stdout);
                writeln!(stdout, "Number of reductions: {}", count)?;
                term
            },
            DeBruijn => {
                writeln!(stdout, "{}", DeBruijnNode::from(terms[0].clone()))?;
                terms[0].clone()
            },
            Debug => {
                writeln!(stdout, "{}", terms[0].clone())?;
                terms[0].clone()
            },
            Macros => { Self::print_all(&mut stdout)?; LambdaTree::new_macro(self, terms) },
            Normalize => strategy.normalize(terms[0].clone(), false, &mut stdout).0,
            Reduce => if let Some(reduced) = strategy.reduce(terms[0].clone(), false, &mut stdout) {
                reduced
            } else {
                terms[0].clone()
            },
            Resolve => terms[0].resolve(),
            Time => {
                #[cfg(feature = "std")]
                writeln!(stdout, "Time elapsed: {}", humantime::format_duration(Duration::from_millis(duration.as_millis() as u64)))?;
                #[cfg(not(feature = "std"))]
                writeln!(stdout, "Time elapsed: {}ms", duration.as_millis() as u64)?;
                terms[0].clone()
            },
            VNormalize => strategy.normalize(terms[0].clone(), true, &mut stdout).0,
            VReduce => if let Some(reduced) = strategy.reduce(terms[0].clone(), true, &mut stdout) {
                reduced
            } else {
                terms[0].clone()
            },
            Type => {
                match typing::infer(terms[0].clone()) {
                    Ok(t) => writeln!(stdout, "Infered type: {}", t)?,
                    Err(e) => writeln!(stdout, "Cannot infer type: {}", e)?,
                }
                terms[0].clone()
            },
        };

        Ok(term)
    }

    fn help(&self) -> &str {
        use Macro::*;
        match self {
            AlphaEq => "check for alpha equivalence and return Church-encoded boolean",
            CNormalize => "normalize and show number of reductions performed",
            DeBruijn => "print out DeBruijn form",
            Debug => "print out current term (useful in non-interactive mode)",
            Macros => "print available macros",
            Normalize => "normalize the given term",
            Reduce => "reduce the given term",
            Resolve => "resolve all named terms",
            Time => "time the execution of the macros contained inside the term",
            Type => "try to infer a type for the given term",
            VNormalize => "visually normalize the given term",
            VReduce => "visually reduce the given term",
        }
    }

    pub fn nargs(&self) -> usize {
        use Macro::*;
        match self {
            AlphaEq => 2,
            CNormalize => 1,
            DeBruijn => 1,
            Debug => 1,
            Macros => 0,
            Normalize => 1,
            Reduce => 1,
            Resolve => 1,
            Time => 1,
            Type => 1,
            VNormalize => 1,
            VReduce => 1,
        }
    }

}

impl Display for Macro {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Macro::*;
        let name = match self {
            AlphaEq => "alphaeq",
            CNormalize => "cnormalize",
            DeBruijn => "debruijn",
            Debug => "debug",
            Macros => "macros",
            Normalize => "normalize",
            Reduce => "reduce",
            Resolve => "resolve",
            Time => "time",
            VNormalize => "vnormalize",
            VReduce => "vreduce",
            Type => "type",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for Macro {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Macro::*;
        match s {
            "cnormalize" => Ok(CNormalize),
            "debug" => Ok(Debug),
            "macros" => Ok(Macros),
            "normalize" => Ok(Normalize),
            "reduce" => Ok(Reduce),
            "resolve" => Ok(Resolve),
            "time" => Ok(Time),
            "vnormalize" => Ok(VNormalize),
            "vreduce" => Ok(VReduce),
            _ => {
                let mut candidates = Macro::VALUES.iter().filter(|m| m.to_string().starts_with(s));
                match candidates.next() {
                    Some(m) => match candidates.next() {
                        Some(_) => Err(()),
                        None => Ok(*m),
                    },
                    None => Err(()),
                }
            },
        }
    }
}
