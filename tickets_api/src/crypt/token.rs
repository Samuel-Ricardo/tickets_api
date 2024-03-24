use std::fmt::Display;
use std::str::FromStr;

use crate::config;
use crate::util::base64::{b64u_decode, b64u_encode};
use crate::{Error, Result};

pub struct Token {
    pub ident: String,
    pub exp: String,
    pub sign_b64u: String,
}

fn _generate_token(ident: &str, duration_sec: f64, salt: &str, key: &[u8]) -> Result<Token> {
    todo!()
}

fn _validate_token_sign_and_exp(origin_token: &str, salt: &str, key: &[u8]) -> Result<Token> {
    todo!()
}

fn _token_sign_into_b64url(ident: &str, exp: &str, salt: &str, key: &[u8]) -> Result<String> {
    todo!()
}

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
    let config = &config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(origin_token: &Token, salt: &str) -> Result<()> {
    let config = &config();
    _validate_token_sign_and_exp(&origin_token.ident, salt, &config.TOKEN_KEY)?;

    Ok(())
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(token_str: &str) -> std::result::Result<Self, Self::Err> {
        let splits: Vec<&str> = token_str.split(".").collect();

        if splits.len() != 3 {
            return Err(Error::TokenInvalidFormat);
        }

        let (ident, exp, sign) = (splits[0], splits[1], splits[2]);

        Ok(Self {
            ident: b64u_decode(ident).map_err(|_| Error::TokenCannotDecodeIdent)?,
            exp: b64u_decode(exp).map_err(|_| Error::TokenCannotDecodeExp)?,
            sign_b64u: sign.to_string(),
        })
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            b64u_encode(&self.ident),
            b64u_encode(&self.exp),
            &self.sign_b64u
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_generate_web_token() -> Result<()> {
        let fx_token = Token {
            ident: "fx-ident-01".to_string(),
            exp: "2024-03-17T15:30:00Z".to_string(),
            sign_b64u: "some_sign_b64u_encoded".to_string(),
        };

        println!("->> {fx_token}");
        Ok(())
    }
}
