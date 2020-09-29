use std::fs;
use serde::{Deserialize};
use serde::de::DeserializeOwned;

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

    from_toml_file::<AppConfig>(filename)
        .and_then(|config|
            if config.version != current_version {
                Err(format!("Configuration layout changed, update the config file and file version to {}", current_version))
            } else {
                Ok(config)
            }
        )
        .expect("")
}

fn from_toml_file<T: DeserializeOwned>(filename: &str) -> Result<T, String> {
    fs::read_to_string(filename)
        .map_err(|_| format!("file `{}` is missing", filename))
        .and_then(|file_content| toml::from_str::<T>(file_content.as_str())
            .map_err(|_| format!("`{}` content is incorrect", filename)))
}
