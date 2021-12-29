#![no_std]
pub use error::*;
use core::fmt::{Display, Formatter, Result as FMTResult};
pub use embedded::*;

#[cfg(feature = "owned")]
pub use owned::*;

#[cfg(feature = "owned")]
mod owned;

#[cfg(feature = "serde_impl")]
pub mod serde_impl;

#[cfg(feature = "serde_impl")]
pub use serde_impl::{ser, de};

mod error;
mod embedded;

fn read_colour(slice: &[u8]) -> Result<(u8, u8, u8), CommandError> {
    if slice.len() < 3 {
        return Err(CommandError::MalformedPayload);
    }
    Ok((slice[0], slice[1], slice[2]))
}

fn read_as_u16(slice: &[u8]) -> Result<u16, CommandError> {
    if slice.len() != 2 {
        return Err(CommandError::MalformedPayload);
    }
    let mut tmp: [u8; 2] = [0; 2];
    tmp.copy_from_slice(slice);
    Ok(u16::from_be_bytes(tmp))
}

#[derive(Debug, PartialEq)]
pub enum Command<T> {
    Health,
    Constant {
        led_count: u16,
        colour: (u8, u8, u8),
    },
    Stream(T),
    Pulse {
        led_count: u16,
        start: (u8, u8, u8),
        end: (u8, u8, u8),
        frames: u8,
        period: u16,
    },
}

impl<T: AsRef<[u8]>> Command<T> {
    /// Reports size of command variant in bytes
    /// Length of payload + 1 for command type
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Command::Constant { .. } => 6,
            Command::Stream(slice) => slice
                .as_ref()
                .len() + 1,
            Command::Pulse { .. } => 12,
            Command::Health => 1,
        }
    }
}

impl<T> Display for Command<T> 
    where
    T: AsRef<[u8]>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        match &self {
            Command::Constant { led_count, colour } => writeln!(
                f,
                "Command::Constant -> ({}, {}, {}) x {}\r",
                colour.0, colour.1, colour.2, led_count
            )?,
            Command::Stream(slice) => {
                let slice = slice.as_ref();
                writeln!(
                    f,
                    "Command::Stream -> {:#x} for {}\r",
                    slice.as_ptr() as usize,
                    slice.len()
                )?
            },
            Command::Pulse {
                start,
                end,
                led_count,
                frames,
                period,
            } => {
                writeln!(f, "Command::Pulse\r")?;
                let (ff, fs, ft) = start;
                let (sf, ss, st) = end;
                writeln!(f, "s::({},{},{})", ff, fs, ft)?;
                writeln!(f, "e::({},{},{})", sf, ss, st)?;
                writeln!(f, "ct::{} fr::{} pr::{}\r", led_count, frames, period)?;
            },
            Command::Health => writeln!(f, "Command::Health")?
        }
        Ok(())
    }
}
