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

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    struct Visitor<T> {
        _marker: core::marker::PhantomData<T>,
    }

    impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
    where
        T: serde::Deserialize<'de>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a nil padded array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            use serde::de::Error;

            let mut values = Vec::with_capacity(seq.size_hint().unwrap_or(0));

            if let Some(v) = seq.next_element::<Option<T>>()? {
                if v.is_some() {
                    return Err(A::Error::custom("the first element was not nil"));
                }
            }

            while let Some(ele) = seq.next_element::<T>()? {
                values.push(ele);
            }

            Ok(values)
        }
    }

    deserializer.deserialize_seq(Visitor {
        _marker: core::marker::PhantomData,
    })
}

pub fn serialize<S, T>(elements: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: serde::Serialize,
{
    use serde::ser::SerializeSeq;

    let mut seq = serializer.serialize_seq(Some(elements.len() + 1))?;
    seq.serialize_element(&None::<T>)?;

    for v in elements {
        seq.serialize_element(v)?;
    }

    seq.end()
}
