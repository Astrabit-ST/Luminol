// Copyright (C) 2022 Melody Madeline Lyons
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

pub fn deserialize_with<'de, D, T>(deserializer: D) -> Result<Vec<T>, alox_48::DeError>
where
    D: alox_48::DeserializerTrait<'de>,
    T: alox_48::Deserialize<'de>,
{
    struct Visitor<T> {
        _marker: core::marker::PhantomData<T>,
    }

    impl<'de, T> alox_48::Visitor<'de> for Visitor<T>
    where
        T: alox_48::Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a nil padded array")
        }

        fn visit_array<A>(self, mut array: A) -> Result<Self::Value, alox_48::DeError>
        where
            A: alox_48::ArrayAccess<'de>,
        {
            let mut values = Vec::with_capacity(array.len());

            if let Some(v) = array.next_element::<Option<T>>()? {
                if v.is_some() {
                    return Err(alox_48::DeError::custom("the first element was not nil"));
                }
            }

            while let Some(ele) = array.next_element::<T>()? {
                values.push(ele);
            }

            Ok(values)
        }
    }

    deserializer.deserialize(Visitor {
        _marker: core::marker::PhantomData,
    })
}

pub fn serialize_with<S, T>(elements: &[T], serializer: S) -> Result<S::Ok, alox_48::SerError>
where
    S: alox_48::SerializerTrait,
    T: alox_48::Serialize,
{
    use alox_48::SerializeArray;

    let mut array = serializer.serialize_array(elements.len() + 1)?;
    array.serialize_element(&None::<T>)?;

    for v in elements {
        array.serialize_element(v)?;
    }

    array.end()
}
