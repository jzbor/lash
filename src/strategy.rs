use crate::lambda::*;
use colored::Colorize;

#[derive(Debug, Copy, Clone)]
pub enum Strategy {
    Applicative,
    Normal,
}

impl Strategy {
    pub fn normalize(&self, term: LambdaTree, verbose: bool) -> LambdaTree {
        let mut current = term;
        loop {
            if let Some(next) = self.reduce(current.clone(), verbose) {
                current = next;
            } else {
                return current;
            }
        }
    }

    pub fn reduce(&self, term: LambdaTree, verbose: bool) -> Option<LambdaTree> {
        use Strategy::*;
        let result = match self {
            Applicative => Self::reduce_applicative(term, verbose),
            Normal => Self::reduce_normal(term, verbose),
        };
        if let Some((lambda, string)) = result {
            if verbose {
                println!("{}", string.unwrap());
            };
            Some(lambda)
        } else {
            None
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
                    let string = Self::reduction_format_redex(&left_term, &right_term, verbose);
                    return Some((inner_term.substitute(var_name, right_term.clone()), string));
                }

                if let Named(named) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = named.term().node() {
                        let string = Self::reduction_format_redex(&left_term, &right_term, verbose);
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
            Macro(..) => panic!(),
            Named(named) => Self::reduce_normal(named.term(), verbose),
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
                        let string = Self::reduction_format_redex(&left_term, &right_term, verbose);
                        return Some((inner_term.substitute(var_name, right_term.clone()), string));
                    }
                }

                if let Named(named) = left_term.node() {
                    if let Abstraction(var_name, inner_term) = named.term().node() {
                        let string = Self::reduction_format_redex(&left_term, &right_term, verbose);
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
            Macro(..) => panic!(),
            Named(named) => Self::reduce_applicative(named.term(), verbose),
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
            Some(format!("{} {}", left_term.fmt_with_parenthesis(true).blue(), right_term.fmt_with_parenthesis(false).bright_blue()))
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
