#![no_std]

use core::convert::From;
use core::fmt::{Display, Result as FMTResult, Formatter};

#[cfg(feature = "serde_derive")]
mod serde_derive;

#[cfg(feature = "serde_derive")]
pub use serde_derive::Command;

#[cfg(not(feature = "serde_derive"))]
mod embedded;

#[cfg(not(feature = "serde_derive"))]
pub use embedded::Command;

#[derive(Debug)]
pub enum CommandError {
    InvalidHeader,
    MalformedPayload,
    BufferTooSmall,
}

impl From<CommandError> for &'static str {
    fn from(value: CommandError) -> Self {
        (&value).into()
    }
}

impl From<&CommandError> for &'static str {
    fn from(value: &CommandError) -> Self {
        match value {
            CommandError::BufferTooSmall => "CS",
            CommandError::InvalidHeader => "CH",
            CommandError::MalformedPayload => "CP",
        }
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {     
        f.write_str(self.into())
    }
}
