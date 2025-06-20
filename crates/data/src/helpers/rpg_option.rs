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

/// Wrapper for `Option<T>` that implements serialization and deserialization
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RpgOption<T>(pub Option<T>);

impl<T> From<Option<T>> for RpgOption<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T> From<RpgOption<T>> for Option<T> {
    fn from(value: RpgOption<T>) -> Self {
        value.0
    }
}

impl<'a, T> From<&'a RpgOption<T>> for &'a Option<T> {
    fn from(value: &'a RpgOption<T>) -> Self {
        &value.0
    }
}

impl<'a, T> From<&'a mut RpgOption<T>> for &'a mut Option<T> {
    fn from(value: &'a mut RpgOption<T>) -> Self {
        &mut value.0
    }
}

trait RpgOptionNumber: num::traits::PrimInt + num::traits::Unsigned {}

impl RpgOptionNumber for u8 {}
impl RpgOptionNumber for u16 {}
impl RpgOptionNumber for u32 {}
impl RpgOptionNumber for u64 {}
impl RpgOptionNumber for usize {}

impl<T> serde::Serialize for RpgOption<T>
where
    T: RpgOptionNumber + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(value) = self.0 {
            value.add(T::one()).serialize(serializer)
        } else {
            T::zero().serialize(serializer)
        }
    }
}

impl<T> alox_48::Serialize for RpgOption<T>
where
    T: RpgOptionNumber + alox_48::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, alox_48::SerError>
    where
        S: alox_48::SerializerTrait,
    {
        if let Some(value) = self.0 {
            value.add(T::one()).serialize(serializer)
        } else {
            T::zero().serialize(serializer)
        }
    }
}

impl<'de, T> serde::Deserialize<'de> for RpgOption<T>
where
    T: RpgOptionNumber + serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        T::deserialize(deserializer)
            .map(|value| Self((value != T::zero()).then(|| value.sub(T::one()))))
    }
}

impl<'de, T> alox_48::Deserialize<'de> for RpgOption<T>
where
    T: RpgOptionNumber + alox_48::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, alox_48::DeError>
    where
        D: alox_48::DeserializerTrait<'de>,
    {
        T::deserialize(deserializer)
            .map(|value| Self((value != T::zero()).then(|| value.sub(T::one()))))
    }
}

impl serde::Serialize for RpgOption<String> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.as_deref().unwrap_or_default().serialize(serializer)
    }
}

impl alox_48::Serialize for RpgOption<String> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, alox_48::SerError>
    where
        S: alox_48::SerializerTrait,
    {
        self.0.as_deref().unwrap_or_default().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RpgOption<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|value| Self((!value.is_empty()).then_some(value)))
    }
}

impl<'de> alox_48::Deserialize<'de> for RpgOption<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, alox_48::DeError>
    where
        D: alox_48::DeserializerTrait<'de>,
    {
        String::deserialize(deserializer).map(|value| Self((!value.is_empty()).then_some(value)))
    }
}

impl serde::Serialize for RpgOption<camino::Utf8PathBuf> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_default()
            .serialize(serializer)
    }
}

impl alox_48::Serialize for RpgOption<camino::Utf8PathBuf> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, alox_48::SerError>
    where
        S: alox_48::SerializerTrait,
    {
        self.0
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_default()
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RpgOption<camino::Utf8PathBuf> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .map(|value| Self((!value.is_empty()).then(|| value.into())))
    }
}

impl<'de> alox_48::Deserialize<'de> for RpgOption<camino::Utf8PathBuf> {
    fn deserialize<D>(deserializer: D) -> Result<Self, alox_48::DeError>
    where
        D: alox_48::DeserializerTrait<'de>,
    {
        String::deserialize(deserializer)
            .map(|value| Self((!value.is_empty()).then(|| value.into())))
    }
}
