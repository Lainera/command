#![cfg(feature = "std-write")]

use crate::Command;
extern crate std;

impl Command {
    pub fn write_as_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let size = self.size_in_bytes();
        match *self {
            Command::Constant { led_count, colour } => {
                writer.write(&[b'c'])?;
                writer.write(&led_count.to_be_bytes())?;
                writer.write(&[colour.0, colour.1, colour.2])?;
                writer.flush()?;
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
                writer.flush()?;
            }
            Command::Stream(ref buf) => {
                writer.write(&[b's'])?;
                writer.write(buf)?;
                writer.flush()?;
            }
        }

        Ok(size)
    }
}
