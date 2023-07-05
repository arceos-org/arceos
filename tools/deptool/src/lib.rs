mod cmd_parser;
mod cmd_builder;
mod mermaid_generator;
mod d2_generator;

use std::process::Command;
use std::fs::File;
use std::io::Write;

use cmd_builder::build_cargo_tree_cmd;
pub use cmd_parser::{parse_cmd, build_loc};
use d2_generator::gen_d2_script;
use mermaid_generator::gen_mermaid_script;

#[derive(Clone, Copy, Debug)]
pub enum GraphFormat {
   Mermaid,
   D2,
}

#[derive(Debug)]
pub struct Config {
    pub no_default: bool,
    pub format: GraphFormat,
    pub features: Vec::<String>,
    loc: String,
    output_loc: String
}

impl Config {
    pub fn build(no_default: bool, features: Vec::<String>, format: GraphFormat, loc: String, output_loc: String) -> Config {
        Config { no_default, format, features, loc, output_loc }
    }
}

fn get_deps_by_crate_name(cfg: &Config) -> String {
    let cmd_ct = build_cargo_tree_cmd(&cfg);
    let cmds = ["-c", &cmd_ct];
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(cmds)
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("sh")
                .args(cmds)
                .output()
                .expect("failed to execute process")
    };

    let deps = output.stdout;
    String::from_utf8(deps).unwrap()
}

fn parse_deps(deps: &String) -> Vec<(i32, String)> {
    let mut rst = vec!();
    for line in deps.lines() {
        let level_name = line.split_whitespace().next().unwrap();
        let level = level_name.get(0..1).unwrap().parse().unwrap();
        let name = level_name.get(1..).unwrap();
        rst.push((level, name.to_string()));
    }
    rst
}

fn generate_mermaid(config: &Config) -> String {
    let mut result = String::from("");
    let deps = get_deps_by_crate_name(config);
    gen_mermaid_script(&deps, &mut result);
    "graph TD;\n".to_string() + &result
}

fn generate_d2(config: &Config) -> String {
    let mut result = String::from("");
    let deps = get_deps_by_crate_name(config);
    gen_d2_script(&deps, &mut result);
    result
}

fn generate_deps_graph(config: &Config) -> String {
    match config.format {
        GraphFormat::D2 => generate_d2(config),
        _ => generate_mermaid(config)
    }
}

fn output_deps_graph(rst: &String) -> std::io::Result<()> {
    let mut file = File::create("output.txt")?;
    file.write_all(rst.as_bytes())?;
    Ok(())
}

pub fn run(config: &Config) {
    let rst = generate_deps_graph(config);
    print!("{}", rst);
    match output_deps_graph(&rst) {
        Ok(()) => {},
        Err(error) => println!("Error during writing file {}, {}", config.output_loc, error)
    }
}
