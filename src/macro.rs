extern crate alloc;

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

enum_with_values! {
    #[derive(Debug, Clone, Copy)]
    #[cfg_attr(feature = "std", derive(clap::ValueEnum))]
    #[cfg_attr(feature = "std", clap(rename_all = "lower"))]
    vis pub enum Macro {
        AlphaEq,
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
}

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
                println!("Terms are alpha equivalent");
                LambdaTree::new_abstraction("x".to_owned(),
                    LambdaTree::new_abstraction("y".to_owned(),
                        LambdaTree::new_variable("x".to_owned())
                    )
                )
            } else {
                println!("Terms are NOT alpha equivalent");
                LambdaTree::new_abstraction("x".to_owned(),
                    LambdaTree::new_abstraction("y".to_owned(),
                        LambdaTree::new_variable("y".to_owned())
                    )
                )
            },
            CNormalize | CN => {
                let (term, count) = strategy.normalize(terms[0].clone(), false, &mut stdout);
                writeln!(stdout, "Number of reductions: {}", count)?;
                term
            },
            DeBruijn => {
                println!("{}", DeBruijnNode::from(terms[0].clone()).to_string());
                terms[0].clone()
            },
            Debug | Dbg => {
                writeln!(stdout, "{}", terms[0].clone())?;
                terms[0].clone()
            },
            Macros => { Self::print_all(&mut stdout)?; LambdaTree::new_macro(self, terms) },
            Normalize | N => strategy.normalize(terms[0].clone(), false, &mut stdout).0,
            Reduce | R => if let Some(reduced) = strategy.reduce(terms[0].clone(), false, &mut stdout) {
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
            VNormalize | VN => strategy.normalize(terms[0].clone(), true, &mut stdout).0,
            VReduce | VR => if let Some(reduced) = strategy.reduce(terms[0].clone(), true, &mut stdout) {
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
            AlphaEq => "check for alpha equivalence and return Church-encoded boolean",
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
            AlphaEq => 2,
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "!{}", self)
    }
}

impl FromStr for Macro {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Macro::*;
        match s {
            "cn" => Ok(CN),
            "cnormalize" => Ok(CNormalize),
            "dbg" => Ok(Dbg),
            "debug" => Ok(Debug),
            "macros" => Ok(Macros),
            "n" => Ok(N),
            "normalize" => Ok(Normalize),
            "r" => Ok(R),
            "reduce" => Ok(Reduce),
            "resolve" => Ok(Resolve),
            "time" => Ok(Time),
            "vn" => Ok(VN),
            "vnormalize" => Ok(VNormalize),
            "vr" => Ok(VR),
            "vreduce" => Ok(VReduce),
            _ => Err(()),
        }
    }
}
