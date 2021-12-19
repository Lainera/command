#![no_std]
use core::convert::From;
use core::fmt::{Display, Formatter, Result as FMTResult};
use error::*;

mod error;

enum Command<T> {
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
