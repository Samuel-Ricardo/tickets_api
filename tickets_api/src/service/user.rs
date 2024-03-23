use crate::{
    base::{self, db::DbBmc},
    ctx::CTX,
    model::{
        error::Result,
        user::{User, UserBy},
        ModelManager,
    },
};

pub struct UserService;

impl DbBmc for UserService {
    const TABLE: &'static str = "user";
}
