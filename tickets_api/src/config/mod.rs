use crate::{error::Error, Result};
use std::{env, sync::OnceLock};

pub fn config() -> &'static Config {
    //static is always global
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|ex| {
            panic!("FATAL - WHILE LOADING CONFIG - {ex:?}");
        })
    })
}

#[allow(non_snake_case)]
pub struct Config {
    pub WEB_FOLDER: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}
