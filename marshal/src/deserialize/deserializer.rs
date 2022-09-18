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

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<()> //Result<V::Value, Self::Error>
		where
			V: Visitor<'de> {
		Ok(())
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
		where
			V: Visitor<'de> {
		visitor.visit_bool
	}
}

impl<'de> Deserializer<'de> {
	
}