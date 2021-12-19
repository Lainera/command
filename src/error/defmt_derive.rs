#![cfg(feature = "defmt_derive")]

use defmt::Format;

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
            Command::Pulse { led_count, start, end, frames, period } => defmt::write!(f, "CP::L({}))", led_count),
            Command::Health => defmt::write!(f, "CH"),
        }
    }
}

impl Format for Command<Vec<u8>> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Command::Constant { led_count, colour } => defmt::write!(f, "CC::L({})::CO({},{},{})", led_count, colour.0, colour.1, colour.2),
            Command::Stream(bytes) => defmt::write!(f, "CS::LB({})", bytes.len()),
            Command::Pulse { led_count, start, end, frames, period } => defmt::write!(f, "CP::L({}))", led_count),
            Command::Health => defmt::write!(f, "CH"),
        }
    }
}
