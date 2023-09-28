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
pub use crate::{id, id_vec, nil_padded, optional_id, optional_path, rpg::AudioFile, Path};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)] // ??? rmxp???
#[serde(rename = "RPG::System")]
pub struct System {
    pub magic_number: i32,
    #[serde(with = "id_vec")]
    pub party_members: Vec<usize>,
    pub elements: Vec<String>,
    #[serde(with = "nil_padded")]
    pub switches: Vec<String>,
    #[serde(with = "nil_padded")]
    pub variables: Vec<String>,

    #[serde(with = "optional_path")]
    pub windowskin_name: Path,
    #[serde(with = "optional_path")]
    pub title_name: Path,
    #[serde(with = "optional_path")]
    pub gameover_name: Path,
    #[serde(with = "optional_path")]
    pub battle_transition: Path,
    pub title_bgm: AudioFile,
    pub battle_bgm: AudioFile,
    pub battle_end_me: AudioFile,
    pub gameover_me: AudioFile,
    pub cursor_se: AudioFile,
    pub decision_se: AudioFile,
    pub cancel_se: AudioFile,
    pub buzzer_se: AudioFile,
    pub equip_se: AudioFile,
    pub shop_se: AudioFile,
    pub save_se: AudioFile,
    pub load_se: AudioFile,
    pub battle_start_se: AudioFile,
    pub escape_se: AudioFile,
    pub actor_collapse_se: AudioFile,
    pub enemy_collapse_se: AudioFile,
    pub words: Words,
    // #[serde(skip_deserializing)]
    pub test_battlers: Vec<TestBattler>,
    #[serde(with = "optional_id")]
    pub test_troop_id: Option<usize>,
    #[serde(with = "id")]
    pub start_map_id: usize,
    pub start_x: i32,
    pub start_y: i32,
    #[serde(with = "optional_path")]
    pub battleback_name: Path,
    #[serde(with = "optional_path")]
    pub battler_name: Path,
    pub battler_hue: i32,
    pub edit_map_id: usize,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::System::Words")]
#[serde(default)]
pub struct Words {
    gold: String,
    hp: String,
    sp: String,
    str: String,
    dex: String,
    agi: String,
    int: String,
    atk: String,
    pdef: String,
    mdef: String,
    weapon: String,
    armor1: String,
    armor2: String,
    armor3: String,
    armor4: String,
    attack: String,
    skill: String,
    guard: String,
    item: String,
    equip: String,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::System::TestBattler")]
pub struct TestBattler {
    level: i32,

    #[serde(with = "id")]
    actor_id: usize,
    #[serde(with = "optional_id")]
    weapon_id: Option<usize>,
    #[serde(with = "optional_id")]
    armor1_id: Option<usize>,
    #[serde(with = "optional_id")]
    armor2_id: Option<usize>,
    #[serde(with = "optional_id")]
    armor3_id: Option<usize>,
    #[serde(with = "optional_id")]
    armor4_id: Option<usize>,
}
