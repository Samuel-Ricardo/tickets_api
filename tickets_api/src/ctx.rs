use crate::Error;
use crate::Result;

#[derive(Clone, Debug)]
pub struct CTX {
    user_id: u64,
}

impl CTX {
    pub fn new(user_id: u64) -> Result<Self> {
        if user_id == 0 {
            Err(Error::CannotNewRootCtx)
        } else {
            Ok(Self { user_id })
        }
    }

    pub fn root_ctx() -> Self {
        CTX { user_id: 0 }
    }
}

impl CTX {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}
