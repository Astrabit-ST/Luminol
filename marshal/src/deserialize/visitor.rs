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
use crate::marshal::deserialize::Deserializer;

impl<'de> Visitor<'de> for Deserializer<'de> {
	type Value = u8;

	fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
		where
			E: de::Error, {
		
	}
}