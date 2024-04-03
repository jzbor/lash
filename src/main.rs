use clap::Parser;
use interpreter::Interpreter;
use r#macro::Macro;
use strategy::Strategy;
use std::path::PathBuf;


mod debruijn;
mod error;
mod interactive;
mod interpreter;
mod lambda;
mod parsing;
mod r#macro;
mod stdlib;
mod strategy;
mod tests;


const DOCS_URL: &str = "https://jzbor.de/lash";


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Evaluate lambda shell file
    #[clap()]
    file: Option<PathBuf>,

    /// Set initial strategy
    #[clap(long, value_enum, default_value_t = Strategy::Applicative)]
    strategy: Strategy,

    /// Print available macros and exit
    #[clap(long)]
    print_macros: bool,

    /// Add std terms to the environment
    #[clap(long)]
    std: bool,

    /// Enable church numerals
    #[clap(long)]
    church_nums: bool,

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
        Macro::print_all();
        return;
    }

    let mut interpreter = Interpreter::new();
    interpreter.set_strategy(args.strategy);
    interpreter.set_church_num_enabled(args.church_nums);
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
