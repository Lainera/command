#![cfg(feature  = "serde_impl")]
pub mod ser {
    use serde::{Serialize, ser::SerializeMap};
    use crate::Command;

    impl<T> Serialize for Command<T>
    where T: AsRef<[u8]> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
                match self {
                    Command::Health => {
                        let mut state = serializer.serialize_map(Some(1))?;
                        state.serialize_entry("type", "health")?;
                        state.end()
                    },
                    Command::Constant { led_count, colour } => {
                        let mut state = serializer.serialize_map(Some(3))?;
                        state.serialize_entry("type", "constant")?;
                        state.serialize_entry("led_count", led_count)?;
                        state.serialize_entry("colour", colour)?;
                        state.end()

                    },
                    Command::Stream(inner) => {
                        let mut state = serializer.serialize_map(Some(2))?;
                        state.serialize_entry("type", "stream")?;
                        state.serialize_entry("bytes", inner.as_ref())?;
                        state.end()
                    },
                    Command::Pulse { led_count, start, end, frames, period } => {
                        let mut state = serializer.serialize_map(Some(6))?;
                        state.serialize_entry("type", "pulse")?;
                        state.serialize_entry("led_count", led_count)?;
                        state.serialize_entry("start", start)?;
                        state.serialize_entry("end", end)?;
                        state.serialize_entry("frames", frames)?;
                        state.serialize_entry("period", period)?;
                        state.end()
                    },
                }
        }
    }
}

pub mod de {
    use crate::Command;
    use serde::{Deserialize, de::{Visitor, self, MapAccess}};
    use core::{fmt::{Formatter, Result as FMTResult}, marker::PhantomData};

    enum CommandVariant {
        Health,
        Constant,
        Stream,
        Pulse,
    }

    impl Default for CommandVariant {
        fn default() -> Self {
            CommandVariant::Health
        }
    }

