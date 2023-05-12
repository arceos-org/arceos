use std::fs;
use std::path::Path;
use std::process::Command;

static CRATE_ROOT: &str = "../../crates/";
static MODULE_ROOT: &str = "../../modules/";

pub struct Config {
    pub crate_name: String,
    loc: &'static str,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str>{
        if args.len() < 2 {
            return Err("not enough arguments");
        }
        let crate_name = args[1].clone();
        if check_crate_name(&crate_name) {
            Ok(Config { crate_name, loc: CRATE_ROOT })
        } else if check_module_name(&crate_name) {
           Ok(Config { crate_name, loc: MODULE_ROOT }) 
        } else {
            Err("crate not exist")
        }
    }
}

fn check_crate_name(name: &String) -> bool {
    let crates = fs::read_dir(CRATE_ROOT).unwrap();
    crates.into_iter().map(|p| p.unwrap().file_name()).any(|n| n.to_str().unwrap() == name)
}

fn check_module_name(name: &String) -> bool {
    let crates = fs::read_dir(MODULE_ROOT).unwrap();
    crates.into_iter().map(|p| p.unwrap().file_name()).any(|n| n.to_str().unwrap() == name)
}

fn is_arceos_crate(name: &String) -> bool {
    check_crate_name(&name) || check_module_name(&name)
}

pub fn get_deps_by_crate_name(cfg: &Config) -> String {
    let path_str = cfg.loc.to_string() + &cfg.crate_name;
    let crate_path = Path::new(&path_str);
    let binding = fs::canonicalize(&crate_path).unwrap();
    let abs_path = binding.to_str().unwrap();

    let cmd1 = &(String::from("cd ") + abs_path + " && " + "cargo tree -e normal --prefix depth --format {lib}");
    let cmds = ["-c", cmd1];
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

    let hello = output.stdout;
    String::from_utf8(hello).unwrap()
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

pub fn generate_deps_path(cfg: &Config, parsed_crates: &mut Vec<String>, result: &mut String) {
    let deps = get_deps_by_crate_name(cfg);
    let deps_parsed = parse_deps(&deps);
    let dep_root = &deps_parsed[0].1;
    let root_level = deps_parsed[0].0;
    for (level, crate_name) in deps_parsed.iter().skip(1) {
        if !is_arceos_crate(&crate_name) {
            continue;
        } else {
            if *level != root_level + 1 {
                continue;
            }
            // println!("{}-->{}", dep_root, crate_name);
            *result += &format!("{}-->{}\n", dep_root, crate_name);
            if parsed_crates.contains(&crate_name) {
                continue;
            }
            parsed_crates.push(crate_name.clone());
            let loc;
            if check_crate_name(&crate_name) {
                loc = CRATE_ROOT;
            } else {
                loc = MODULE_ROOT;
            }
            let new_cfg = Config {crate_name: (*crate_name).clone(), loc};
            generate_deps_path(&new_cfg, parsed_crates, result);
        }
    }
}

pub fn generate_mermaid(config: &Config) -> String {
    let mut tmp = Vec::new();
    let mut result = String::from("");
    generate_deps_path(&config, &mut tmp, &mut result);
    "graph TD;\n".to_string() + &result
}