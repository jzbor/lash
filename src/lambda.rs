extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use core::fmt::Display;
use core::str;

use crate::debruijn::DeBruijnNode;
use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::r#macro::Macro;
use crate::error::LashResult;


#[derive(Clone, Debug)]
pub enum LambdaNode {
    Abstraction(String, LambdaTree),
    Application(LambdaTree, LambdaTree),
    Macro(Macro, Vec<LambdaTree>),
    Named(Rc<NamedTerm>),
    Variable(String),
    ChurchNum(u32),
}

#[derive(Clone, Debug)]
pub struct NamedTerm {
    name: String,
    term: LambdaTree,
}

#[derive(Clone, Debug)]
pub struct LambdaTree(Rc<LambdaNode>);

impl NamedTerm {
    pub fn new(name: String, term: LambdaTree) -> Self {
        NamedTerm { name, term }
    }

    pub fn term(&self) -> LambdaTree {
        self.term.clone()
    }
}

impl LambdaTree {
    pub fn alpha_eq(&self, other: &LambdaTree) -> bool {
        DeBruijnNode::from(self.clone()) == DeBruijnNode::from(other.clone())
    }

    pub fn new_abstraction(var: String, term: Self) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Abstraction(var, term)))
    }

    pub fn new_application(left_term: Self, right_term: Self) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Application(left_term, right_term)))
    }

    pub fn new_church_num(denominator: u32) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(ChurchNum(denominator)))
    }

    pub fn new_macro(m: Macro, terms: Vec<Self>) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Macro(m, terms)))
    }

    pub fn new_variable(name: String) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Variable(name)))
    }

    pub fn apply_macros<E: Environment>(&self, interpreter: &mut Interpreter<E>) -> LashResult<Self> {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var, term) => Ok(Self::new_abstraction(var.to_string(), term.apply_macros(interpreter)?)),
            Application(left_term, right_term)
                => Ok(Self::new_application(left_term.apply_macros(interpreter)?, right_term.apply_macros(interpreter)?)),
            Variable(_) => Ok(self.clone()),
            Macro(m, terms) => {
                let time_start = interpreter.env().now();
                let terms = terms.iter().map(|m| m.apply_macros(interpreter)).collect::<Vec<LashResult<LambdaTree>>>();
                let duration = interpreter.env().elapsed(time_start);
                if let Some(e) = terms.iter().find(|r| r.is_err()) {
                    e.clone()
                } else {
                    let terms = terms.iter().flatten().cloned().collect();
                    m.apply(interpreter, terms, duration)
                }
            } ,
            Named(_) | ChurchNum(_) => Ok(self.clone()),
        }
    }

    fn contains_free_variable(&self, variable: &str) -> bool {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var, term) => if var == variable { false } else { term.contains_free_variable(variable) },
            Application(left_term, right_term) => left_term.contains_free_variable(variable) || right_term.contains_free_variable(variable),
            Variable(var) => var == variable,
            Macro(_, terms) => terms.iter().any(|t| t.contains_free_variable(variable)),
            Named(named) => named.term().contains_free_variable(variable),
            ChurchNum(_) => false,
        }
    }

    pub fn fmt_with_parenthesis(&self, left_of_appl: bool) -> String {
        if self.needs_parenthesis(left_of_appl) {
            format!("({})", self)
        } else {
            format!("{}", self)
        }
    }

    pub fn has_church_nums(&self) -> bool {
        use LambdaNode::*;
        match self.node() {
            Abstraction(_, term) => term.has_church_nums(),
            Application(left_term, right_term) => left_term.has_church_nums() || right_term.has_church_nums(),
            Variable(_) => false,
            Macro(_, terms) => terms.iter().any(|t| t.has_church_nums()),
            Named(named) => named.term().has_church_nums(),
            ChurchNum(_) => true,
        }
    }

    pub fn is_abstraction(&self) -> bool {
        use LambdaNode::*;
        matches!(self.node(), Abstraction(..))
    }

    pub fn is_application(&self) -> bool {
        use LambdaNode::*;
        matches!(self.node(), Application(..))
    }

    pub fn is_church_num(&self) -> bool {
        use LambdaNode::*;
        matches!(self.node(), ChurchNum(..))
    }

    pub fn is_named(&self) -> bool {
        use LambdaNode::*;
        matches!(self.node(), Named(..))
    }

    pub fn is_variable(&self) -> bool {
        use LambdaNode::*;
        matches!(self.node(), Variable(..))
    }

    pub fn needs_parenthesis(&self, left_of_appl: bool) -> bool {
        !(self.is_named() || self.is_variable() || self.is_church_num() || (left_of_appl && self.is_application()))
    }

    pub fn node(&self) -> &LambdaNode {
        let LambdaTree(item) = self;
        item.as_ref()
    }

    pub fn set_named_terms(&self, named_terms: &BTreeMap<String, Rc<NamedTerm>>) -> Self {
        self.set_named_terms_helper(named_terms, &mut Vec::new())
    }

    fn set_named_terms_helper(&self, named_terms: &BTreeMap<String, Rc<NamedTerm>>, bound_vars: &mut Vec<String>) -> Self {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var, term) => {
                bound_vars.push(var.to_owned());
                let new_term = term.set_named_terms_helper(named_terms, bound_vars);
                bound_vars.pop();
                LambdaTree(Rc::new(Abstraction(var.clone(), new_term)))
            },
            Application(left_term, right_term) => {
                let new_left_term = left_term.set_named_terms_helper(named_terms, bound_vars);
                let new_right_term = right_term.set_named_terms_helper(named_terms, bound_vars);
                LambdaTree(Rc::new(Application(new_left_term, new_right_term)))
            },
            Variable(name) => {
                if bound_vars.contains(name) {
                    self.clone()
                } else if let Some(named) = named_terms.get(name) {
                    LambdaTree(Rc::new(Named(named.clone())))
                } else {
                    self.clone()
                }
            },
            Macro(m, terms) => Self::new_macro(*m, terms.iter()
                                               .map(|t| t.set_named_terms_helper(named_terms, bound_vars)).collect()),
            Named(_) | ChurchNum(_) => self.clone(),
        }
    }

    /// Resolve all named terms
    pub fn resolve(&self) -> Self {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var, inner_term) => Self::new_abstraction(var.clone(), inner_term.resolve()),
            Application(left_term, right_term) => Self::new_application(left_term.resolve(), right_term.resolve()),
            Macro(m, terms) => Self::new_macro(*m, terms.iter().map(|t| t.resolve()).collect()),
            Variable(_) => self.clone(),
            Named(term) => term.term().resolve(),
            ChurchNum(d) => Self::unwrap_church_num(*d),
        }
    }

    pub fn substitute(&self, name: &str, term: LambdaTree) -> Self {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var, inner_term) => {
                if var == name {
                    self.clone()
                } else {
                    // avoid capturing free variables
                    if term.contains_free_variable(var) {
                        let mut fresh_var = var.to_owned();
                        while term.contains_free_variable(&fresh_var) {
                            fresh_var = format!("{}'", fresh_var);
                        }
                        let new_inner = inner_term
                            .substitute(var, Self::new_variable(fresh_var.clone()))
                            .substitute(name, term.clone());
                        Self::new_abstraction(fresh_var, new_inner)
                    } else {
                        Self::new_abstraction(var.clone(), inner_term.substitute(name, term.clone()))
                    }
                }
            },
            Application(left_term, right_term) => {
                let new_left_term = left_term.substitute(name, term.clone());
                let new_right_term = right_term.substitute(name, term);
                LambdaTree(Rc::new(Application(new_left_term, new_right_term)))
            },
            Variable(var_name) => {
                if var_name == name {
                    term.clone()
                } else {
                    self.clone()
                }
            },
            Macro(m, terms) => Self::new_macro(*m, terms.iter().map(|t| t.substitute(name, term.clone())).collect()),
            Named(_) | ChurchNum(_) => self.clone(),
        }
    }

    pub fn unwrap_church_num(denominator: u32) ->  Self {
        let mut inner = Self::new_variable("x".to_string());
        for _ in 0..denominator {
            inner = Self::new_application(Self::new_variable("f".to_string()), inner);
        }
        Self::new_abstraction("f".to_owned(), Self::new_abstraction("x".to_string(), inner))
    }
}

impl Display for LambdaTree {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var_name, term) => {
                write!(f, "\\{} . ", var_name)?;
                term.fmt(f)
            },
            Application(term1, term2) => {
                write!(f, "{} {}", term1.fmt_with_parenthesis(true), term2.fmt_with_parenthesis(false))
            },
            Variable(name) => write!(f, "{}", name),
            Macro(m, terms) => {
                write!(f, "!{} ", m)?;
                for term in terms {
                    if let Variable(name) = term.node() {
                        write!(f, "{}", name)?;
                    } else {
                        write!(f, "({})", term)?;
                    }
                }
                Ok(())
            }
            Named(named) => write!(f, "{}", named.name),
            ChurchNum(d) => write!(f, "${}", d),
        }

    }
}
