use std::fs;
use serde::{Deserialize};

#[derive(Deserialize)]
#[derive(Debug)]
pub struct AppConfig {
    pub version: String,
    pub mapping: Mapping
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Mapping {
    pub config: Vec<MappingConfig>
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct MappingConfig {
    pub source: String,
    pub id: String,
    pub actions: Vec<String>,
    #[serde(default)]
    pub category: String
}

pub fn app_config() -> AppConfig {
    let filename = "config.toml";
    let current_version = "1.0";

    fs::read_to_string(filename)
        .map_err(|_| format!("file `{}` is missing", filename))
        .and_then(|file_content| toml::from_str::<AppConfig>(file_content.as_str())
            .map_err(|_| format!("`{}` content is incorrect", filename)))
        .and_then(|config|
            if config.version != current_version {
                Err(String::from("Configuration layout changed, update the config file"))
            } else {
                Ok(config)
            }
        )
        .expect("")
}
