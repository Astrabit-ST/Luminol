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
use crate::{optional_path, Path};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[serde(rename = "RPG::AudioFile")]
pub struct AudioFile {
    #[serde(with = "optional_path")]
    pub name: Path,
    pub volume: u8,
    pub pitch: u8,
}

impl From<alox_48::Object> for AudioFile {
    fn from(obj: alox_48::Object) -> Self {
        let name = obj.fields["name"]
            .clone()
            .into_string()
            .unwrap()
            .to_string()
            .unwrap();
        let name = if name.is_empty() {
            None
        } else {
            Some(name.into())
        };
        AudioFile {
            name,
            volume: obj.fields["volume"].clone().into_integer().unwrap() as _,
            pitch: obj.fields["pitch"].clone().into_integer().unwrap() as _,
        }
    }
}

impl From<AudioFile> for alox_48::Object {
    fn from(a: AudioFile) -> Self {
        let mut fields = alox_48::value::RbFields::with_capacity(3);
        fields.insert(
            "name".into(),
            a.name.map(camino::Utf8PathBuf::into_string).into(),
        );
        fields.insert("volume".into(), alox_48::Value::Integer(a.volume as _));
        fields.insert("pitch".into(), alox_48::Value::Integer(a.pitch as _));

        alox_48::Object {
            class: "RPG::AudioFile".into(),
            fields,
        }
    }
}
