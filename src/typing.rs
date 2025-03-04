extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::cell::RefCell;
use core::cmp;
use core::fmt::Display;

use crate::lambda::{LambdaNode, LambdaTree};

type VarName = String;
type Level = usize;
type Environment = BTreeMap<VarName, Type>;


#[derive(PartialEq, Clone, Debug)]
pub enum TypeVariable {
    Unbound(String, Level),
    Link(Box<Type>),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Type {
    TypeVar(Rc<RefCell<TypeVariable>>),
    TypeArrow(Box<Type>, Box<Type>),
}

struct TypeMachine {
    gensym_counter: usize,
    current_level: usize,
}

impl TypeMachine {
    pub fn new() -> Self {
        TypeMachine {
            gensym_counter: 0,
            current_level: 1,
        }
    }

    fn gensym(&mut self) -> String {
        let nletters = 'Z' as usize - 'A' as usize + 1;
        let n = self.gensym_counter;
        self.gensym_counter += 1;
        if n < nletters {
            format!("{}", ('A' as usize + n) as u8 as char)
        } else {
            format!("T{}", n)
        }
    }

    fn newvar(&mut self) -> Type {
        use Type::*;
        use TypeVariable::*;
        TypeVar(Rc::new(RefCell::new(Unbound(self.gensym(), self.current_level))))
    }

    fn occurs(tv: &Rc<RefCell<TypeVariable>>, t: &Type) -> Result<(), String> {
        use Type::*;
        use TypeVariable::*;
        match t {
            TypeVar(tv2) => if tv == tv2 {
                Err(format!("occurs check failed - {} == {}", tv.borrow(), tv2.borrow()))
            } else if let Unbound(name, level) = tv2.borrow().clone() {
                let min_level = match &*tv.borrow() {
                    Unbound(_, l) => cmp::min(level, *l),
                    _ => level,
                };
                // *tv2.borrow_mut() = Unbound(name.to_owned(), min_level);
                unsafe { *tv2.as_ptr() = Unbound(name.to_string(), min_level); }
                Ok(())
            } else if let Link(ty) = &*tv2.borrow() {
                Self::occurs(tv, ty)
            } else {
                Err(format!("occurs check failed - {} versus {}", tv.borrow(), tv2.borrow()))
            },
            TypeArrow(t1, t2) => { Self::occurs(tv, t1)?;Self::occurs(tv, t2) },
        }
    }

    fn unify(t1: &Type, t2: &Type) -> Result<(), String> {
        use Type::*;
        use TypeVariable::*;

        let is_unbound = |tv: &RefCell<_>| matches!(*tv.borrow(), Unbound(..));

        if t1 == t2 { return Ok(()); }

        if let TypeVar(tv) = t1 {
            if let Link(tv) = &*tv.borrow() {
                return Self::unify(&tv, t2);
            }
        }

        if let TypeVar(tv) = t2 {
            if let Link(tv) = &*tv.borrow() {
                return Self::unify(t1, &tv);
            }
        }


        if let TypeVar(tv) = t1 {
            if is_unbound(tv) {
                Self::occurs(tv, t2)?;
                *tv.borrow_mut() = Link(Box::new(t2.clone()));
                return Ok(());
            }
        }

        if let TypeVar(tv) = t2 {
            if is_unbound(tv) {
                Self::occurs(tv, t1)?;
                *tv.borrow_mut() = Link(Box::new(t1.clone()));
                return Ok(());
            }
        }

        if let TypeArrow(tyl1, tyl2) = t1 {
            if let TypeArrow(tyr1, tyr2) = t2 {
                Self::unify(tyl1, tyr1)?;
                return Self::unify(tyl2, tyr2);
            }
        }

        Err(format!("unable to unify {} with {}", t1, t2))
    }

    fn inst(&mut self, t: Type) -> Type {
        Self::inst_rec(t, BTreeMap::new()).0
    }

    fn inst_rec(t: Type, subst: BTreeMap<String, Type>) -> (Type, BTreeMap<String, Type>) {
        use Type::*;
        use TypeVariable::*;

        match t {
            TypeVar(ref tv) => match &*tv.borrow() {
                Link(ty) => Self::inst_rec(*ty.clone(), subst),
                Unbound(..) => (t.clone(), subst),
            },
            TypeArrow(ty1, ty2) => {
                let (ty1, subst) = Self::inst_rec(*ty1, subst);
                let (ty2, subst) = Self::inst_rec(*ty2, subst);
                (TypeArrow(Box::new(ty1), Box::new(ty2)), subst)
            },
        }
    }

    fn infer(&mut self, env: &Environment, lambda: LambdaTree) -> Result<Type, String> {
        use LambdaNode::*;
        use Type::*;
        match lambda.node() {
            Variable(x) => Ok(self.inst(env.get(x).ok_or(format!("unable to instantiate {}", x))?.clone())),
            Abstraction(x, e) => {
                let ty_x = self.newvar();
                let mut nextenv = env.clone();
                nextenv.insert(x.to_string(), ty_x.clone());
                let ty_e = self.infer(&nextenv, e.clone())?;
                Ok(TypeArrow(Box::new(ty_x), Box::new(ty_e)))
            },
            Application(e1, e2) => {
                let ty_fun = self.infer(env, e1.clone())?;
                let ty_arg = Box::new(self.infer(env, e2.clone())?);
                let ty_res = Box::new(self.newvar());
                Self::unify(&ty_fun, &TypeArrow(ty_arg, ty_res.clone()))?;
                Ok(*ty_res)
            },
            Named(n) => self.infer(env, n.term()),
            ChurchNum(n) => self.infer(env, LambdaTree::unwrap_church_num(*n)),
            Macro(_, _) => unreachable!(),
        }
    }
}

pub fn infer(lambda: LambdaTree) -> Result<Type, String> {
    TypeMachine::new().infer(&Environment::new(), lambda)
}

impl Display for Type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Type::*;
        match self {
            TypeVar(var) => write!(f, "{}", var.borrow()),
            TypeArrow(ty1, ty2) => write!(f, "({} -> {})", ty1, ty2),
        }
    }
}

impl Display for TypeVariable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use TypeVariable::*;
        match self {
            Unbound(u, _) => write!(f, "{}", u),
            Link(l) => write!(f, "{}", l),
        }
    }
}
