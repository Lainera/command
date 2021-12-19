#![cfg(feature = "owned")]
extern crate std;
use std::vec::Vec;
use crate::{Command, CommandError, read_as_u16, read_colour};
use core::convert::TryFrom;

impl<'a> TryFrom<&'a [u8]> for Command<Vec<u8>> {
    type Error = CommandError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(CommandError::MalformedPayload);
        }

        let (header, buffer) = value.split_at(1);
        match header[0] {
            b's' => {
                if buffer.len() % 3 != 0 {
                    Err(CommandError::MalformedPayload)
                } else {
                    Ok(Command::Stream(buffer.to_vec()))
                }
            }
            b'c' => {
                let led_count = read_as_u16(&buffer[..2])?;
                let colour = read_colour(&buffer[2..])?;
                Ok(Command::Constant { led_count, colour })
            }
            b'p' => {
                let led_count = read_as_u16(&buffer[..2])?;
                let start_led = read_colour(&buffer[2..5])?;
                let end_led = read_colour(&buffer[5..8])?;
                let frames = buffer[8];
                let period = read_as_u16(&buffer[9..11])?;

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

