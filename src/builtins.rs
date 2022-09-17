use std::collections::HashMap;
use crate::lambda::*;
use crate::parsing::*;

// Sources:
// https://www8.cs.fau.de/ext/teaching/sose2022/thprog/skript.pdf
// https://en.wikipedia.org/wiki/Lambda_calculus
// https://en.wikipedia.org/wiki/Church_encoding
//
static BUILTINS: &'static [(&str, &str)] = &[
    // standard terms
    ("ID",      "\\x . x"),
    ("S",       "\\x y z . x z (y z)"),
    ("K",       "\\x y . x"),
    ("B",       "\\x y z . x (y z)"),
    ("C",       "\\x y z . x z y"),
    ("W",       "\\x y . x y y"),
    ("OMEGA",   "\\x . x x"),
    // booleans
    ("TRUE",    "\\x y . x"),
    ("FALSE",   "\\x y . y"),
    ("AND",     "\\p q . p q p"),
    ("OR",      "\\p q . p p q"),
    ("NOT",     "\\p . p FALSE TRUE"),
    ("IFTHENELSE", "\\p a b . p a b"),
    // pairs
    ("PAIR",    "\\ x y . \\ z. z x y"),
    ("FIRST",   "\\ p . p TRUE"),
    ("SECOND",  "\\ p . p FALSE"),
    // lists (using right fold)
    ("NIL",     "\\c n . n"),
    ("ISNIL",   "\\l . l (\\h t . FALSE) TRUE"),
    ("CONS",    "\\h t c n . c h (t c n)"),
    ("HEAD",    "\\l . l (\\h t. h) FALSE"),
    ("TAIL",    "\\l c n . l (\\h t g . g h (t c)) (\\t . n) (\\h t . t)"),
    // church numerals
    ("ADD",     "\\m n f x . m f (n f x)"),
    ("SUCC",    "\\n f x . f (n f x)"),
    ("MULT",    "\\m n f x . m (n f) x"),
    ("EXP",     "\\m n . n m"),
    ("PRED",    "\\n f x . n (\\g h . h (g f)) (\\u . x) (\\u . u)"),
    ("SUB",     "\\m n . (n PRED) m"),
];

pub fn get_builtins() -> HashMap<&'static str, LambdaNode> {
    let mut hash_map = HashMap::new();
    for (k, v) in BUILTINS {
        if let Ok((_, tree)) = match_complete_lambda(Span::from(*v)) {
            hash_map.insert(k.to_owned(), tree);
        } else {
            panic!("Builtin '{}' is broken!!!", k);
        }
    }
    return hash_map;
}

