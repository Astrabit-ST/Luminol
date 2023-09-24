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
use crate::helpers::ParameterType;

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(rename = "RPG::MoveRoute")]
pub struct MoveRoute {
    pub repeat: bool,
    pub skippable: bool,
    pub list: Vec<MoveCommand>,
}

impl From<alox_48::Object> for MoveRoute {
    fn from(obj: alox_48::Object) -> Self {
        MoveRoute {
            repeat: obj.fields["repeat"].clone().into_bool().unwrap(),
            skippable: obj.fields["skippable"].clone().into_bool().unwrap(),
            list: obj.fields["list"]
                .clone()
                .into_array()
                .unwrap()
                .into_iter()
                .map(|obj| {
                    let obj = obj.into_object().unwrap();
                    obj.into()
                })
                .collect(),
        }
    }
}

impl From<MoveRoute> for alox_48::Object {
    fn from(value: MoveRoute) -> Self {
        let mut fields = alox_48::value::RbFields::with_capacity(3);
        fields.insert("repeat".into(), alox_48::Value::Bool(value.repeat));
        fields.insert("skippable".into(), alox_48::Value::Bool(value.skippable));
        fields.insert(
            "list".into(),
            alox_48::Value::Array(
                value
                    .list
                    .into_iter()
                    .map(Into::into)
                    .map(alox_48::Value::Object)
                    .collect(),
            ),
        );

        alox_48::Object {
            class: "RPG::MoveRoute".into(),
            fields,
        }
    }
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[allow(missing_docs)]
#[serde(rename = "RPG::MoveCommand")]
pub struct MoveCommand {
    pub code: u16,
    pub parameters: Vec<ParameterType>,

    #[serde(default = "rand::random")]
    #[serde(skip)]
    pub guid: u16,
}

impl From<alox_48::Object> for MoveCommand {
    fn from(obj: alox_48::Object) -> Self {
        MoveCommand {
            code: obj.fields["code"].clone().into_integer().unwrap() as _,
            parameters: obj.fields["parameters"]
                .clone()
                .into_array()
                .unwrap()
                .into_iter()
                .map(Into::into)
                .collect(),

            guid: rand::random(),
        }
    }
}

impl From<MoveCommand> for alox_48::Object {
    fn from(c: MoveCommand) -> Self {
        let mut fields = alox_48::value::RbFields::with_capacity(2);
        fields.insert("code".into(), alox_48::Value::Integer(c.code as _));
        fields.insert(
            "parameters".into(),
            alox_48::Value::Array(c.parameters.into_iter().map(Into::into).collect()),
        );

        alox_48::Object {
            class: "RPG::MoveCommand".into(),
            fields,
        }
    }
}
