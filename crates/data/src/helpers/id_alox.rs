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

pub fn deserialize_with<'de, D>(deserializer: D) -> Result<usize, alox_48::DeError>
where
    D: alox_48::DeserializerTrait<'de>,
{
    use alox_48::Deserialize;

    let id = std::num::NonZeroUsize::deserialize(deserializer)?;

    Ok(id.get() - 1)
}

pub fn serialize_with<S>(value: &usize, serializer: S) -> Result<S::Ok, alox_48::SerError>
where
    S: alox_48::SerializerTrait,
{
    use alox_48::Serialize;

    (value + 1).serialize(serializer)
}
