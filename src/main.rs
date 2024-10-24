use clap::Parser;
use environment::StdEnvironment;
use interpreter::Interpreter;
use r#macro::Macro;
use strategy::Strategy;
use std::path::PathBuf;


mod debruijn;
mod environment;
mod error;
mod interactive;
mod interpreter;
mod lambda;
mod parsing;
mod r#macro;
mod stdlib;
mod strategy;
mod typing;

#[cfg(test)]
mod tests;

use environment::Environment;


const DOCS_URL: &str = "https://jzbor.de/lash";


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Evaluate lambda shell file
    #[clap()]
    file: Option<PathBuf>,

    /// Set initial strategy
    #[clap(long, value_enum, default_value_t = Strategy::Normal)]
    strategy: Strategy,

    /// Print available macros and exit
    #[clap(long)]
    print_macros: bool,

    /// Disable standard environment and church numerals
    #[clap(short, long)]
    strict: bool,

    /// Open documentation in the browser
    #[clap(long)]
    docs: bool,
}

fn main() {
    let args = Args::parse();

    if args.docs {
        let result = std::process::Command::new("xdg-open")
            .arg(DOCS_URL)
            .spawn();
        match result {
            Ok(_) => (),
            Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
        }
        return;
    } else if args.print_macros {
        Macro::print_all(&mut StdEnvironment::new().stdout()).unwrap();
        return;
    }

    let env = StdEnvironment::new();
    let mut interpreter = Interpreter::new(env);
    interpreter.set_strategy(args.strategy);
    interpreter.set_church_num_enabled(!args.strict);
    if !args.strict {
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
