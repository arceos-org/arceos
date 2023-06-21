use std::{process::Command, collections::HashMap};
use cmd_parser::is_arceos_crate;
use cmd_builder::build_cargo_tree_cmd;

mod cmd_parser;
mod cmd_builder;
pub use cmd_parser::{parse_cmd, build_loc};

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
}

impl Config {
    pub fn build(no_default: bool, features: Vec::<String>, format: GraphFormat, loc: String) -> Config {
        Config { no_default, format, features, loc }
    }
}

pub fn get_deps_by_crate_name(cfg: &Config) -> String {
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
pub fn generate_deps_path(cfg: &Config, result: &mut String) {
    let deps = get_deps_by_crate_name(cfg);
    let deps_parsed = parse_deps(&deps);
    let dep_root = &deps_parsed[0];

    let mut parsed_crates: Vec<&String> = Vec::new();
    let mut lastest_dep_map: HashMap<i32, &String> = HashMap::new();
    let mut idx: usize = 1;

    lastest_dep_map.insert(0, &dep_root.1);
    while idx < deps_parsed.len() {
        let (level, name) = deps_parsed.get(idx).unwrap();
        if !is_arceos_crate(&name) {
            idx += 1;
            continue;
        }
        *result += &format!("{}-->{}\n", lastest_dep_map[&(level - 1)], name);
        if parsed_crates.contains(&name) {
            let mut skip_idx: usize = idx + 1;
            if skip_idx >= deps_parsed.len() {
                break;
            }
            while deps_parsed.get(skip_idx).unwrap().0 > *level {
                idx += 1;
                skip_idx += 1;
            }
            idx += 1;
        } else {
            parsed_crates.push(&name);
            lastest_dep_map.insert(*level, name);
            idx += 1;
        }
    }
}

pub fn generate_mermaid(config: &Config) -> String {
    let mut result = String::from("");
    generate_deps_path(&config, &mut result);
    "graph TD;\n".to_string() + &result
}
