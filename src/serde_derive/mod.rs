#![cfg(feature = "serde_derive")]

#[cfg(feature = "std-write")]
pub mod std_write;

extern crate std;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

/// add custom serialization logic for self-describing formats for stream variant
mod stream {
    use super::Vec;
    use core::fmt;
    use serde::{
        de::{self, MapAccess, Visitor},
        Deserialize, Deserializer, Serialize, Serializer,
    };

    pub(super) fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize, Debug)]
        struct SelfDescribing<'a> {
            bytes: &'a [u8],
        }

        let output = SelfDescribing { bytes };

        output.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            Bytes,
        }

        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = Vec<u8>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct with 'bytes' key")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut bytes: Option<Vec<u8>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Bytes => {
                            if bytes.is_none() {
                                bytes = map.next_value()?;
                            } else {
                                return Err(de::Error::duplicate_field("bytes"));
                            }
                        }
                    }
                }

                let bytes = bytes.ok_or_else(|| de::Error::missing_field("bytes"))?;
                if bytes.len() % 3 == 0 {
                    Ok(bytes)
                } else {
                    Err(de::Error::custom("Byte length must be multiple of 3"))
                }
            }
        }

        deserializer.deserialize_map(BytesVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, core::cmp::PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Command {
    #[serde(rename_all = "camelCase")]
    Constant {
        led_count: u16,
        colour: (u8, u8, u8),
    },
    #[serde(with = "stream")]
    Stream(Vec<u8>),
    #[serde(rename_all = "camelCase")]
    Pulse {
        led_count: u16,
        start: (u8, u8, u8),
        end: (u8, u8, u8),
        frames: u8,
        period: u16,
    },
    Health,
}

impl Command {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Command::Constant { .. } => 6,
            Command::Stream(bytes) => bytes.len() + 1,
            Command::Pulse { .. } => 12,
            Command::Health => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Vec};

    #[test]
    fn constant_ser() {
        let command = Command::Constant {
            led_count: 1,
            colour: (0, 126, 0),
        };
        let serialized = serde_json::to_string(&command).unwrap();
        let as_string = "{\"type\":\"constant\",\"ledCount\":1,\"colour\":[0,126,0]}";
        assert_eq!(as_string, serialized)
    }

    #[test]
    fn constant_de() {
        let command = Command::Constant {
            led_count: 1,
            colour: (0, 126, 0),
        };
        let as_string = "{\"type\":\"constant\",\"ledCount\":1,\"colour\":[0,126,0]}";
        let deserialized: Command =
            serde_json::from_str(&as_string).expect("Failed to deserialize const example");
        assert_eq!(command, deserialized)
    }

    #[test]
    fn stream_ser() {
        let bytes = Vec::from([0, 127, 0]);
        let command = Command::Stream(bytes);
        let serialized = serde_json::to_string(&command).unwrap();
        let as_string = "{\"type\":\"stream\",\"bytes\":[0,127,0]}";
        assert_eq!(as_string, serialized);
    }

    #[test]
    fn stream_de() {
        let bytes = Vec::from([0, 127, 0]);
        let command = Command::Stream(bytes);
        let as_string = "{\"type\":\"stream\",\"bytes\":[0,127,0]}";
        let deserialized: Command =
            serde_json::from_str(&as_string).expect("Failed to deserialize stream example");
        assert_eq!(deserialized, command);
    }

    #[test]
    fn pulse_ser() {
        let command = Command::Pulse {
            led_count: 5,
            start: (0, 0, 0),
            end: (127, 0, 127),
            frames: 60,
            period: 2000,
        };
        let as_string = "{\"type\":\"pulse\",\"ledCount\":5,\"start\":[0,0,0],\"end\":[127,0,127],\"frames\":60,\"period\":2000}";
        let serialized =
            serde_json::to_string(&command).expect("Failed to serialize pulse example");
        assert_eq!(as_string, serialized);
    }

    #[test]
    fn pulse_de() {
        let command = Command::Pulse {
            led_count: 5,
            start: (0, 0, 0),
            end: (127, 0, 127),
            frames: 60,
            period: 2000,
        };
        let as_string = "{\"type\":\"pulse\",\"ledCount\":5,\"start\":[0,0,0],\"end\":[127,0,127],\"frames\":60,\"period\":2000}";
        let deserialized =
            serde_json::from_str(&as_string).expect("Failed to deserialize pulse example");
        assert_eq!(command, deserialized);
    }
    
    #[test]
    fn health_ser() {
        let command = Command::Health;
        let as_string = "{\"type\":\"health\"}";
        let serialized = serde_json::to_string(&command).expect("Failed to serialize health example");
        assert_eq!(serialized, as_string);
    }

    #[test]
    fn health_de() {
        let command = Command::Health;
        let as_string = "{\"type\":\"health\"}";
        let deserialized = serde_json::from_str(&as_string).expect("Failed to deserialize health example");
        assert_eq!(command, deserialized);
    }

    #[test]
    #[should_panic]
    fn garbled() {
        let _: Command = serde_json::from_str(&"{\"not\":\"legit\"}").unwrap();
    }

    #[test]
    #[should_panic(expected = "Byte length must be multiple of 3")]
    fn wrong_byte_count() {
        let _: Command =
            serde_json::from_str(&"{\"type\": \"stream\", \"bytes\": [127, 0]}").unwrap();
    }
}
