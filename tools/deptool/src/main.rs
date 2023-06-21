use deptool::{generate_mermaid, parse_cmd};
use std::process;

fn main() {
    let config = parse_cmd().unwrap_or_else(|err| {
        eprintln!("problem parsinig arguments: {err}");
        process::exit(1);
    });

    let rst = generate_mermaid(&config);
    print!("{}", rst);
}
