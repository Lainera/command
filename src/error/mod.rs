#[cfg(feature = "defmt_derive")]
mod defmt_derive;

#[cfg(feature = "defmt_derive")]
pub use defmt_derive::*;

use core::fmt::{
    Display, 
    Formatter,
    Result as FMTResult,
};

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
