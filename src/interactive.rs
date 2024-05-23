use rustyline::Editor;
use rustyline::error::*;

use crate::environment::Environment;
use crate::interpreter::Interpreter;

const PROMPT: &str = "[λ] ";

pub fn repl<E: Environment>(interpreter: &mut Interpreter<E>) {
    let mut rl = Editor::<()>::new().unwrap();

    loop {
        match rl.readline(PROMPT) {
            Ok(input) => {
                rl.add_history_entry(input.as_str());
                match interpreter.interpret_line(&input) {
                    Ok(statement) => println!("{}\n", statement),
                    Err(e) => eprintln!("{}\n", e),
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!();
            },
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Fatal Error: {:?}", err);
                break
            }

        }
    }
}