    struct CommandVisitor<'a, T> {
        cmd_variant: CommandVariant,
        led_count: Option<u16>,
        colour: Option<(u8, u8, u8)>,
        start: Option<(u8, u8, u8)>,
        end: Option<(u8, u8, u8)>,
        frames: Option<u8>,
        period: Option<u16>,
        bytes: Option<T>,
        _pd: PhantomData<&'a u8>,
    }

    impl<'a, T> Default for CommandVisitor<'a, T> 
    {
        fn default() -> Self {
        Self { 
            cmd_variant: Default::default(), 
            led_count: None, 
            colour: None, 
            start: None, 
            end: None, 
            frames: None, 
            period: None, 
            bytes: None, 
            _pd: Default::default() 
        }
        }
    }

    impl<'a, T> CommandVisitor<'a, T> {
        fn resolve_cmd_type<'de, E: de::Error>(&mut self, mut map: impl MapAccess<'de, Error = E>) -> Result<(), E> {
            match map.next_value()? {
                "constant" => self.cmd_variant = CommandVariant::Constant,
                "pulse" => self.cmd_variant = CommandVariant::Pulse,
                "health" => self.cmd_variant = CommandVariant::Health,
                "stream" => self.cmd_variant = CommandVariant::Stream,
                _ => return Err(de::Error::custom("Unexpected command type"))
            }

            Ok(())
        }
    }

    impl<'de: 'a, 'a, T> Visitor<'de> for CommandVisitor<'a, T> 
        where
            T: AsRef<[u8]> + Deserialize<'de>
    {
        type Value = Command<T>;

        fn expecting(&self, formatter: &mut Formatter) -> FMTResult {
            formatter.write_str("Map or sequence of bytes")
        }

        fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
        where
                A: serde::de::MapAccess<'de>, 
        {
            while let Some(key) = map.next_key()? {
                match key {
                    "type" => {
                        self.resolve_cmd_type(&mut map)?;
                    },
                    "led_count" => self.led_count = map.next_value()?,
                    "start" => self.start = map.next_value()?,
                    "end" => self.end = map.next_value()?,
                    "colour" => self.colour = map.next_value()?,
                    "frames" => self.frames = map.next_value()?,
                    "period" => self.period = map.next_value()?,
                    "bytes" => self.bytes = map.next_value()?,
                    _ => return Err(de::Error::unknown_field(key, &["type", "led_count", "start", "end", "colour", "frames", "period", "bytes"])),
                }
            }

            match self.cmd_variant {
                CommandVariant::Health => Ok(Command::Health),
                CommandVariant::Constant => {
                    let colour = self.colour.ok_or_else(|| de::Error::missing_field("colour"))?;
                    let led_count = self.led_count.ok_or_else(|| de::Error::missing_field("led_count"))?;

                    Ok(Command::Constant {led_count, colour})
                },
                CommandVariant::Stream => {
                    let bytes = self.bytes.ok_or_else(|| de::Error::missing_field("bytes"))?;
                    if bytes.as_ref().len() % 3 == 0 {
                        Ok(Command::Stream(bytes))
                    } else {
                        Err(de::Error::custom("Byte length must be multiple of 3"))
                    }
                },
                CommandVariant::Pulse => {
                    let start = self.start.ok_or_else(|| de::Error::missing_field("start"))?;
                    let end = self.end.ok_or_else(|| de::Error::missing_field("end"))?;
                    let frames = self.frames.ok_or_else(|| de::Error::missing_field("frames"))?;
                    let period = self.period.ok_or_else(|| de::Error::missing_field("period"))?;
                    let led_count = self.led_count.ok_or_else(|| de::Error::missing_field("led_count"))?;

                    Ok(Command::Pulse { led_count, start, end, frames, period })
                },
            }
        }
    } 

    impl<'de: 'a, 'a, T> Deserialize<'de> for Command<T>
        where
            T: AsRef<[u8]> + Deserialize<'de>
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
                D: serde::Deserializer<'de> 
        {
            deserializer.deserialize_map(CommandVisitor::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Command;
    extern crate std;
    use std::{vec, vec::Vec};
    type Plh = Vec<u8>;

    #[test]
    fn constant_ser() {
        let command: Command<Plh> = Command::Constant {
            led_count: 1,
            colour: (0, 126, 0),
        };
        let serialized = serde_json::to_string(&command).unwrap();
        let as_str = "{\"type\":\"constant\",\"led_count\":1,\"colour\":[0,126,0]}";
        assert_eq!(as_str, serialized)
    }

    #[test]
    fn constant_de() {
        let command = Command::Constant {
            led_count: 1,
            colour: (0, 126, 0),
        };
        let as_str = "{\"type\":\"constant\",\"led_count\":1,\"colour\":[0,126,0]}";
        let deserialized: Command<Plh> =
            serde_json::from_str(as_str).expect("Failed to deserialize const example");
        assert_eq!(command, deserialized)
    }

    #[test]
    fn stream_ser() {
        let bytes: &[u8] = &[0, 127, 0];
        let command = Command::Stream(bytes);
        let serialized = serde_json::to_string(&command).unwrap();
        let as_string = "{\"type\":\"stream\",\"bytes\":[0,127,0]}";
        assert_eq!(as_string, serialized);
    }

    #[test]
    fn stream_de() {
        let bytes = vec![0, 127, 0];
        let command = Command::Stream(bytes);
        let as_str = "{\"type\":\"stream\",\"bytes\":[0,127,0]}";
        let deserialized: Command<Plh> =
            serde_json::from_str(as_str).expect("Failed to deserialize stream example");
        assert_eq!(deserialized, command);
    }

    #[test]
    fn pulse_ser() {
        let command: Command<Plh> = Command::Pulse {
            led_count: 5,
            start: (0, 0, 0),
            end: (127, 0, 127),
            frames: 60,
            period: 2000,
        };
        let as_string = "{\"type\":\"pulse\",\"led_count\":5,\"start\":[0,0,0],\"end\":[127,0,127],\"frames\":60,\"period\":2000}";
        let serialized =
            serde_json::to_string(&command).expect("Failed to serialize pulse example");
        assert_eq!(as_string, serialized);
    }

    #[test]
    fn pulse_de() {
        let command: Command<Plh> = Command::Pulse {
            led_count: 5,
            start: (0, 0, 0),
            end: (127, 0, 127),
            frames: 60,
            period: 2000,
        };
        let as_str = "{\"type\":\"pulse\",\"led_count\":5,\"start\":[0,0,0],\"end\":[127,0,127],\"frames\":60,\"period\":2000}";
        let deserialized =
            serde_json::from_str(as_str).expect("Failed to deserialize pulse example");
        assert_eq!(command, deserialized);
    }
    
    #[test]
    fn health_ser() {
        let command: Command<Plh> = Command::Health;
        let serialized = serde_json::to_string(&command).expect("Failed to serialize health example");
        assert_eq!(serialized, "{\"type\":\"health\"}");
    }

    #[test]
    fn health_de() {
        let command: Command<Plh> = Command::Health;
        let deserialized = serde_json::from_str("{\"type\":\"health\"}").expect("Failed to deserialize health example");
        assert_eq!(command, deserialized);
    }
    
    #[test]
    #[should_panic]
    fn garbled() {
        let _: Command<Plh> = serde_json::from_str("{\"not\":\"legit\"}").unwrap();
    }

    #[test]
    #[should_panic(expected = "Byte length must be multiple of 3")]
    fn wrong_byte_count() {
        let _: Command<Plh> =
            serde_json::from_str("{\"type\": \"stream\", \"bytes\": [127, 0]}").unwrap();
    }
}
