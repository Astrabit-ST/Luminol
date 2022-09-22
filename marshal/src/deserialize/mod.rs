// Copyright (C) 2022 Lily Lyons
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

use crate::marshal::error::{Error, Result};
use serde::{
	Deserialize,
	de::{
		self,
		DeserializeSeed,
		EnumAccess,
		IntoDeserializer,
		MapAccess,
		SeqAccess,
		VariantAccess,
		Visitor
	}
};

pub struct Deserializer<'de> {
	input: &'de [u8]
}

impl <'de> Deserializer<'de> {
	pub fn from_str(input: &'de str) -> Self {
		Deserializer { input: input.as_bytes() }
	}

	pub fn from_bytes(input: &'de [u8]) -> Self {
		Deserializer { input }
	}
}

pub fn from_bytes<'a, T>(s: &'a [u8]) -> Result<T>
where 
	T: Deserialize<'a>,
{
	if s[0] != 0x4 && s[1] != 0x8 {
		return Err(Error::IncompatibleVersion(format!("{}.{}", s[0] as char, s[1] as char)))
	}

	let mut deserializer = Deserializer::from_bytes(s);
	let t = T::deserialize(&mut deserializer)?;
	if deserializer.input.is_empty() {
		Ok(t)
	} else {
		Err(Error::FormatError)
	}
}

pub mod visitor;
pub mod deserializer;