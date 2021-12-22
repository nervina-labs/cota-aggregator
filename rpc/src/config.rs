use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{env, fs};

pub const CONFIG_FILE: &str = "aggregator.toml";

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub database: String,
}

pub fn load_config() -> Config {
    let mut project_path = PathBuf::new();
    project_path.push(env::current_dir().expect("Current directory error"));
    let content = {
        let mut config_path = project_path.clone();
        config_path.push(CONFIG_FILE);
        read_config_file(config_path).expect("Read config file error")
    };
    let config: Config = toml::from_slice(content.as_bytes()).expect("parse config");
    config
}

fn read_config_file<P: AsRef<Path> + std::fmt::Debug>(path: P) -> Result<String> {
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(err) => Err(anyhow!(
            "Can't found {:?}, current directory is not a project. error: {:?}",
            path,
            err
        )),
    }
}
