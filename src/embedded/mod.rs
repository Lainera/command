#![cfg(not(feature = "serde_derive"))]

use crate::CommandError;
use core::convert::TryFrom;
use core::fmt::{Display, Formatter, Result as FMTResult};

#[cfg(feature = "rbf-write")]
pub mod rbf_write;

pub enum Command<'a> {
    Health,
    Constant {
        led_count: u16,
        colour: (u8, u8, u8),
    },
    Stream(&'a [u8]),
    Pulse {
        led_count: u16,
        start: (u8, u8, u8),
        end: (u8, u8, u8),
        frames: u8,
        period: u16,
    },
}

impl<'a> Command<'a> {
    /// Reports size of command variant in bytes
    /// Length of payload + 1 for command type
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Command::Constant { .. } => 6,
            Command::Stream(slice) => slice.len() + 1,
            Command::Pulse { .. } => 12,
            Command::Health => 1,
        }
    }

    fn read_colour(slice: &[u8]) -> Result<(u8, u8, u8), CommandError> {
        if slice.len() < 3 {
            return Err(CommandError::MalformedPayload);
        }
        Ok((slice[0], slice[1], slice[2]))
    }

    pub fn read_as_u16(slice: &[u8]) -> Result<u16, CommandError> {
        if slice.len() != 2 {
            return Err(CommandError::MalformedPayload);
        }
        let mut tmp: [u8; 2] = [0; 2];
        tmp.copy_from_slice(slice);
        Ok(u16::from_be_bytes(tmp))
    }
}

impl<'a> TryFrom<&'a [u8]> for Command<'a> {
    type Error = CommandError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == 0 {
            return Err(CommandError::MalformedPayload);
        }

        let (header, buffer) = value.split_at(1);
        match header[0] {
            b's' => {
                if buffer.len() % 3 != 0 {
                    Err(CommandError::MalformedPayload)
                } else {
                    Ok(Command::Stream(buffer))
                }
            }
            b'c' => {
                let led_count = Self::read_as_u16(&buffer[..2])?;
                let colour = Self::read_colour(&buffer[2..])?;
                Ok(Command::Constant { led_count, colour })
            }
            b'p' => {
                let led_count = Self::read_as_u16(&buffer[..2])?;
                let start_led = Self::read_colour(&buffer[2..5])?;
                let end_led = Self::read_colour(&buffer[5..8])?;
                let frames = buffer[8];
                let period = Self::read_as_u16(&buffer[9..11])?;

                Ok(Command::Pulse {
                    start: start_led,
                    end: end_led,
                    frames,
                    period,
                    led_count,
                })
            }
            b'h' => Ok(Command::Health),
            _ => Err(CommandError::InvalidHeader),
        }
    }
}

impl<'a> Display for Command<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FMTResult {
        match &self {
            Command::Constant { led_count, colour } => writeln!(
                f,
                "Command::Constant -> ({}, {}, {}) x {}\r",
                colour.0, colour.1, colour.2, led_count
            )?,
            Command::Stream(slice) => writeln!(
                f,
                "Command::Stream -> {:#x} for {}\r",
                slice.as_ptr() as u32,
                slice.len()
            )?,
            Command::Pulse {
                start,
                end,
                led_count,
                frames,
                period,
            } => {
                writeln!(f, "Command::Pulse\r")?;
                let (ff, fs, ft) = start.clone();
                let (sf, ss, st) = end.clone();
                writeln!(f, "s::({},{},{})", ff, fs, ft)?;
                writeln!(f, "e::({},{},{})", sf, ss, st)?;
                writeln!(f, "ct::{} fr::{} pr::{}\r", led_count, frames, period)?;
            },
            Command::Health => writeln!(f, "Command::Health")?
        }
        Ok(())
    }
}
