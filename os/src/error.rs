use alloc::string::String;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("invalid argument error: `{0}`")]
    InvalidArgument(String),
}

pub type Result<T> = core::result::Result<T, KernelError>;
