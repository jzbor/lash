extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use core::fmt::Write;
use core::str::FromStr;

#[cfg(feature = "std")]
use colored::Colorize;

use crate::lambda::*;


#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "std", derive(clap::ValueEnum))]
#[cfg_attr(feature = "std", clap(rename_all = "lower"))]
pub enum Strategy {
    Applicative,
    Normal,
    CallByName,
}

impl Strategy {
    pub fn normalize(&self, term: LambdaTree, verbose: bool, out: &mut impl Write) -> (LambdaTree, usize) {
        let mut current = term;
        let mut nreductions = 0;
        loop {
            if let Some(next) = self.reduce(current.clone(), verbose, out) {
                current = next;
                nreductions += 1;
            } else {
                return (current, nreductions);
            }
        }
    }

    pub fn reduce(&self, term: LambdaTree, verbose: bool, out: &mut impl Write) -> Option<LambdaTree> {
        use Strategy::*;
        let result = match self {
            Applicative => Self::reduce_applicative(term, verbose),
            Normal => Self::reduce_normal(term, verbose),
            CallByName => Self::reduce_cbn(term, verbose),
        };
        if let Some((lambda, string)) = result {
            if verbose {
                let _ignored = writeln!(out, "{}", string.unwrap());
            };
            Some(lambda)
        } else {
            None
        }
    }

