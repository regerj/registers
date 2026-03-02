#![no_std]

pub use register_macros::*;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Field write out of bounds")]
    OutOfBoundsFieldWrite,
}

pub type Result<T> = core::result::Result<T, Error>;
