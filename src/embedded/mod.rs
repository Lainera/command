use crate::{Command, CommandError};
use core::convert::TryFrom;

fn try_read_colour(slice: &[u8]) -> Result<(u8, u8, u8), CommandError> {
    if slice.len() < 3 {
        return Err(CommandError::MalformedPayload);
    }
    Ok((slice[0], slice[1], slice[2]))
}

fn try_read_u16(slice: &[u8]) -> Result<u16, CommandError> {
    if slice.len() != 2 {
        return Err(CommandError::MalformedPayload);
    }
    let mut tmp: [u8; 2] = [0; 2];
    tmp.copy_from_slice(slice);
    Ok(u16::from_be_bytes(tmp))
}

fn try_write_u16(v: u16, buf: &mut [u8]) -> Result<usize, CommandError> {
    let size = core::mem::size_of::<u16>();
    if buf.len() < size {
        return Err(CommandError::BufferTooSmall);
    }
    buf[..2].copy_from_slice(&v.to_be_bytes());

    Ok(size)
}

fn try_write_colour(colour: (u8, u8, u8), buf: &mut [u8]) -> Result<usize, CommandError> {
    let size = 3;
    if buf.len() < size {
        return Err(CommandError::BufferTooSmall);
    }

    buf[0] = colour.0;
    buf[1] = colour.1;
    buf[2] = colour.2;
    Ok(size)
}

impl<T> Command<T>
where
    T: AsRef<[u8]>,
{
    pub fn try_write_bytes(&self, buf: &mut dyn AsMut<[u8]>) -> Result<usize, CommandError> {
        let buf = buf.as_mut();
        let len = self.size_in_bytes();
        if len > buf.len() {
            return Err(CommandError::BufferTooSmall);
        }
        match self {
            Command::Health => buf[0] = b'h',
            Command::Constant { led_count, colour } => {
                buf[0] = b'c';
                try_write_u16(*led_count, &mut buf[1..3])?;
                try_write_colour(*colour, &mut buf[3..])?;
            }
            Command::Stream(bytes) => {
                buf[0] = b's';
                let bytes = bytes.as_ref();
                let to_copy = bytes.len();
                buf[1..to_copy + 1].copy_from_slice(bytes);
            }
            Command::Pulse {
                led_count,
                start,
                end,
                frames,
                period,
            } => {
                buf[0] = b'p';
                try_write_u16(*led_count, &mut buf[1..3])?;
                try_write_colour(*start, &mut buf[3..6])?;
                try_write_colour(*end, &mut buf[6..9])?;
                buf[9] = *frames;
                try_write_u16(*period, &mut buf[10..12])?;
            }
        };

        Ok(len)
    }
}

impl<'a> TryFrom<&'a [u8]> for Command<&'a [u8]> {
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
                    Ok(Command::Stream(buffer))
                }
            }
            b'c' => {
                let led_count = try_read_u16(&buffer[..2])?;
                let colour = try_read_colour(&buffer[2..])?;
                Ok(Command::Constant { led_count, colour })
            }
            b'p' => {
                let led_count = try_read_u16(&buffer[..2])?;
                let start_led = try_read_colour(&buffer[2..5])?;
                let end_led = try_read_colour(&buffer[5..8])?;
                let frames = buffer[8];
                let period = try_read_u16(&buffer[9..11])?;

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

#[cfg(test)]
mod tests {
    use core::convert::TryFrom;

    use crate::{
        embedded::{try_read_colour, try_read_u16, try_write_colour, try_write_u16},
        Command, CommandError,
    };

    #[test]
    fn given_valid_slice_reads_u16() {
        let val = 666_u16;
        let parsed = try_read_u16(&val.to_be_bytes()).expect("Deserialization fail");
        assert_eq!(val, parsed);
    }

    #[test]
    fn given_invalid_slice_returns_error() {
        let val = 666_u16;
        let outcome = try_read_u16(&val.to_be_bytes()[..1]);
        assert!(outcome.is_err());
        assert_eq!(outcome.unwrap_err(), CommandError::MalformedPayload);
    }

    #[test]
    fn given_valid_slice_reads_colour() {
        let colour: &[u8] = &[254, 0, 254];
        let parsed = try_read_colour(colour).expect("Deserialization fail");
        assert_eq!((colour[0], colour[1], colour[2]), parsed);
    }

    #[test]
    fn given_invalid_col_slice_returns_error() {
        let colour: &[u8] = &[254, 0, 254];
        let outcome = try_read_colour(&colour[..1]);
        assert!(outcome.is_err());
        assert_eq!(outcome.unwrap_err(), CommandError::MalformedPayload);
    }

    #[test]
    fn given_buf_too_small_fails_to_write_u16() {
        let mut buf = [0_u8; 1];
        let outcome = try_write_u16(666, &mut buf);
        assert!(outcome.is_err());
        assert_eq!(outcome.unwrap_err(), CommandError::BufferTooSmall);
    }

    #[test]
    fn given_valid_buf_writes_u16() {
        let val = 666_u16;
        let mut buf = [0_u8; 2];
        let outcome = try_write_u16(val, &mut buf);
        assert!(outcome.is_ok());
        assert_eq!(&val.to_be_bytes(), &buf[..]);
    }

    #[test]
    fn given_buf_too_small_fails_to_write_colour() {
        let mut buf = [0_u8; 1];
        let outcome = try_write_colour((1, 1, 1), &mut buf);
        assert!(outcome.is_err());
        assert_eq!(outcome.unwrap_err(), CommandError::BufferTooSmall);
    }

    #[test]
    fn given_valid_buf_writes_colour() {
        let colour = (254, 0, 254);
        let mut buf = [0_u8; 3];
        let outcome = try_write_colour(colour, &mut buf);
        assert!(outcome.is_ok());
        assert_eq!(&[colour.0, colour.1, colour.2], &buf[..]);
    }

    #[test]
    fn e2e_const() {
        let cmd: Command<&[u8]> = Command::Constant {
            led_count: 257,
            colour: (254, 0, 254),
        };
        let mut buf = [0_u8; 128];
        let serialized = cmd.try_write_bytes(&mut buf);
        assert!(serialized.is_ok());
        let len = serialized.unwrap();
        let deserialized = Command::try_from(&buf[..len]);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), cmd);
    }

    #[test]
    fn e2e_health() {
        let cmd: Command<&[u8]> = Command::Health;
        let mut buf = [0_u8; 128];
        let serialized = cmd.try_write_bytes(&mut buf);
        assert!(serialized.is_ok());
        let len = serialized.unwrap();
        let deserialized = Command::try_from(&buf[..len]);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), cmd);
    }

    #[test]
    fn e2e_pulse() {
        let cmd: Command<&[u8]> = Command::Pulse {
            led_count: 300,
            start: (0, 0, 0),
            end: (255, 0, 0),
            frames: 60,
            period: 1000,
        };
        let mut buf = [0_u8; 128];
        let serialized = cmd.try_write_bytes(&mut buf);
        assert!(serialized.is_ok());
        let len = serialized.unwrap();
        let deserialized = Command::try_from(&buf[..len]);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), cmd);
    }

    #[test]
    fn e2e_stream() {
        let cmd = Command::Stream([127, 127, 127, 0, 0, 0, 127, 127, 127, 0, 0, 0].as_ref());
        let mut buf = [0_u8; 128];
        let serialized = cmd.try_write_bytes(&mut buf);
        assert!(serialized.is_ok());
        let len = serialized.unwrap();
        let deserialized = Command::try_from(&buf[..len]);
        assert!(deserialized.is_ok());
        assert_eq!(deserialized.unwrap(), cmd);
    }
}