    fn reduce_cbn(term: LambdaTree, verbose: bool) -> Option<(LambdaTree, Option<String>)> {
        use LambdaNode::*;
        match term.node() {
            Abstraction(..) => None,
            Application(left_term, right_term) => {
                let left_option = Self::reduce_normal(left_term.clone(), verbose);
                let right_option = Self::reduce_normal(right_term.clone(), verbose);

                if let Abstraction(var_name, inner_term) = left_term.node() {
                    let string = Self::reduction_format_redex(left_term, right_term, verbose);
                    return Some((inner_term.substitute(var_name, right_term.clone()), string));
                }

                if let Named(named) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = named.term().node() {
                        let string = Self::reduction_format_redex(left_term, right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if let ChurchNum(d) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = LambdaTree::unwrap_church_num(*d).node() {
                        let string = Self::reduction_format_redex(left_term, right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if left_term.is_abstraction() {
                    None
                } else if let Some((left_reduced, left_string)) = left_option {
                    let string = Self::reduction_format_application(left_term.clone(), left_string, right_term.clone(), None, verbose);
                    Some((LambdaTree::new_application(left_reduced, right_term.clone()), string))
                } else if let Some((right_reduced, right_string)) = right_option {
                    let string = Self::reduction_format_application(left_term.clone(), None, right_term.clone(), right_string, verbose);
                    Some((LambdaTree::new_application(left_term.clone(), right_reduced), string))
                } else {
                    None
                }
            },
            Variable(_) => None,
            Macro(..) => None,
            Named(named) => Self::reduce_normal(named.term(), verbose),
            ChurchNum(d) => Self::reduce_normal(LambdaTree::unwrap_church_num(*d), verbose),
        }
    }

    fn reduce_normal(term: LambdaTree, verbose: bool) -> Option<(LambdaTree, Option<String>)> {
        use LambdaNode::*;
        match term.node() {
            Abstraction(var_name, term) => {
                let inner_reduced = Self::reduce_normal(term.clone(), verbose);
                if let Some((term, inner_string)) = inner_reduced {
                    let string = inner_string.map(|s| format!("{}{} . {}", '\\', var_name, s));
                    Some((LambdaTree::new_abstraction(var_name.to_owned(), term), string))
                } else {
                    None
                }
            },
            Application(left_term, right_term) => {
                let left_option = Self::reduce_normal(left_term.clone(), verbose);
                let right_option = Self::reduce_normal(right_term.clone(), verbose);

                if let Abstraction(var_name, inner_term) = left_term.node() {
                    let string = Self::reduction_format_redex(left_term, right_term, verbose);
                    return Some((inner_term.substitute(var_name, right_term.clone()), string));
                }

                if let Named(named) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = named.term().node() {
                        let string = Self::reduction_format_redex(left_term, right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if let ChurchNum(d) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = LambdaTree::unwrap_church_num(*d).node() {
                        let string = Self::reduction_format_redex(left_term, right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if left_term.is_abstraction() {
                    None
                } else if let Some((left_reduced, left_string)) = left_option {
                    let string = Self::reduction_format_application(left_term.clone(), left_string, right_term.clone(), None, verbose);
                    Some((LambdaTree::new_application(left_reduced, right_term.clone()), string))
                } else if let Some((right_reduced, right_string)) = right_option {
                    let string = Self::reduction_format_application(left_term.clone(), None, right_term.clone(), right_string, verbose);
                    Some((LambdaTree::new_application(left_term.clone(), right_reduced), string))
                } else {
                    None
                }
            },
            Variable(_) => None,
            Macro(..) => None,
            Named(named) => Self::reduce_normal(named.term(), verbose),
            ChurchNum(d) => Self::reduce_normal(LambdaTree::unwrap_church_num(*d), verbose),
        }
    }

    fn reduce_applicative(term: LambdaTree, verbose: bool) -> Option<(LambdaTree, Option<String>)> {
        use LambdaNode::*;
        match term.node() {
            Abstraction(var_name, term) => {
                let inner_reduced = Self::reduce_applicative(term.clone(), verbose);
                if let Some((term, inner_string)) = inner_reduced {
                    let string = inner_string.map(|s| format!("{}{} . {}", '\\', var_name, s));
                    Some((LambdaTree::new_abstraction(var_name.to_owned(), term), string))
                } else {
                    None
                }
            },
            Application(left_term, right_term) => {
                let left_option = Self::reduce_applicative(left_term.clone(), verbose);
                let right_option = Self::reduce_applicative(right_term.clone(), verbose);

                if let Abstraction(var_name, inner_term) = left_term.node() {
                    let inner_reduced = Self::reduce_applicative(inner_term.clone(), verbose);
                    if inner_reduced.is_none() && right_option.is_none() {
                        let string = Self::reduction_format_redex(left_term, right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if let Named(named) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = named.term().node() {
                        let string = Self::reduction_format_redex(left_term, right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if let ChurchNum(d) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = LambdaTree::unwrap_church_num(*d).node() {
                        let string = Self::reduction_format_redex(left_term, right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if let Some((left_reduced, left_string)) = left_option {
                    let string = Self::reduction_format_application(left_term.clone(), left_string, right_term.clone(), None, verbose);
                    Some((LambdaTree::new_application(left_reduced, right_term.clone()), string))
                } else if let Some((right_reduced, right_string)) = right_option {
                    let string = Self::reduction_format_application(left_term.clone(), None, right_term.clone(), right_string, verbose);
                    Some((LambdaTree::new_application(left_term.clone(), right_reduced), string))
                } else {
                    None
                }
            },
            Variable(_) => None,
            Macro(..) => None,
            Named(named) => Self::reduce_applicative(named.term(), verbose),
            ChurchNum(d) => Self::reduce_applicative(LambdaTree::unwrap_church_num(*d), verbose),
        }
    }

    fn reduction_format_application(left_term: LambdaTree, left_string: Option<String>,
                                     right_term: LambdaTree, right_string: Option<String>,
                                     verbose: bool) -> Option<String> {
        assert!(!(left_string.is_some() && right_string.is_some()));
        if verbose {
            if let Some(left_string) = left_string {
                if left_term.needs_parenthesis(true) {
                    Some(format!("({}) {}", left_string, right_term.fmt_with_parenthesis(false)))
                } else {
                    Some(format!("{} {}", left_string, right_term.fmt_with_parenthesis(false)))
                }
            } else if let Some(right_string) = right_string {
                if right_term.needs_parenthesis(false) {
                    Some(format!("{} ({})", left_term.fmt_with_parenthesis(true), right_string))
                } else {
                    Some(format!("{} {}", left_term.fmt_with_parenthesis(true), right_string))
                }
            } else {
                Some(format!("{} {}", left_term.fmt_with_parenthesis(true), right_term.fmt_with_parenthesis(false)))
            }
        } else {
            None
        }
    }

    fn reduction_format_redex(left_term: &LambdaTree, right_term: &LambdaTree, verbose: bool) -> Option<String> {
        if verbose {
            #[cfg(feature = "std")]
            let result = Some(format!("{} {}",
                left_term.fmt_with_parenthesis(true).blue(),
                right_term.fmt_with_parenthesis(false).bright_blue()));
            #[cfg(not(feature = "std"))]
            let result = Some(format!("{} {}",
                left_term.fmt_with_parenthesis(true),
                right_term.fmt_with_parenthesis(false)));
            result
        } else {
            None
        }
    }
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Applicative
    }
}

impl FromStr for Strategy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "applicative" => Ok(Self::Applicative),
            "normal" => Ok(Self::Normal),
            "callbyname" => Ok(Self::CallByName),
            _ => Err(()),
        }
    }
}
