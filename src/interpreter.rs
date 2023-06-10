use std::rc::Rc;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::*;
use crate::parsing;
use crate::parsing::Statement;
use crate::strategy::Strategy;
use crate::lambda::*;
use crate::stdlib::*;


pub struct Interpreter {
    named_terms: HashMap<String, Rc<NamedTerm>>,
    strategy: Strategy,
}


impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            named_terms: HashMap::new(),
            strategy: Strategy::default()
        }
    }

    pub fn interpret_contents(&mut self, content: &str) -> LashResult<()> {
        use parsing::Statement::*;
        let (rest, statements) = parsing::match_statements(parsing::Span::new(&content))?;
        let (rest, _) = parsing::finish(rest)?;
        assert!(rest.is_empty(), "{:?}", rest);

        for statement in statements {
            match statement {
                Assignment(name, term) => {
                    let term = self.process_lambda_term(term)?;
                    self.named_terms.insert(name.clone(), Rc::new(NamedTerm::new(name, term)));
                },
                Lambda(term) => { self.process_lambda_term(term)?; },
            }
        }

        Ok(())
    }

    pub fn interpret_line(&mut self, line: &str) -> LashResult<parsing::Statement> {
        use parsing::Statement::*;
        let (rest, statement) = parsing::match_statement(parsing::Span::new(&line), false)?;
        let (rest, _) = parsing::finish(rest)?;
        assert!(rest.is_empty(), "{:?}", rest);

        match statement.clone() {
            Assignment(name, term) => {
                let term = self.process_lambda_term(term)?;
                self.named_terms.insert(name.clone(), Rc::new(NamedTerm::new(name.clone(), term.clone())));
                Ok(Statement::Assignment(name, term))
            },
            Lambda(term) => Ok(Statement::Lambda(self.process_lambda_term(term)?)),
        }
    }

    pub fn interpret_std(&mut self) -> LashResult<()> {
        self.interpret_contents(STD)
    }

    fn process_lambda_term(&self, term: LambdaTree) -> LashResult<LambdaTree> {
        let with_named = term.set_named_terms(&self.named_terms);
        let with_macros = with_named.apply_macros(self)?;
        Ok(with_macros)
    }

    pub fn interpret_file(&mut self, file: PathBuf) -> LashResult<()> {
        let contents = fs::read_to_string(file).unwrap();
        self.interpret_contents(&contents)
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    pub fn set_strategy(&mut self, strategy: Strategy) {
        self.strategy = strategy;
    }
}

