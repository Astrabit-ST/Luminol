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
use camino::Utf8PathBuf;

pub fn serialize_with<S>(
    path: &Option<Utf8PathBuf>,
    serializer: S,
) -> Result<S::Ok, alox_48::SerError>
where
    S: alox_48::SerializerTrait,
{
    match path {
        Some(path) => serializer.serialize_rust_string(path.as_str()),
        None => serializer.serialize_rust_string(""),
    }
}

pub fn deserialize_with<'de, D>(deserializer: D) -> Result<Option<Utf8PathBuf>, alox_48::DeError>
where
    D: alox_48::DeserializerTrait<'de>,
{
    struct Visitor;

    impl alox_48::Visitor<'_> for Visitor {
        type Value = Option<Utf8PathBuf>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_string(self, v: &[u8]) -> Result<Self::Value, alox_48::DeError> {
            if v.is_empty() {
                Ok(None)
            } else {
                String::from_utf8(v.to_vec())
                    .map(Into::into)
                    .map(Some)
                    .map_err(|e| alox_48::DeError::custom(e.to_string()))
            }
        }
    }

    deserializer.deserialize(Visitor)
}
