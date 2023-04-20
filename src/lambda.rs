use std::collections::HashMap;
use std::collections::HashSet;
use nom::{
    character::complete::*,
    combinator::*,
};

use crate::parsing::*;


#[derive(Debug,Clone)]
pub enum LambdaNode {
    Variable(String), // variable name
    Abstraction(String, Box<LambdaNode>), // variable name, term
    Application(Box<LambdaNode>, Box<LambdaNode>),
}

#[derive(Debug,Clone,Eq,PartialEq)]
pub enum DeBruijnNode {
    BoundVariable(u32), // binder
    FreeVariable(String), // variable name
    Abstraction(Box<DeBruijnNode>), // term
    Application(Box<DeBruijnNode>, Box<DeBruijnNode>),
}

#[derive(Debug,Clone)]
pub enum ReductionLambdaNode {
    Variable(String), // variable name
    Abstraction(String, Box<ReductionLambdaNode>), // variable name, term
    Application(bool, Box<ReductionLambdaNode>, Box<ReductionLambdaNode>),
}

#[derive(Debug,Copy,Clone,Default,clap::ArgEnum)]
pub enum ReductionStrategy {
    #[default]
    Normal, Applicative,
}

impl DeBruijnNode {
    pub fn to_string(&self) -> String {
        match self {
            DeBruijnNode::FreeVariable(name) => name.clone(),
            DeBruijnNode::BoundVariable(i) => format!("{}", i),
            DeBruijnNode::Abstraction(term)
                => format!("{} {}", '\\', term.to_string()),
            DeBruijnNode::Application(term1, term2)
                => {
                    let s1 = if let DeBruijnNode::FreeVariable(name) = &**term1 {
                        name.to_string()
                    } else if let DeBruijnNode::BoundVariable(i) = &**term1 {
                        format!("{}", i)
                    } else if let DeBruijnNode::Application(_, _) = &**term1 {
                        term1.to_string()
                    } else {
                        format!("({})", term1.to_string())
                    };
                    let s2 = if let DeBruijnNode::FreeVariable(name) = &**term2 {
                        name.to_string()
                    } else if let DeBruijnNode::BoundVariable(i) = &**term2 {
                        format!("{}", i)
                    } else {
                        format!("({})", term2.to_string())
                    };
                    format!("{} {}", s1, s2)
                },
        }
    }

}

impl LambdaNode {
    pub fn church_numeral(n: u32) -> LambdaNode {
        LambdaNode::Abstraction("f".to_owned(),
                    Box::new(LambdaNode::Abstraction("x".to_owned(),
                        Box::new(Self::church_applications(n)))))
    }

    pub fn church_applications(n: u32) -> LambdaNode {
        if n == 0 {
            LambdaNode::Variable("x".to_owned())
        } else {
            LambdaNode::Application(
                        Box::new(LambdaNode::Variable("f".to_owned())),
                        Box::new(Self::church_applications(n - 1)))
        }
    }

