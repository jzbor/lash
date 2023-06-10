use clap::Parser;
use interpreter::Interpreter;
use strategy::Strategy;
use std::path::PathBuf;


mod error;
mod interactive;
mod interpreter;
mod lambda;
mod parsing;
mod r#macro;
mod stdlib;
mod strategy;


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap()]
    file: Option<PathBuf>,

    #[clap(long, value_enum, default_value_t = Strategy::Applicative)]
    strategy: Strategy,
}

fn main() {
    let args = Args::parse();
    let mut interpreter = Interpreter::new();
    interpreter.set_strategy(args.strategy);
    interpreter.interpret_std().unwrap();

    if let Some(file) = args.file {
        if let Err(e) = interpreter.interpret_file(file) {
            e.resolve();
        }
    } else {
        interactive::repl(&mut interpreter);
    }
}
