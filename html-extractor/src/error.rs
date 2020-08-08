use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(Cow<'static, str>),
}
