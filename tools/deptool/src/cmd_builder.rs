use crate::Config;

pub fn build_cargo_tree_cmd(cfg: &Config) -> String {
    let default_opt = match cfg.no_default {
        true => "",
        false => "--no-default-features"
    };

    let features_opt = match cfg.features.len() {
        0 => "".to_string(),
        _ => "-F ".to_string() + cfg.features.join(" ").as_str()
    };
    let path = &cfg.loc;
    let cmd_str = format!(
        "cd {path} && cargo tree -e normal,build {default_opt} {features_opt} --format {{p}} --prefix depth",
    );
    cmd_str.to_string()
}
