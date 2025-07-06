use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = std::result::Result<T, Error>;

#[allow(unused)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chain is empty")]
    ChainIsEmpty,
    #[error("Block with index {0} has invalid hash {1}")]
    BlockHasInvalidHash(u64, String),
    #[error("Block with index {0} has invalid previous block hash: actual: '{1}', expected: '{2}'")]
    BlockHasInvalidPreviusBlockHash(u64, String, String),
    #[error("Block with index {0} doesn't satisfy difficulty '{1}'")]
    UnsatisfiedHashDifficulty(u64, usize),
    #[error("HTTP parsing error: {0}")]
    HttpParsing(#[from] axum::http::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + 'static>),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("RESTful API internal error: {self:#?}");
        match self {
            Error::HttpParsing(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response(),
        }
    }
}
