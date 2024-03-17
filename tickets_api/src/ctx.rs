#[derive(Clone, Debug)]
pub struct CTX {
    user_id: u64,
}

impl CTX {
    pub fn new(user_id: u64) -> Self {
        Self { user_id }
    }
}

impl CTX {
    pub fn user_id(&self) -> u64 {
        self.user_id
    }
}
