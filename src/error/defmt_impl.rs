#![cfg(feature = "defmt_impl")]
use defmt::Format;

use crate::{Command, CommandError};

impl Format for CommandError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            CommandError::InvalidHeader => defmt::write!(f, "CH"),
            CommandError::MalformedPayload => defmt::write!(f, "CP"),
            CommandError::BufferTooSmall => defmt::write!(f, "CS"),
        }
    }
}

impl<'a> Format for Command<&'a [u8]> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Command::Constant { led_count, colour } => defmt::write!(f, "CC::L({})::CO({},{},{})", led_count, colour.0, colour.1, colour.2),
            Command::Stream(bytes) => defmt::write!(f, "CS::LB({})", bytes.len()),
            Command::Pulse { led_count, ..} => defmt::write!(f, "CP::L({}))", led_count),
            Command::Health => defmt::write!(f, "CH"),
        }
    }
}

#[cfg(feature = "owned")]
extern crate std;
#[cfg(feature = "owned")]
use std::vec::Vec;

#[cfg(feature = "owned")]
impl Format for Command<Vec<u8>> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Command::Constant { led_count, colour } => defmt::write!(f, "CC::L({})::CO({},{},{})", led_count, colour.0, colour.1, colour.2),
            Command::Stream(bytes) => defmt::write!(f, "CS::LB({})", bytes.len()),
            Command::Pulse { led_count, ..} => defmt::write!(f, "CP::L({}))", led_count),
            Command::Health => defmt::write!(f, "CH"),
        }
    }
}
