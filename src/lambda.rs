use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::interpreter::Interpreter;
use crate::r#macro::Macro;


#[derive(Clone, Debug)]
pub enum LambdaNode {
    Abstraction(String, LambdaTree),
    Application(LambdaTree, LambdaTree),
    Macro(Macro, LambdaTree),
    Named(Rc<NamedTerm>),
    Variable(String),
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn term(&self) -> LambdaTree {
        self.term.clone()
    }
}

impl LambdaTree {
    pub fn new_abstraction(var: String, term: Self) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Abstraction(var, term)))
    }

    pub fn new_application(left_term: Self, right_term: Self) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Application(left_term, right_term)))
    }

    pub fn new_macro(m: Macro, term: Self) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Macro(m, term)))
    }

    pub fn new_variable(name: String) -> Self {
        use LambdaNode::*;
        LambdaTree(Rc::new(Variable(name)))
    }

    pub fn apply_macros(&self, interpreter: &Interpreter) -> Self {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var, term) => Self::new_abstraction(var.to_string(), term.apply_macros(interpreter)),
            Application(left_term, right_term)
                => Self::new_application(left_term.apply_macros(interpreter), right_term.apply_macros(interpreter)),
            Variable(_) => self.clone(),
            Macro(m, term) => m.apply(interpreter, term.apply_macros(interpreter)),
            Named(_) => self.clone(),
        }
    }

    pub fn fmt_with_parenthesis(&self) -> String {
        if self.needs_parenthesis() {
            format!("({})", self)
        } else {
            format!("{}", self)
        }
    }

    pub fn is_abstraction(&self) -> bool {
        use LambdaNode::*;
        if let Abstraction(..) = self.node() { true } else { false }
    }

    pub fn is_application(&self) -> bool {
        use LambdaNode::*;
        if let Application(..) = self.node() { true } else { false }
    }

    pub fn is_macro(&self) -> bool {
        use LambdaNode::*;
        if let Macro(..) = self.node() { true } else { false }
    }

    pub fn is_named(&self) -> bool {
        use LambdaNode::*;
        if let Named(..) = self.node() { true } else { false }
    }

    pub fn is_variable(&self) -> bool {
        use LambdaNode::*;
        if let Variable(..) = self.node() { true } else { false }
    }

    pub fn needs_parenthesis(&self) -> bool {
        !(self.is_named() || self.is_variable())
    }

    pub fn node(&self) -> &LambdaNode {
        let LambdaTree(item) = self;
        item.as_ref()
    }

    pub fn set_named_terms(&self, named_terms: &HashMap<String, Rc<NamedTerm>>) -> Self {
        self.set_named_terms_helper(named_terms, &mut Vec::new())
    }

    fn set_named_terms_helper(&self, named_terms: &HashMap<String, Rc<NamedTerm>>, bound_vars: &mut Vec<String>) -> Self {
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
            Macro(m, term) => Self::new_macro(*m, term.set_named_terms_helper(named_terms, bound_vars)),
            Named(_) => self.clone(),
        }
    }

    pub fn substitute(&self, name: &str, term: LambdaTree) -> Self {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var, inner_term) => {
                if var == name {
                    inner_term.clone()
                } else {
                    Self::new_abstraction(var.clone(), inner_term.substitute(name, term.clone()))
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
            Macro(m, term) => Self::new_macro(*m, term.substitute(name, term.clone())),
            Named(_) => self.clone(),
        }
    }
}

impl Display for LambdaTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LambdaNode::*;
        match self.node() {
            Abstraction(var_name, term) => {
                write!(f, "{}{} . ", '\\', var_name)?;
                term.fmt(f)
            },
            Application(term1, term2) => {
                write!(f, "{} {}", term1.fmt_with_parenthesis(), term2.fmt_with_parenthesis())
            },
            Variable(name) => write!(f, "{}", name),
            Macro(m, term) => {
                write!(f, "!{} ", m)?;
                if let Variable(name) = term.node() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "({})", term)
                }
            }
            Named(named) => write!(f, "{}", named.name),
        }

    }
}
