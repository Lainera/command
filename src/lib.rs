#![no_std]

use core::convert::From;
use core::fmt::{Display, Formatter, Result as FMTResult};

#[cfg(feature = "serde_derive")]
mod serde_derive;

#[cfg(feature = "defmt_derive")]
use defmt::Format;

#[cfg(feature = "std-write")]
pub use serde_derive::std_write;

#[cfg(feature = "serde_derive")]
pub use serde_derive::Command;

#[cfg(not(feature = "serde_derive"))]
mod embedded;

#[cfg(not(feature = "serde_derive"))]
pub use embedded::Command;

#[cfg(feature = "rbf-write")]
use embedded::rbf_write;

#[cfg(feature = "defmt_derive")]
impl Format for CommandError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            CommandError::InvalidHeader => defmt::write!(f, "CH"),
            CommandError::MalformedPayload => defmt::write!(f, "CP"),
            CommandError::BufferTooSmall => defmt::write!(f, "CS"),
        }
    }
}

#[cfg(feature = "defmt_derive")]
impl<'a> Format for Command<'a> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Command::Constant { led_count, colour } => defmt::write!(f, "CC::L({})::CO({},{},{})", led_count, colour.0, colour.1, colour.2),
            Command::Stream(bytes) => defmt::write!(f, "CS::LB({})", bytes.len()),
            Command::Pulse { led_count, start, end, frames, period } => defmt::write!(f, "CP::L({}))", led_count),
        }
    }
}

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

