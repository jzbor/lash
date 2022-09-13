use std::collections::HashMap;

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
];

pub fn get_builtins() -> HashMap<&'static str, &'static str> {
    let mut hash_map = HashMap::new();
    for (k, v) in BUILTINS {
        hash_map.insert(k.to_owned(), v.to_owned());
    }
    return hash_map;
}

