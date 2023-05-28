#![cfg(feature = "owned")]
extern crate alloc;
use crate::{Command, CommandError};
use alloc::vec::Vec;
use core::convert::TryFrom;

impl<'a> From<Command<&'a [u8]>> for Command<Vec<u8>> {
    fn from(src: Command<&'a [u8]>) -> Self {
        match src {
            Command::Health => Command::Health,
            Command::Constant { led_count, colour } => Command::Constant { led_count, colour },
            Command::Stream(bytes) => Command::Stream(bytes.to_vec()),
            Command::Pulse {
                led_count,
                start,
                end,
                frames,
                period,
            } => Command::Pulse {
                led_count,
                start,
                end,
                frames,
                period,
            },
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Command<Vec<u8>> {
    type Error = CommandError;
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Command::try_from(value).map(|cmd: Command<&[u8]>| cmd.into())
    }
}
