use serde::{Serialize, Deserialize};

#[cfg(feature = "deserialize")] extern crate std;
#[derive(Serialize,Deserialize)]
pub enum Command<'a> {
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
    }
}
