// Copyright (C) 2024 Melody Madeline Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.

use base64::Engine;
use rand::Rng;

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Script {
    pub id: u32,
    pub name: String,
    pub script_text: String,
}

impl Script {
    /// Creates a new `Script` with a random ID.
    pub fn new(name: impl Into<String>, script_text: impl Into<String>) -> Self {
        Self {
            id: rand::thread_rng().gen_range(0..=99999999),
            name: name.into(),
            script_text: script_text.into(),
        }
    }
}

impl<'de> alox_48::Deserialize<'de> for Script {
    fn deserialize<D>(deserializer: D) -> Result<Self, alox_48::DeError>
    where
        D: alox_48::DeserializerTrait<'de>,
    {
        struct Visitor;

        impl<'de> alox_48::Visitor<'de> for Visitor {
            type Value = Script;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an array")
            }

            fn visit_array<A>(self, mut array: A) -> Result<Self::Value, alox_48::DeError>
            where
                A: alox_48::ArrayAccess<'de>,
            {
                use std::io::Read;

                let Some(id) = array.next_element()? else {
                    return Err(alox_48::DeError::missing_field("id".into()));
                };

                let Some(name) = array.next_element()? else {
                    return Err(alox_48::DeError::missing_field("name".into()));
                };

                let Some(data) = array.next_element::<alox_48::RbString>()? else {
                    return Err(alox_48::DeError::missing_field("data".into()));
                };

                let mut decoder = flate2::bufread::ZlibDecoder::new(data.data.as_slice());
                let mut script = String::new();
                decoder
                    .read_to_string(&mut script)
                    .map_err(alox_48::DeError::custom)?;

                Ok(Script {
                    id,
                    name,
                    script_text: script,
                })
            }
        }

        deserializer.deserialize(Visitor)
    }
}

impl alox_48::Serialize for Script {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, alox_48::SerError>
    where
        S: alox_48::SerializerTrait,
    {
        use alox_48::SerializeArray;
        use std::io::Write;

        let mut array = serializer.serialize_array(3)?;

        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Default::default());
        let data = encoder
            .write_all(self.script_text.as_bytes())
            .and_then(|_| encoder.finish())
            .map_err(alox_48::SerError::custom)?;

        array.serialize_element(&self.id)?;
        array.serialize_element(&self.name)?;
        array.serialize_element(&alox_48::RbString { data })?;

        array.end()
    }
}

impl<'de> serde::Deserialize<'de> for Script {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Script;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a key-value mapping")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                use serde::de::Error;
                use std::io::Read;

                let mut id = None;
                let mut name = None;
                let mut data = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "name" => name = Some(map.next_value()?),
                        "data" => data = Some(map.next_value::<String>()?),
                        _ => {}
                    }
                }

                let Some(name) = name else {
                    return Err(A::Error::missing_field("name"));
                };

                let Some(data) = data else {
                    return Err(A::Error::missing_field("data"));
                };

                let mut decoder = flate2::bufread::ZlibDecoder::new(std::io::Cursor::new(
                    base64::engine::general_purpose::STANDARD
                        .decode(data)
                        .map_err(A::Error::custom)?,
                ));
                let mut script = String::new();
                decoder
                    .read_to_string(&mut script)
                    .map_err(A::Error::custom)?;

                Ok(if let Some(id) = id {
                    Script {
                        id,
                        name,
                        script_text: script,
                    }
                } else {
                    Script::new(name, script)
                })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl serde::Serialize for Script {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        use serde::ser::SerializeMap;
        use std::io::Write;

        let mut map = serializer.serialize_map(Some(3))?;

        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Default::default());
        let data = encoder
            .write_all(self.script_text.as_bytes())
            .and_then(|_| encoder.finish())
            .map_err(S::Error::custom)?;

        map.serialize_entry("id", &self.id)?;
        map.serialize_entry("name", &self.name)?;
        map.serialize_entry(
            "data",
            &base64::engine::general_purpose::STANDARD.encode(data),
        )?;

        map.end()
    }
}
