use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use crate::error::*;
use crate::parsing;
use crate::strategy::Strategy;
use crate::lambda::*;
use crate::stdlib::*;


pub struct Interpreter {
    named_terms: HashMap<String, Rc<NamedTerm>>,
    strategy: Strategy,
}

#[derive(Debug, Clone)]
pub enum InterpreterDirective {
    Echo(String),
    Include(PathBuf),
    Set(String, String),
    UseStd,
}


impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            named_terms: HashMap::new(),
            strategy: Strategy::default()
        }
    }

    fn apply_directive(&mut self, directive: InterpreterDirective) -> LashResult<()> {
        use InterpreterDirective::*;
        match directive {
            Echo(msg) => Ok(println!("{}", msg)),
            Set(key, value) => self.set(&key, &value),
            Include(file) => self.include(file),
            UseStd => self.interpret_std(),
        }
    }

    pub fn include(&mut self, file: PathBuf) -> LashResult<()> {
        self.interpret_file(file)
    }

    pub fn interpret_contents(&mut self, content: &str) -> LashResult<()> {
        use parsing::Statement::*;
        let (rest, statements) = parsing::match_statements(parsing::Span::new(content))?;
        let (rest, _) = parsing::finish(rest)?;
        assert!(rest.is_empty(), "{:?}", rest);

        for statement in statements {
            match statement {
                Assignment(name, term) => {
                    let term = self.process_lambda_term(term)?;
                    self.named_terms.insert(name.clone(), Rc::new(NamedTerm::new(name, term)));
                },
                Comment => {},
                Lambda(term) => { self.process_lambda_term(term)?; },
                Directive(directive) => self.apply_directive(directive)?,
            }
        }

        Ok(())
    }

    pub fn interpret_line(&mut self, line: &str) -> LashResult<parsing::Statement> {
        use parsing::Statement::*;
        let (rest, statement) = parsing::match_statement(parsing::Span::new(line), false)?;
        let (rest, _) = parsing::finish(rest)?;
        assert!(rest.is_empty(), "{:?}", rest);

        match statement.clone() {
            Assignment(name, term) => {
                let term = self.process_lambda_term(term)?;
                self.named_terms.insert(name.clone(), Rc::new(NamedTerm::new(name.clone(), term.clone())));
                Ok(Assignment(name, term))
            },
            Comment => Ok(Comment),
            Lambda(term) => Ok(Lambda(self.process_lambda_term(term)?)),
            Directive(directive) => { self.apply_directive(directive)?; Ok(statement) },
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
        let contents = fs::read_to_string(&file)
            .map_err(|e| LashError::new_file_error(file, Some(e)))?;
        self.interpret_contents(&contents)
    }

    pub fn set(&mut self, key: &str, value: &str) -> LashResult<()> {
        match key {
            "strategy" => match clap::ValueEnum::from_str(value, false).ok() {
                Some(strat) => self.set_strategy(strat),
                None => return Err(LashError::new_set_value_error(value)),
            },
            _ => return Err(LashError::new_set_key_error(key)),
        }
        Ok(())
    }

    pub fn set_strategy(&mut self, strategy: Strategy) {
        self.strategy = strategy;
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }
}


impl fmt::Display for InterpreterDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InterpreterDirective::*;
        match self {
            Echo(msg) => write!(f, "@echo \"{}\"", msg),
            Set(key, value) => write!(f, "@set {} {}", key, value),
            Include(file) => write!(f, "@include \"{}\"", file.to_string_lossy()),
            UseStd => write!(f, "@usestd"),
        }
    }
}
