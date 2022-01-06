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

impl<T: Clone> Clone for Command<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Health => Self::Health,
            Self::Constant { led_count, colour } => Self::Constant { led_count: *led_count, colour: *colour },
            Self::Stream(inner) => Self::Stream(inner.clone()),
            Self::Pulse { led_count, start, end, frames, period } => Self::Pulse { 
                led_count: *led_count, 
                start: *start, 
                end: *end, 
                frames: *frames, 
                period: *period 
            },
        }
    }
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