    pub fn application_possible(&self) -> bool {
        if let LambdaNode::Application(left, _right) = self {
            left.is_abstraction()
        } else {
            false
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
        vars
    }

    fn find_next_redex_applicative(&self) -> Option<(ReductionLambdaNode, i32)> {
        match self {
            LambdaNode::Variable(_) => None,
            LambdaNode::Abstraction(var_name, term)
                    => term.find_next_redex_applicative().map(|(tree, depth)| (ReductionLambdaNode::Abstraction(var_name.clone(), Box::new(tree)), depth + 1)),
            LambdaNode::Application(left, right) => {
                let left_option = left.find_next_redex_applicative();
                let right_option = right.find_next_redex_applicative();

                if left_option.is_some() && right_option.is_some() {
                    let (left_tree, left_depth) = left_option.unwrap();
                    let (right_tree, right_depth) = right_option.unwrap();
                    if left_depth >= right_depth {
                        Some((ReductionLambdaNode::Application(false, Box::new(left_tree),
                                            Box::new(right.to_reduction_lambda_node(false))),
                                left_depth + 1))
                    } else {
                        Some((ReductionLambdaNode::Application(false,
                                            Box::new(left.to_reduction_lambda_node(false)), Box::new(right_tree)),
                                right_depth + 1))
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
                    => term.find_next_redex_normal().map(|(tree, depth)| (ReductionLambdaNode::Abstraction(var_name.clone(), Box::new(tree)), depth + 1)),
            LambdaNode::Application(left, right) => {
                if self.application_possible() {
                    Some((self.to_reduction_lambda_node(true), 0))
                } else {
                    let left_option = left.find_next_redex_normal();
                    let right_option = right.find_next_redex_normal();

                    if left_option.is_some() && right_option.is_some() {
                        let (left_tree, left_depth) = left_option.unwrap();
                        let (right_tree, right_depth) = right_option.unwrap();
                        if left_depth >= right_depth {
                            Some((ReductionLambdaNode::Application(false, Box::new(left_tree),
                                                Box::new(right.to_reduction_lambda_node(false))),
                                    left_depth + 1))
                        } else {
                            Some((ReductionLambdaNode::Application(false,
                                                Box::new(left.to_reduction_lambda_node(false)), Box::new(right_tree)),
                                    right_depth + 1))
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
        vars
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
            Some((redex, depth))
        } else {
            None
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
        (term, counter)
    }

    pub fn reduce(&self, strategy: ReductionStrategy) -> LambdaNode {
        let tree_option = match strategy {
            ReductionStrategy::Normal => self.find_next_redex_normal(),
            ReductionStrategy::Applicative => self.find_next_redex_applicative(),
        };

        match tree_option {
            Some(tree) => tree.0.reduce(),
            None => self.clone(),
        }
    }

    pub fn resolve_vars(&self, builtins: &HashMap<&str, LambdaNode>, variables: &HashMap<String, LambdaNode>)
            -> (LambdaNode, u32, u32) {
        let mut var_subs = 0;
        let variables_borrowed = variables.iter().map(|(k, v)| (k.as_str(), v)).collect();
        let mut tree = self.clone();
        loop {
            let (t, vs) = tree.substitute(&variables_borrowed);
            if vs == 0 { break; }
            tree = t;
            var_subs += vs;
        }

        let mut bi_subs = 0;
        let builtins_borrowed = builtins.iter().map(|(k, v)| (*k, v)).collect();
        loop {
            let (t, bs) = tree.substitute(&builtins_borrowed);
            if bs == 0 { break; }
            tree = t;
            bi_subs += bs;
        }

        (tree, bi_subs, var_subs)
    }

    pub fn substitute(&self, sigma: &HashMap<&str, &LambdaNode>) -> (LambdaNode, u32) {
        let (a, b) = match self {
            LambdaNode::Variable(name) => apply_substitution(name.as_str(), sigma),
            LambdaNode::Abstraction(var_name, term) => {
                let rotten = self.free_variables().iter()
                    .map(|z| apply_substitution(z, sigma).0.free_variables())
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
        };
        (a, b)
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

    pub fn to_debrujin(&self) -> DeBruijnNode {
        self.to_debrujin_helper(&mut HashMap::new())
    }

    fn to_debrujin_helper<'a>(&'a self, map: &mut HashMap<&'a str, u32>) -> DeBruijnNode {
        match self {
            LambdaNode::Variable(name) => {
                if map.contains_key(name.as_str()) {
                    DeBruijnNode::BoundVariable(*map.get(name.as_str()).unwrap())
                } else {
                    DeBruijnNode::FreeVariable(name.clone())
                }
            },
            LambdaNode::Abstraction(var_name, inner) => {
                let next_index = match map.values().max() {
                    Some(ui) => *ui + 1,
                    None => 1,
                };
                map.insert(var_name, next_index);
                let new = DeBruijnNode::Abstraction(Box::new(inner.to_debrujin_helper(map)));
                map.remove(var_name.as_str());
                new
            },
            LambdaNode::Application(left, right) => {
                DeBruijnNode::Application(Box::new(left.to_debrujin_helper(map)),
                                            Box::new(right.to_debrujin_helper(map)))
            },
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            LambdaNode::Variable(name) => name.clone(),
            LambdaNode::Abstraction(var_name, term)
                => format!("{}{} . {}", '\\', var_name, term.to_string()),
            LambdaNode::Application(term1, term2)
                => {
                    let s1 = if let LambdaNode::Variable(name) = &**term1 {
                        name.to_string()
                    } else if let LambdaNode::Application(_, _) = &**term1 {
                        term1.to_string()
                    } else {
                        format!("({})", term1.to_string())
                    };
                    let s2 = if let LambdaNode::Variable(name) = &**term2 {
                        name.to_string()
                    } else {
                        format!("({})", term2.to_string())
                    };
                    format!("{} {}", s1, s2)
                },
        }
    }

    pub fn substitute_var(&self, var_name: &str, replacement: &LambdaNode) -> (LambdaNode, u32) {
        let mut sigma = HashMap::new();
        sigma.insert(var_name, replacement);
        self.substitute(&sigma)
    }
}

impl ReductionLambdaNode {
    fn marked_reduction(&self) -> Option<LambdaNode> {
        match self {
            ReductionLambdaNode::Variable(_) => None,
            ReductionLambdaNode::Abstraction(_, term) => term.marked_reduction(),
            ReductionLambdaNode::Application(mark, left, right) => if *mark {
                Some(LambdaNode::Application(Box::new(left.to_lambda_node()),
                                                Box::new(right.to_lambda_node())))
            } else {
                left.marked_reduction()
                    .or(right.marked_reduction())
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
                    left_inner.to_lambda_node().substitute(&sigma).0
                } else {
                    panic!("Malformed ReductionLambdaNode");
                }
            } else {
                LambdaNode::Application(Box::new(left.reduce()), Box::new(right.reduce()))
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

impl PartialEq for LambdaNode {
    fn eq(&self, other: &Self) -> bool {
        self.to_debrujin() == other.to_debrujin()
    }
}


pub fn match_assignment(s: Span) -> IResult<(String, LambdaNode)> {
    let (rest, name) = map(match_variable_name, |s| s.to_owned())(s)?;
    let (rest, _) = space1(rest)?;
    let (rest, _) = char('=')(rest)?;

    let match_right_hand_side = |s| {
        let (rest, _) = space1(s)?;
        return match_complete_lambda(rest);
    };
    let (rest, term) = with_err(match_right_hand_side(rest), rest,
                            "missing right hand side on assignment".to_owned())?;

    Ok((rest, (name, term)))
}

fn apply_substitution(var: &str, sigma: &HashMap<&str, &LambdaNode>) -> (LambdaNode, u32) {
    match sigma.get(var) {
        Some(term)
            => if LambdaNode::Variable(var.to_string()) == **term {
                ((*term).clone(), 0)
            } else {
                ((*term).clone(), 1)
            },
        None => (LambdaNode::Variable(var.to_owned()), 0),
    }
}

fn fresh_var(old_var: &str, rotten: HashSet<String>) -> String {
    let mut new_var = old_var.to_owned();
    let mut i = 1;

    while rotten.contains(&new_var) && i <= 3 {
        new_var = format!("{}{}", new_var, '\'');
        i += 1;
    }

    while rotten.contains(&new_var) {
        new_var = format!("{}{}'", old_var, i);
        i += 1;
    }
    new_var
}

