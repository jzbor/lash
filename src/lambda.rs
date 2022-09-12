use std::collections::HashMap;
use std::collections::HashSet;
use nom::{
    bytes::complete::*,
    character::complete::*,
    combinator::*,
    character::*,
    IResult,
};

mod pure;
mod impure;


#[derive(Debug,Copy,Clone)]
pub enum Parser {
    Default, Pure,
}

#[derive(Debug,Clone)]
pub enum LambdaNode {
    Variable(String), // variable name
    Abstraction(String, Box<LambdaNode>), // variable name, term
    Application(Box<LambdaNode>, Box<LambdaNode>),
}

#[derive(Debug,Clone)]
pub enum ReductionLambdaNode {
    Variable(String), // variable name
    Abstraction(String, Box<ReductionLambdaNode>), // variable name, term
    Application(bool, Box<ReductionLambdaNode>, Box<ReductionLambdaNode>),
}

#[derive(Debug,Copy,Clone)]
pub enum ReductionStrategy {
    Normal, Applicative,
}

impl LambdaNode {
    pub fn application_possible(&self) -> bool {
        if let LambdaNode::Application(left, _right) = self {
            return left.is_abstraction();
        } else {
            return false;
        }
    }

    pub fn binds_variable(&self, variable: &str) -> bool {
        match self {
            LambdaNode::Variable(_) => false,
            LambdaNode::Abstraction(var_name, term) => var_name == variable || term.binds_variable(variable),
            LambdaNode::Application(left, right) => left.binds_variable(variable) || right.binds_variable(variable),
        }
    }

    pub fn bound_variables(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        match self {
            LambdaNode::Variable(_) => (),
            LambdaNode::Abstraction(abs_var, term) => {
                vars.extend(term.bound_variables());
                vars.insert(abs_var.clone());
            },
            LambdaNode::Application(left, right) => {
                vars.extend(left.bound_variables());
                vars.extend(right.bound_variables());
            },
        }
        return vars;
    }

    fn find_next_redex_applicative(&self) -> Option<(ReductionLambdaNode, i32)> {
        match self {
            LambdaNode::Variable(_) => None,
            LambdaNode::Abstraction(var_name, term)
                    => match term.find_next_redex_applicative() {
                Some((tree, depth)) => Some((ReductionLambdaNode::Abstraction(var_name.clone(), Box::new(tree)), depth + 1)),
                None => None,
            },
            LambdaNode::Application(left, right) => {
                let left_option = left.find_next_redex_applicative();
                let right_option = right.find_next_redex_applicative();

                if left_option.is_some() && right_option.is_some() {
                    let (left_tree, left_depth) = left_option.unwrap();
                    let (right_tree, right_depth) = right_option.unwrap();
                    if left_depth >= right_depth {
                        return Some((ReductionLambdaNode::Application(false, Box::new(left_tree),
                                            Box::new(right.to_reduction_lambda_node(false))),
                                left_depth + 1));
                    } else {
                        return Some((ReductionLambdaNode::Application(false,
                                            Box::new(left.to_reduction_lambda_node(false)), Box::new(right_tree)),
                                right_depth + 1));
                    }
                } else if left_option.is_some() {
                    let (left_tree, left_depth) = left_option.unwrap();
                    return Some((ReductionLambdaNode::Application(false, Box::new(left_tree),
                                        Box::new(right.to_reduction_lambda_node(false))),
                            left_depth + 1));
                } else if right_option.is_some() {
                    let (right_tree, right_depth) = right_option.unwrap();
                    return Some((ReductionLambdaNode::Application(false,
                                        Box::new(left.to_reduction_lambda_node(false)), Box::new(right_tree)),
                            right_depth + 1));
                } else if self.application_possible() {
                    return Some((self.to_reduction_lambda_node(true), 0))
                } else {
                    return None;
                }
            }
        }
    }

