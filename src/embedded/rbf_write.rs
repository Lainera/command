#![cfg(feature = "rbf-write")]

use crate::Command;
use ring_buffer::Write;

impl<'a> Command<'a> {
    pub fn write_bytes<W: Write<u8>>(
        &self,
        writer: &mut W,
    ) -> Result<usize, <W as Write<u8>>::Error> {
        let size = self.size_in_bytes();
        writer.write(&(size as u16).to_be_bytes())?;
        match *self {
            Command::Constant { led_count, colour } => {
                writer.write(&[b'c'])?;
                writer.write(&led_count.to_be_bytes())?;
                writer.write(&[colour.0, colour.1, colour.2])?;
            }
            Command::Pulse {
                led_count,
                start,
                end,
                frames,
                period,
            } => {
                writer.write(&[b'p'])?;
                writer.write(&led_count.to_be_bytes())?;
                writer.write(&[start.0, start.1, start.2, end.0, end.1, end.2, frames])?;
                writer.write(&period.to_be_bytes())?;
            }
            Command::Stream(buf) => {
                writer.write(&[b's'])?;
                writer.write(buf)?;
            }
        }

        Ok(size)
    }
}
