#![cfg(feature  = "serde_impl")]
use serde::{Deserialize, Serialize, ser::{SerializeMap, SerializeStructVariant}};
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

impl<'de, T> Deserialize<'de> for Command<T> 
where T: AsRef<[u8]> 
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: serde::Deserializer<'de> {
                todo!()
    }
}

//#[derive(Serialize, Deserialize, Debug, core::cmp::PartialEq)]
//#[serde(tag = "type")]
//#[serde(rename_all = "camelCase")]
//pub enum Command {
//    #[serde(rename_all = "camelCase")]
//    Constant {
//        led_count: u16,
//        colour: (u8, u8, u8),
//    },
//    #[serde(with = "stream")]
//    Stream(Vec<u8>),
//    #[serde(rename_all = "camelCase")]
//    Pulse {
//        led_count: u16,
//        start: (u8, u8, u8),
//        end: (u8, u8, u8),
//        frames: u8,
//        period: u16,
//    },
//    Health,
//}
///// add custom serialization logic for self-describing formats for stream variant
//mod stream {
//    use super::Vec;
//    use core::fmt;
//    use serde::{
//        de::{self, MapAccess, Visitor},
//        Deserialize, Deserializer, Serialize, Serializer,
//    };
//
//    pub(super) fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
//        #[derive(Serialize, Debug)]
//        struct SelfDescribing<'a> {
//            bytes: &'a [u8],
//        }
//
//        let output = SelfDescribing { bytes };
//
//        output.serialize(serializer)
//    }
//
//    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
//        deserializer: D,
//    ) -> Result<Vec<u8>, D::Error> {
//        #[derive(Deserialize)]
//        #[serde(field_identifier, rename_all = "lowercase")]
//        enum Fields {
//            Bytes,
//        }
//
//        struct BytesVisitor;
//
//        impl<'de> Visitor<'de> for BytesVisitor {
//            type Value = Vec<u8>;
//            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                formatter.write_str("struct with 'bytes' key")
//            }
//
//            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
//            where
//                A: MapAccess<'de>,
//            {
//                let mut bytes: Option<Vec<u8>> = None;
//
//                while let Some(key) = map.next_key()? {
//                    match key {
//                        Fields::Bytes => {
//                            if bytes.is_none() {
//                                bytes = map.next_value()?;
//                            } else {
//                                return Err(de::Error::duplicate_field("bytes"));
//                            }
//                        }
//                    }
//                }
//
//                let bytes = bytes.ok_or_else(|| de::Error::missing_field("bytes"))?;
//                if bytes.len() % 3 == 0 {
//                    Ok(bytes)
//                } else {
//                    Err(de::Error::custom("Byte length must be multiple of 3"))
//                }
//            }
//        }
//
//        deserializer.deserialize_map(BytesVisitor)
//    }
//}

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn constant_ser() {
        let command: Command<&[u8]> = Command::Constant {
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
        let deserialized: Command<&[u8]> =
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
        let bytes: &[u8] = &[0, 127, 0];
        let command = Command::Stream(bytes);
        let as_str = "{\"type\":\"stream\",\"bytes\":[0,127,0]}";
        let deserialized: Command<&[u8]> =
            serde_json::from_str(as_str).expect("Failed to deserialize stream example");
        assert_eq!(deserialized, command);
    }

    #[test]
    fn pulse_ser() {
        let command: Command<&[u8]> = Command::Pulse {
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
        let command: Command<&[u8]> = Command::Pulse {
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
        let command: Command<&[u8]> = Command::Health;
        let serialized = serde_json::to_string(&command).expect("Failed to serialize health example");
        assert_eq!(serialized, "{\"type\":\"health\"}");
    }

    #[test]
    fn health_de() {
        let command: Command<&[u8]> = Command::Health;
        let deserialized = serde_json::from_str("{\"type\":\"health\"}").expect("Failed to deserialize health example");
        assert_eq!(command, deserialized);
    }

    #[test]
    #[should_panic]
    fn garbled() {
        let _: Command<&[u8]> = serde_json::from_str("{\"not\":\"legit\"}").unwrap();
    }

    #[test]
    #[should_panic(expected = "Byte length must be multiple of 3")]
    fn wrong_byte_count() {
        let _: Command<&[u8]> =
            serde_json::from_str("{\"type\": \"stream\", \"bytes\": [127, 0]}").unwrap();
    }
}
