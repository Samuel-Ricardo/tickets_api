use crate::{error::Error, Result};
use std::{env, str::FromStr, sync::OnceLock, u8};

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
    pub DB_URL: String,
    pub WEB_FOLDER: String,
    // INFO: Crypt
    pub PWD_KEY: Vec<u8>,
    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: f64,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            DB_URL: get_env("SERVICE_DB_URL")?,
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
            PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
        })
    }
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    get_env(name)?
        .parse::<T>()
        .map_err(|_| Error::ConfigWrongFormat(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    base64_url::decode(&get_env(name)?).map_err(|_| Error::ConfigWrongFormat(name))
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}
