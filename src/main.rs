use clap::Parser;
use interpreter::Interpreter;
use std::path::PathBuf;


mod error;
mod interactive;
mod interpreter;
mod lambda;
mod parsing;
mod r#macro;
mod strategy;


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap()]
    file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let mut interpreter = Interpreter::new();

    if let Some(file) = args.file {
        if let Err(e) = interpreter.interpret_file(file) {
            e.resolve();
        }
    } else {
        interactive::repl(&mut interpreter);
    }
}
