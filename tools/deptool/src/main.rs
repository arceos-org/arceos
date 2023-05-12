// use std::process::Command;
use std::env;
use deptool::{Config, generate_mermaid};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("problem parsinig arguments: {err}");
        process::exit(1);
    });

    let rst = generate_mermaid(&config);
    println!("{}", rst);
}
