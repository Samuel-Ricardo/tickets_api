use serde::Serialize;

use crate::model::store;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    Store(store::Error),
}

impl From<store::Error> for Error {
    fn from(err: store::Error) -> Self {
        Self::Store(err)
    }
}
