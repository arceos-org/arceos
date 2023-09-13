use std::{fs, path::Path};
use clap::{Arg, ArgAction, Command};
use crate::{Config, GraphFormat};

static APP_ROOT: &str = "../../apps/";
static CRATE_ROOT: &str = "../../crates/";
static MODULE_ROOT: &str = "../../modules/";
static ULIB_ROOT: &str = "../../ulib/";


/// Ex: exe --default=false --format=mermaid --features=f1 f2 f3
pub fn parse_cmd() -> Result<Config, &'static str> {
    let matches = Command::new("Dependency analysis tool for Arceos")
        .version("1.0")
        .author("ctr")
        .about("Generate d2 or mermaid dependency graph for Arceos based on cargo tree")
        .arg(
            Arg::new("no-default").short('d').long("no-default").action(ArgAction::SetFalse)
        )
        .arg(
            Arg::new("features").short('f').long("name").action(ArgAction::Append)
        )
        .arg(
            Arg::new("format").short('o').long("format").default_value("mermaid")
        )
        .arg(
            Arg::new("target").short('t').long("target").required(true)
        )
        .arg(
            Arg::new("save-path").short('s').long("save-path").default_value("out.txt")
        )
        .get_matches();

    let is_default = matches.get_flag("no-default");
    let features = matches.get_many::<String>("features").unwrap_or_default()
        .map(|f| f.to_string())
        .collect();
    let format = match matches.get_one::<String>("format").unwrap().as_str() {
        "d2" => GraphFormat::D2,
        _ => GraphFormat::Mermaid
    };
    let target = matches.get_one::<String>("target").unwrap().to_string();
    if !is_arceos_crate(&target) {
        return Err("target not exist, should be valid arceos crate, module or app");
    }

    let loc;
    if check_crate_name(&target) {
        loc = CRATE_ROOT.to_string() + &target;
    } else if check_module_name(&target) {
        loc = MODULE_ROOT.to_string() + &target;
    } else {
        loc = APP_ROOT.to_string() + &target;
    }
    let output_loc = matches.get_one::<String>("save-path").unwrap().to_string();
    Ok(gen_config(is_default, features, format, loc, output_loc))
}

fn gen_config(is_default: bool, features: Vec::<String>, format: GraphFormat, loc: String, output_loc: String) -> Config {
    Config::build(is_default, features, format, loc, output_loc)
}

pub fn check_crate_name(name: &String) -> bool {
    let crates = fs::read_dir(CRATE_ROOT).unwrap();
    crates.into_iter().map(|p| p.unwrap().file_name()).any(|n| n.to_str().unwrap() == name)
}

pub fn check_module_name(name: &String) -> bool {
    let crates = fs::read_dir(MODULE_ROOT).unwrap();
    crates.into_iter().map(|p| p.unwrap().file_name()).any(|n| n.to_str().unwrap() == name)
}

pub fn check_app_name(name: &String) -> bool {
    Path::new(&(APP_ROOT.to_string() + name)).exists()
}

pub fn check_lib_name(name: &String) -> bool {
    Path::new(&(ULIB_ROOT.to_string() + name)).exists()
}

pub fn is_arceos_crate(name: &String) -> bool {
    check_crate_name(&name) || check_module_name(&name) || check_app_name(name) || check_lib_name(name)
}

pub fn build_loc(name: &String) -> String {
    if check_module_name(name) {
        MODULE_ROOT.to_string() + name
    } else {
        CRATE_ROOT.to_string() + name
    }
}
