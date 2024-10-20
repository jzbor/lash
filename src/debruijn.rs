use std::{collections::HashMap, fmt::Display};

use crate::lambda::{LambdaNode, LambdaTree};


#[derive(Debug,Clone,Eq,PartialEq)]
pub enum DeBruijnNode {
    BoundVariable(usize),
    FreeVariable(String),
    Abstraction(Box<DeBruijnNode>),
    Application(Box<DeBruijnNode>, Box<DeBruijnNode>),
}

fn to_debrujin_helper(l: LambdaTree, map: &mut HashMap<String, usize>, depth: usize) -> DeBruijnNode {
    match l.node() {
        LambdaNode::Variable(name) => {
            if map.contains_key(name.as_str()) {
                DeBruijnNode::BoundVariable(depth - *map.get(name.as_str()).unwrap())
            } else {
                DeBruijnNode::FreeVariable(name.clone())
            }
        },
        LambdaNode::Abstraction(var_name, inner) => {
            map.insert(var_name.to_owned(), depth);
            let new = DeBruijnNode::Abstraction(Box::new(to_debrujin_helper(inner.clone(), map, depth + 1)));
            map.remove(var_name.as_str());
            new
        },
        LambdaNode::Application(left, right) => {
            DeBruijnNode::Application(Box::new(to_debrujin_helper(left.clone(), map, depth)),
                Box::new(to_debrujin_helper(right.clone(), map, depth)))
        },
        LambdaNode::Named(named_term) => {
            let term = named_term.term();
            to_debrujin_helper(term.clone(), map, depth)
        },
        LambdaNode::ChurchNum(n) => to_debrujin_helper(LambdaTree::unwrap_church_num(*n), map, depth),
        LambdaNode::Macro(_, _) => unreachable!(),
    }

}

impl From<LambdaTree> for DeBruijnNode {
    fn from(value: LambdaTree) -> Self {
        to_debrujin_helper(value, &mut HashMap::new(), 0)
    }
}

impl Display for DeBruijnNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DeBruijnNode::*;
        match self {
            FreeVariable(name) => write!(f, "{}", name),
            BoundVariable(i) => write!(f, "{}", i),
            Abstraction(term) => write!(f, "\\ {}", term),
            Application(term1, term2) => {
                let s1 = if let DeBruijnNode::FreeVariable(name) = &**term1 {
                    name.to_string()
                } else if let DeBruijnNode::BoundVariable(i) = &**term1 {
                    format!("{}", i)
                } else if let DeBruijnNode::Application(_, _) = &**term1 {
                    term1.to_string()
                } else {
                    format!("({})", term1)
                };
                let s2 = if let DeBruijnNode::FreeVariable(name) = &**term2 {
                    name.to_string()
                } else if let DeBruijnNode::BoundVariable(i) = &**term2 {
                    format!("{}", i)
                } else {
                    format!("({})", term2)
                };
                write!(f, "{} {}", s1, s2)
            },
        }
    }
}
