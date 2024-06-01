// Copyright (C) 2023 Lily Lyons
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

#[allow(missing_docs)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub script_text: String,
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

                let Some(_) = array.next_element::<alox_48::de::Ignored>()? else {
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

        array.serialize_element(&0usize)?;
        array.serialize_element(&self.name)?;
        array.serialize_element(&alox_48::RbString { data })?;

        array.end()
    }
}
