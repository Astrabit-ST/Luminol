use std::ops::{Deref, DerefMut};

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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

/// An array that is serialized and deserialized as padded with a None element.
#[derive(Debug, Clone)]
pub struct NilPadded<T>(pub Vec<T>);

impl<'de, T> serde::Deserialize<'de> for NilPadded<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<T> {
            _marker: core::marker::PhantomData<T>,
        }

        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: serde::Deserialize<'de>,
        {
            type Value = NilPadded<T>;

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

                Ok(values.into())
            }
        }

        deserializer.deserialize_seq(Visitor {
            _marker: core::marker::PhantomData,
        })
    }
}

impl<T> serde::Serialize for NilPadded<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.len() + 1))?;
        seq.serialize_element(&None::<T>)?;

        for v in self.iter() {
            seq.serialize_element(v)?;
        }

        seq.end()
    }
}

impl<T> Deref for NilPadded<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NilPadded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default> Default for NilPadded<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T> From<Vec<Option<T>>> for NilPadded<T> {
    fn from(value: Vec<Option<T>>) -> Self {
        let mut iter = value.into_iter();

        assert!(
            iter.next()
                .expect("there should be at least one element")
                .is_none(),
            "the array should be padded with nil at the first index"
        );
        Self(iter.flatten().collect())
    }
}

impl<T> From<Vec<T>> for NilPadded<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}
