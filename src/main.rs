use clap::Parser;
use std::rc::Rc;
use interpreter::Interpreter;
use std::path::PathBuf;


mod context;
mod interpreter;
mod lambda;
mod r#macro;
mod parsing;
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
        interpreter.parse_file(file);
    } else {
        todo!("interactive mode")
    }
}
