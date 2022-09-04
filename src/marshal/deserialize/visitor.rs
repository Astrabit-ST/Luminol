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