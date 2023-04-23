use std::rc::Rc;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::parsing;
use crate::strategy;
use crate::strategy::Strategy;
use crate::lambda::*;


pub struct Interpreter {
    named_terms: HashMap<String, Rc<NamedTerm>>,
    strategy: Box<dyn Strategy>,
}


impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            named_terms: HashMap::new(),
            strategy: Box::new(strategy::default_strategy())
        }
    }

    pub fn parse_contents(&mut self, content: &str) {
        use parsing::Statement::*;
        let (rest, statements) = parsing::match_statements(parsing::Span::new(&content))
            .unwrap();
        assert!(rest.is_empty());

        for statement in statements {
            match statement {
                Assignment(name, term) => {
                    let term = self.process_lambda_term(term);
                    self.named_terms.insert(name.clone(), Rc::new(NamedTerm::new(name, term)));
                },
                Lambda(term) => { self.process_lambda_term(term); },
            }
        }
    }

    fn process_lambda_term(&self, term: LambdaTree) -> LambdaTree {
        let with_named = term.set_named_terms(&self.named_terms);
        let with_macros = with_named.apply_macros(self);
        with_macros
    }

    pub fn parse_file(&mut self, file: PathBuf) {
        let contents = fs::read_to_string(file).unwrap();
        self.parse_contents(&contents);
    }

    pub fn strategy(&self) -> &dyn Strategy {
        self.strategy.as_ref()
    }
}

