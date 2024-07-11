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

pub fn deserialize_with<'de, D>(deserializer: D) -> Result<Vec<usize>, alox_48::DeError>
where
    D: alox_48::DeserializerTrait<'de>,
{
    struct Visitor;

    impl<'de> alox_48::Visitor<'de> for Visitor {
        type Value = Vec<usize>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a vec of nonzero usizes")
        }

        fn visit_array<A>(self, mut array: A) -> Result<Self::Value, alox_48::DeError>
        where
            A: alox_48::ArrayAccess<'de>,
        {
            let mut values = Vec::with_capacity(array.len());

            while let Some(value) = array.next_element::<std::num::NonZeroUsize>()? {
                values.push(value.get() - 1);
            }

            Ok(values)
        }
    }

    deserializer.deserialize(Visitor)
}

pub fn serialize_with<S>(values: &Vec<usize>, serializer: S) -> Result<S::Ok, alox_48::SerError>
where
    S: alox_48::SerializerTrait,
{
    use alox_48::SerializeArray;

    let mut array = serializer.serialize_array(values.len())?;

    for value in values {
        array.serialize_element(&(*value + 1))?;
    }

    array.end()
}
