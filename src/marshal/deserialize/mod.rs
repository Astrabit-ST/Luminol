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