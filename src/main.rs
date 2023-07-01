use clap::Parser;
use interpreter::Interpreter;
use r#macro::Macro;
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

    #[clap(long)]
    print_macros: bool,

    #[clap(long)]
    std: bool,
}

fn main() {
    let args = Args::parse();

    if args.print_macros {
        Macro::print_all();
        return;
    }

    let mut interpreter = Interpreter::new();
    interpreter.set_strategy(args.strategy);
    if args.std {
        interpreter.interpret_std().unwrap();
    }

    if let Some(file) = args.file {
        if let Err(e) = interpreter.interpret_file(file) {
            e.resolve();
        }
    } else {
        interactive::repl(&mut interpreter);
    }
}
