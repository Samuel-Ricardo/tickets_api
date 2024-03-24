use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::{crypt, model::store};

pub type Result<T> = std::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
    Crypt(crypt::Error),
    Store(store::Error),
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),
    EntityNotFound { entity: &'static str, id: i64 },
}

impl From<crypt::Error> for Error {
    fn from(err: crypt::Error) -> Self {
        Self::Crypt(err)
    }
}

impl From<store::Error> for Error {
    fn from(err: store::Error) -> Self {
        Self::Store(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Sqlx(err)
    }
}