    fn find_next_redex_normal(&self) -> Option<(ReductionLambdaNode, i32)> {
        match self {
            LambdaNode::Variable(_) => None,
            LambdaNode::Abstraction(var_name, term)
                    => match term.find_next_redex_normal() {
                Some((tree, depth)) => Some((ReductionLambdaNode::Abstraction(var_name.clone(), Box::new(tree)), depth + 1)),
                None => None,
            },
            LambdaNode::Application(left, right) => {
                if self.application_possible() {
                    return Some((self.to_reduction_lambda_node(true), 0))
                } else {
                    let left_option = left.find_next_redex_normal();
                    let right_option = right.find_next_redex_normal();

                    if left_option.is_some() && right_option.is_some() {
                        let (left_tree, left_depth) = left_option.unwrap();
                        let (right_tree, right_depth) = right_option.unwrap();
                        if left_depth >= right_depth {
                            return Some((ReductionLambdaNode::Application(false, Box::new(left_tree),
                                                Box::new(right.to_reduction_lambda_node(false))),
                                    left_depth + 1));
                        } else {
                            return Some((ReductionLambdaNode::Application(false,
                                                Box::new(left.to_reduction_lambda_node(false)), Box::new(right_tree)),
                                    right_depth + 1));
                        }
                    } else if left_option.is_some() {
                        let (left_tree, left_depth) = left_option.unwrap();
                        return Some((ReductionLambdaNode::Application(false, Box::new(left_tree),
                                            Box::new(right.to_reduction_lambda_node(false))),
                                left_depth + 1));
                    } else if right_option.is_some() {
                        let (right_tree, right_depth) = right_option.unwrap();
                        return Some((ReductionLambdaNode::Application(false,
                                            Box::new(left.to_reduction_lambda_node(false)), Box::new(right_tree)),
                                right_depth + 1));
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    pub fn free_variables(&self) -> HashSet<String> {
        let mut vars = HashSet::new();
        match self {
            LambdaNode::Variable(a) => { vars.insert(a.clone()); },
            LambdaNode::Abstraction(abs_var, term) => {
                vars.extend(term.free_variables());
                vars.remove(abs_var);
            },
            LambdaNode::Application(left, right) => {
                vars.extend(left.free_variables());
                vars.extend(right.free_variables());
            },
        }
        return vars;
    }

    pub fn is_variable(&self) -> bool {
        match self {
            LambdaNode::Variable(_) => true,
            _ => false,
        }
    }

    pub fn is_abstraction(&self) -> bool {
        match self {
            LambdaNode::Abstraction(_, _) => true,
            _ => false,
        }
    }

    pub fn is_application(&self) -> bool {
        match self {
            LambdaNode::Application(_, _) => true,
            _ => false,
        }
    }

    pub fn next_redex(&self, strategy: ReductionStrategy) -> Option<(LambdaNode, i32)> {
        let tree_option = match strategy {
            ReductionStrategy::Normal => self.find_next_redex_normal(),
            ReductionStrategy::Applicative => self.find_next_redex_applicative(),
        };

        if let Some((tree, depth)) = tree_option {
            let redex = tree.marked_reduction()?;
            return Some((redex, depth));
        } else {
            return None;
        }
    }

    pub fn normalize(&self, strategy: ReductionStrategy) -> (LambdaNode, u32) {
        let mut term = self.clone();
        let mut counter: u32 = 0;
        while let Some((reduction_tree, _)) = match strategy {
                    ReductionStrategy::Normal => term.find_next_redex_normal(),
                    ReductionStrategy::Applicative => term.find_next_redex_applicative(),
                } {
            term = reduction_tree.reduce();
            counter += 1;
        }
        return (term, counter);
    }

    pub fn reduce(&self, strategy: ReductionStrategy) -> LambdaNode {
        let tree_option = match strategy {
            ReductionStrategy::Normal => self.find_next_redex_normal(),
            ReductionStrategy::Applicative => self.find_next_redex_applicative(),
        };

        return match tree_option {
            Some(tree) => tree.0.reduce(),
            None => self.clone(),
        }
    }

    pub fn substitute(&self, sigma: &HashMap<&str, &LambdaNode>) -> (LambdaNode, u32) {
        match self {
            LambdaNode::Variable(name) => apply_substitution(name.as_str(), sigma),
            LambdaNode::Abstraction(var_name, term) => {
                let rotten = self.free_variables().iter()
                    .map(|z| apply_substitution(z, &sigma).0.free_variables())
                    .fold(HashSet::new(), |mut set, fv| { set.extend(fv); set });
                let fresh = fresh_var(var_name, rotten);
                let fresh_node = LambdaNode::Variable(fresh.clone());
                let mut new_sigma = sigma.clone();
                new_sigma.insert(var_name, &fresh_node);
                let (child, count) = term.substitute(&new_sigma);
                (LambdaNode::Abstraction(fresh, Box::new(child)), count)
            },
            LambdaNode::Application(left, right) => {
                let (left, cleft) = left.substitute(sigma);
                let (right, cright) = right.substitute(sigma);
                (LambdaNode::Application(Box::new(left), Box::new(right)), cleft + cright)
            }
        }
    }

    fn to_reduction_lambda_node(&self, mark: bool) -> ReductionLambdaNode {
        match self {
            LambdaNode::Variable(name) => ReductionLambdaNode::Variable(name.clone()),
            LambdaNode::Abstraction(var_name, term)
                => ReductionLambdaNode::Abstraction(var_name.clone(), Box::new(term.to_reduction_lambda_node(false))),
            LambdaNode::Application(term1, term2)
                => ReductionLambdaNode::Application(mark, Box::new(term1.to_reduction_lambda_node(false)),
                                                    Box::new(term2.to_reduction_lambda_node(false))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            LambdaNode::Variable(name) => name.clone(),
            LambdaNode::Abstraction(var_name, term)
                => format!("({}{} . {})", '\\', var_name, term.to_string()),
            LambdaNode::Application(term1, term2)
                => format!("{} {}", term1.to_string(), term2.to_string()),
        }
    }

    pub fn substitute_var(&self, var_name: &str, replacement: &LambdaNode) -> (LambdaNode, u32) {
        let mut sigma = HashMap::new();
        sigma.insert(var_name, replacement);
        return self.substitute(&sigma);
    }

    pub fn with_vars(&self, var_map: &HashMap<&str, &str>, parser: Parser)
            -> (LambdaNode, u32) {
        let mut owned = HashMap::new();
        for (k, v) in var_map {
            let tree = lambda_matcher(parser)(v).unwrap().1;
            owned.insert(*k, tree);
        }
        let sigma = owned.iter().map(|(k, v)| (*k, v)).collect();
        return self.substitute(&sigma);
    }
}

impl ReductionLambdaNode {
    fn marked_reduction(&self) -> Option<LambdaNode> {
        match self {
            ReductionLambdaNode::Variable(_) => None,
            ReductionLambdaNode::Abstraction(_, term) => term.marked_reduction(),
            ReductionLambdaNode::Application(mark, left, right) => if *mark {
                return Some(LambdaNode::Application(Box::new(left.to_lambda_node()),
                                                Box::new(right.to_lambda_node())));
            } else {
                return left.marked_reduction()
                    .or(right.marked_reduction());
            }
        }
    }

    fn reduce(&self) -> LambdaNode {
        match self {
            ReductionLambdaNode::Variable(v) => LambdaNode::Variable(v.clone()),
            ReductionLambdaNode::Abstraction(var, term)
                => LambdaNode::Abstraction(var.clone(), Box::new(term.reduce())),
            ReductionLambdaNode::Application(mark, left, right) => if *mark {
                if let ReductionLambdaNode::Abstraction(var, left_inner) = &**left {
                    let mut sigma = HashMap::new();
                    let replacement = right.to_lambda_node();
                    sigma.insert(var.as_str(), &replacement);
                    return left_inner.to_lambda_node().substitute(&sigma).0;
                } else {
                    panic!("Malformed ReductionLambdaNode");
                }
            } else {
                return LambdaNode::Application(Box::new(left.reduce()), Box::new(right.reduce()));
            },
        }
    }

    fn to_lambda_node(&self) -> LambdaNode {
        match self {
            ReductionLambdaNode::Variable(name) => LambdaNode::Variable(name.clone()),
            ReductionLambdaNode::Abstraction(var_name, term)
                => LambdaNode::Abstraction(var_name.clone(), Box::new(term.to_lambda_node())),
            ReductionLambdaNode::Application(_, term1, term2)
                => LambdaNode::Application(Box::new(term1.to_lambda_node()),
                                            Box::new(term2.to_lambda_node())),
        }
    }
}

fn apply_substitution(var: &str, sigma: &HashMap<&str, &LambdaNode>) -> (LambdaNode, u32) {
    match sigma.get(var) {
        Some(term) => ((*term).clone(), 1),
        None => (LambdaNode::Variable(var.to_owned()), 0),
    }
}

fn fresh_var(old_var: &str, rotten: HashSet<String>) -> String {
    let mut new_var = old_var.to_owned();
    let mut i = 1;
    while rotten.contains(&new_var) {
        new_var = format!("{}{}", new_var, i);
        i += 1;
    }
    return new_var;
}

pub fn lambda_matcher(parser: Parser) -> impl FnMut(&str) -> IResult<&str, LambdaNode> {
    match parser {
        Parser::Default => match_lambda_default,
        Parser::Pure => match_lambda_pure,
    }
}

pub fn match_lambda_default(s: &str) -> IResult<&str, LambdaNode> {
    impure::match_lambda(s)
}

pub fn match_lambda_pure(s: &str) -> IResult<&str, LambdaNode> {
    pure::match_lambda(s)
}


fn match_lambda_sign(s: &str) -> IResult<&str, &str> {
    return recognize(char('\\'))(s);
}

fn match_variable_name(s: &str) -> IResult<&str, &str> {
    let (rest, name) = take_while1(|x| is_alphanumeric(x as u8) || x == '-' || x == '_')(s)?;
    return Ok((rest, name));
}
