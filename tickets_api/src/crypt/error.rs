pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    //TODO: implement
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}