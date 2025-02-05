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

pub fn deserialize_with<'de, D>(deserializer: D) -> Result<[Option<String>; 7], alox_48::DeError>
where
    D: alox_48::DeserializerTrait<'de>,
{
    struct Visitor;

    impl<'de> alox_48::Visitor<'de> for Visitor {
        type Value = [Option<String>; 7];

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a vec of strings")
        }

        fn visit_array<A>(self, mut array: A) -> Result<Self::Value, alox_48::DeError>
        where
            A: alox_48::ArrayAccess<'de>,
        {
            const DEFAULT_VALUE: Option<String> = None;
            let mut values = [DEFAULT_VALUE; 7];
            let mut i = 0;

            while let Some(value) = array.next_element::<String>()? {
                values[i] = (!value.is_empty()).then_some(value);
                i += 1;
                if i == 7 {
                    break;
                }
            }

            Ok(values)
        }
    }

    deserializer.deserialize(Visitor)
}

pub fn serialize_with<S>(
    values: &[Option<String>; 7],
    serializer: S,
) -> Result<S::Ok, alox_48::SerError>
where
    S: alox_48::SerializerTrait,
{
    use alox_48::SerializeArray;

    let mut array = serializer.serialize_array(values.len())?;

    for value in values {
        array.serialize_element(value.as_deref().unwrap_or_default())?;
    }

    array.end()
}
