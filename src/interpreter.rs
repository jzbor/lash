extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::String;
use core::fmt;
use core::fmt::Write;
use core::str;

use crate::error::*;
use crate::environment::*;
use crate::parsing;
use crate::strategy::Strategy;
use crate::lambda::*;
use crate::stdlib::*;


pub struct Interpreter<E: Environment> {
    church_num_enabled: bool,
    named_terms: BTreeMap<String, Rc<NamedTerm>>,
    strategy: Strategy,
    env: E,
}

#[derive(Debug, Clone)]
pub enum InterpreterDirective {
    Echo(String),
    Include(String),
    Set(String, String),
    UseStd,
}


impl<E: Environment> Interpreter<E> {
    pub fn new(env: E) -> Interpreter<E> {
        Interpreter {
            church_num_enabled: false,
            named_terms: BTreeMap::new(),
            strategy: Strategy::default(),
            env,
        }
    }

    fn apply_directive(&mut self, directive: InterpreterDirective) -> LashResult<()> {
        use InterpreterDirective::*;
        match directive {
            Echo(msg) => { Ok(writeln!(self.env.stdout(), "{}", msg)?) },
            Set(key, value) => self.set(&key, &value),
            Include(file) => self.include(file),
            UseStd => self.interpret_std(),
        }
    }

    pub fn include(&mut self, file: String) -> LashResult<()> {
        #[cfg(feature = "std")]
        let result = self.interpret_file(std::path::PathBuf::from(file));
        #[cfg(not(feature = "std"))]
        let result = Err(LashError::new_no_std_error(String::from("cannot open files")));
        #[cfg(not(feature = "std"))]
        let _ = file;

        result
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
        if !self.church_num_enabled && term.has_church_nums() {
            return Err(LashError::new_church_num_error());
        }

        let with_named = term.set_named_terms(&self.named_terms);
        let with_macros = with_named.apply_macros(self)?;
        Ok(with_macros)
    }

    #[cfg(feature = "std")]
    pub fn interpret_file(&mut self, file: std::path::PathBuf) -> LashResult<()> {
        let contents = std::fs::read_to_string(&file)
            .map_err(|e| LashError::new_file_error(file, Some(e)))?;
        self.interpret_contents(&contents)
    }

    pub fn set(&mut self, key: &str, value: &str) -> LashResult<()> {
        match key {
            "strategy" => match Strategy::from_str(value).ok() {
                Some(strat) => self.set_strategy(strat),
                None => return Err(LashError::new_set_value_error(value)),
            },
            "numerals" => match str::parse(value).ok() {
                Some(b) => self.set_church_num_enabled(b),
                None => return Err(LashError::new_set_value_error(value)),
            },
            _ => return Err(LashError::new_set_key_error(key)),
        }
        Ok(())
    }


    pub fn set_church_num_enabled(&mut self, b: bool) {
        self.church_num_enabled = b;
    }

    pub fn set_strategy(&mut self, strategy: Strategy) {
        self.strategy = strategy;
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    pub fn env(&self) -> &E {
        &self.env
    }
}


impl fmt::Display for InterpreterDirective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InterpreterDirective::*;
        match self {
            Echo(msg) => write!(f, "@echo \"{}\"", msg),
            Set(key, value) => write!(f, "@set {} {}", key, value),
            Include(file) => write!(f, "@include \"{}\"", file),
            UseStd => write!(f, "@usestd"),
        }
    }
}
