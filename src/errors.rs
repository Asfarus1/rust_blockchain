use std::time::SystemTimeError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chain is empty")]
    ChainIsEmpty,
    #[error("Block with index {0} has invalid hash {1}")]
    BlockHasInvalidHash(u64, String),
    #[error("Block with index {0} has invalid previous block hash: actual: '{1}', expected: '{2}'")]
    BlockHasInvalidPreviusBlockHash(u64, String, String),
    #[error("Block with index {0} doesn't satisfy difficulty '{1}'")]
    UnsatisfiedHashDifficulty(u64, String),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + 'static>),
}

impl From<SystemTimeError> for Error {
    fn from(err: SystemTimeError) -> Self {
        Error::Other(Box::new(err))
    }
}
