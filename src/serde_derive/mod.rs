#![cfg(feature = "serde_derive")]

#[cfg(feature = "std-write")]
mod std_write;

extern crate std;
use std::vec::Vec;
use serde::{Serialize, Deserialize};


mod stream {
    // adds some extra info to self-describing formats for stream variant
    use serde::{Serialize, Serializer, Deserializer, Deserialize, de::{Visitor, MapAccess, self}};
    use super::Vec;
    use core::fmt;

    pub(super) fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize, Debug)]
        struct SelfDescribing<'a> {
            bytes: &'a [u8],
        }

        let output = SelfDescribing {
            bytes,
        };

        output.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        
        #[derive(Deserialize)] 
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields { Bytes }
        
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
                        Fields::Bytes => if bytes.is_none() {
                            bytes = map.next_value()?;
                        } else {
                            return Err(de::Error::duplicate_field("bytes"))
                        }
                    }
                }

                let bytes = bytes
                    .ok_or_else(|| de::Error::missing_field("bytes"))?;
                if bytes.len() % 3 == 0 { 
                    Ok(bytes)
                } else {
                    Err(de::Error::custom("Invalid length for bytes command"))
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
    Pulse {
        led_count: u16,
        start: (u8, u8, u8),
        end: (u8, u8, u8),
        frames: u8,
        period: u16,
    }
}

impl Command {
    pub fn size_in_bytes(&self) -> usize {
       match self {
            Command::Constant {..} => 6,
            Command::Stream(bytes) => bytes.len() + 1,
            Command::Pulse {..} => 12,
       } 
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Vec};

   #[test]
   fn constant_ser() {
       let command = Command::Constant {led_count: 1, colour: (0, 126, 0)};
       let serialized = serde_json::to_string(&command).unwrap();
       let as_string = "{\"type\":\"constant\",\"ledCount\":1,\"colour\":[0,126,0]}";
       assert_eq!(as_string, serialized)
   }

   #[test]
   fn constant_de() {
       let command = Command::Constant {led_count: 1, colour: (0, 126, 0)};
       let as_string = "{\"type\":\"constant\",\"ledCount\":1,\"colour\":[0,126,0]}";
       let deserialized: Command = serde_json::from_str(&as_string).expect("Failed to deserialize const example");
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
       let deserialized: Command = serde_json::from_str(&as_string).expect("Failed to deserialize stream example");
       assert_eq!(deserialized, command);
   }

   #[test]
   fn pulse_ser() {

   }
}
