#![allow(unused_variables)]
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
#![allow(dead_code)]
#![allow(unused_must_use)]

use crate::error::{Error, Result};
use num_traits::int::PrimInt;
use serde::{ser, Serialize};

pub struct Serializer {
    output: Vec<u8>,
}

// TODO
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: vec![0x4, 0x8], // The first two bytes of ANY marshal data are 0x4 and 0x8.
                                // Those two butes are a version specifier. Most ruby versions
                                // support version 48. (hence 0x4 0x8)
                                // Marshal version 48 is so old even RMXP uses it.
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl Serializer {
    fn process_digit<T>(&mut self, d: T) -> Result<()>
    where
        T: PrimInt,
    {
        self.output.push(b'i');
        if let Some(mut i) = d.to_i128() {
            if i == 0 {
                self.output.push(0x0);
            } else if 0 < i && i < 123 {
                self.output.push((i + 5) as u8);
            } else if -124 < i && i < 0 {
                self.output.push(((i - 5) & 0xff) as u8);
            }

            let mut chars: Vec<u8> = Vec::new();
            for ii in 0..i32::BITS as i32 {
                chars.push((i & 0xff) as u8);
                i <<= 8;
                if i == 0 {
                    chars[0] = ii as u8;
                    break;
                }
                if i == -1 {
                    chars[0] = -ii as u8;
                    break;
                }
            }
        }
        Ok(())
    }

    fn write_rstr(&mut self, v: &str) {
        for character in v.bytes() {
            self.output.push(character);
        }
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // No additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // ! Implemented

    // This function is easy, because all marshal expects is a T or an F,
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output.push(if v { b'T' } else { b'F' });
        Ok(())
    }

    // An optional is just serialized as its value.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Super simple. An optional None is nil. nil is 0.
    fn serialize_none(self) -> Result<()> {
        self.output.push(0x0);
        Ok(())
    }

    // Maps to nil.
    fn serialize_unit(self) -> Result<()> {
        self.serialize_none()
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.process_digit(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        if v.is_infinite() || v.is_nan() || v == 0.0 {
            if v.is_infinite() {
                if v < 0.0 {
                    self.write_rstr("-inf");
                } else {
                    self.write_rstr("inf");
                }
            } else if v.is_nan() {
                self.write_rstr("nan");
            } else if v == 0.0 {
                self.write_rstr("0");
            }
            return Ok(());
        }

        let dtoa = v.to_string();

        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(v.to_string().as_str())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        for byte in v {
            self.output.push(*byte);
        }
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
