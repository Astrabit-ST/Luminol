#![allow(dead_code, missing_docs)]
#![allow(clippy::struct_excessive_bools)]

use std::fmt::Display;

use num_derive::FromPrimitive;
use slab::Slab;
use strum::EnumIter;

use crate::nil_padded::NilPadded;
use crate::rgss_structs::{Color, Table1, Table2, Table3, Tone};
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Map")]
pub struct Map {
    pub tileset_id: i32,
    pub width: usize,
    pub height: usize,
    pub autoplay_bgm: bool,
    pub bgm: AudioFile,
    pub autoplay_bgs: bool,
    pub bgs: AudioFile,
    pub encounter_list: Vec<i32>,
    pub encounter_step: i32,
    pub data: Table3,
    pub events: Slab<Event>,

    #[serde(skip)]
    /// (direction: i32, start_pos: Pos2, route: MoveRoute)
    pub preview_move_route: Option<(i32, MoveRoute)>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::MapInfo")]
pub struct MapInfo {
    pub name: String,
    pub parent_id: i32,
    pub order: i32,
    pub expanded: bool,
    pub scroll_x: i32,
    pub scroll_y: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "RPG::Event::Page::Condition")]
pub struct EventCondition {
    pub switch1_valid: bool,
    pub switch2_valid: bool,
    pub variable_valid: bool,
    pub self_switch_valid: bool,
    pub switch1_id: usize,
    pub switch2_id: usize,
    pub variable_id: usize,
    pub variable_value: i32,
    pub self_switch_ch: String,
}

impl Default for EventCondition {
    fn default() -> Self {
        Self {
            switch1_valid: false,
            switch2_valid: false,
            variable_valid: false,
            self_switch_valid: false,
            switch1_id: 1,
            switch2_id: 1,
            variable_id: 1,
            variable_value: 0,
            self_switch_ch: "A".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "RPG::Event::Page::Graphic")]
pub struct Graphic {
    pub tile_id: i32,
    pub character_name: String,
    pub character_hue: i32,
    pub direction: i32,
    pub pattern: i32,
    pub opacity: i32,
    pub blend_type: i32,
}

impl Default for Graphic {
    fn default() -> Self {
        Self {
            tile_id: 0,
            character_name: String::new(),
            character_hue: 0,
            direction: 2,
            pattern: 0,
            opacity: 255,
            blend_type: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "RPG::Event::Page")]
pub struct EventPage {
    pub condition: EventCondition,
    pub graphic: Graphic,
    pub move_type: usize,
    pub move_speed: usize,
    pub move_frequency: usize,
    pub move_route: MoveRoute,
    pub walk_anime: bool,
    pub step_anime: bool,
    pub direction_fix: bool,
    pub through: bool,
    pub always_on_top: bool,
    pub trigger: i32,
    pub list: Vec<EventCommand>,
}

impl Default for EventPage {
    fn default() -> Self {
        Self {
            condition: EventCondition::default(),
            graphic: Graphic::default(),
            move_type: 0,
            move_speed: 3,
            move_frequency: 3,
            move_route: MoveRoute::default(),
            walk_anime: true,
            step_anime: false,
            direction_fix: false,
            through: false,
            always_on_top: false,
            trigger: 0,
            list: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "RPG::Event")]
pub struct Event {
    pub id: usize,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub pages: Vec<EventPage>,
}

impl Event {
    #[must_use]
    pub fn new(x: i32, y: i32, id: usize) -> Self {
        Self {
            id,
            name: format!("EV{id:0>3}"),
            x,
            y,
            pages: vec![EventPage::default()],
        }
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq)]
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

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Actor")]
pub struct Actor {
    pub id: i32,
    pub name: String,
    pub class_id: i32,
    pub initial_level: i32,
    pub final_level: i32,
    pub exp_basis: i32,
    pub exp_inflation: i32,
    pub character_name: String,
    pub character_hue: i32,
    pub battler_name: String,
    pub battler_hue: i32,
    pub parameters: Table2,
    pub weapon_id: i32,
    pub armor1_id: i32,
    pub armor2_id: i32,
    pub armor3_id: i32,
    pub armor4_id: i32,
    pub weapon_fix: bool,
    pub armor1_fix: bool,
    pub armor2_fix: bool,
    pub armor3_fix: bool,
    pub armor4_fix: bool,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Class::Learning")]
pub struct Learning {
    pub level: i32,
    pub skill_id: i32,
}
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Class")]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub position: i32,
    pub weapon_set: Vec<i32>,
    pub armor_set: Vec<i32>,
    pub element_ranks: Table1,
    pub state_ranks: Table1,
    pub learnings: Vec<Learning>,
}

// FIXME: I don't use the battle system, so I'm unsure what some of these types *should* be.
// I plan to support the battle system but that comes after everything else.
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Skill")]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub icon_name: String,
    pub description: String,
    pub scope: i32,
    pub occasion: i32,
    pub animation1_id: i32,
    pub animation2_id: i32,
    pub menu_se: AudioFile,
    pub common_event_id: i32,
    pub sp_cost: i32,
    pub power: i32,
    pub atk_f: i32,
    pub eva_f: i32,
    pub str_f: i32,
    pub dex_f: i32,
    pub agi_f: i32,
    pub int_f: i32,
    pub hit: i32,
    pub pdef_f: i32,
    pub mdef_f: i32,
    pub variance: i32,
    pub element_set: Vec<i32>,
    pub plus_state_set: Vec<i32>,
    pub minus_state_set: Vec<i32>,
}

#[derive(Debug, EnumIter, FromPrimitive)]
pub enum ItemScope {
    None,
    OneEnemy,
    AllEnemies,
    OneAlly,
    AllAllies,
    OneAllyHP0,
    AllAlliesHP0,
    TheUser,
}
impl Display for ItemScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ItemScope::{
            AllAllies, AllAlliesHP0, AllEnemies, None, OneAlly, OneAllyHP0, OneEnemy, TheUser,
        };

        write!(
            f,
            "{}",
            match self {
                None => "None",
                OneEnemy => "One Enemy",
                AllEnemies => "All Enemies",
                OneAlly => "One Ally",
                AllAllies => "All Allies",
                OneAllyHP0 => "One Ally (HP 0)",
                AllAlliesHP0 => "All Allies (HP 0)",
                TheUser => "The User",
            }
        )
    }
}

#[derive(Debug, FromPrimitive, EnumIter)]
pub enum ItemOccasion {
    Always,
    OnlyInBattle,
    OnlyFromTheMenu,
    Never,
}
impl Display for ItemOccasion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ItemOccasion::*;

        write!(
            f,
            "{}",
            match self {
                Always => "Always",
                OnlyInBattle => "Only in Battle",
                OnlyFromTheMenu => "Only from the Menu",
                Never => "Never",
            },
        )
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "RPG::Item")]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub icon_name: String,
    pub description: String,
    pub scope: i32,
    pub occasion: i32,
    pub animation1_id: i32,
    pub animation2_id: i32,
    pub menu_se: AudioFile,
    pub common_event_id: i32,
    pub price: i32,
    pub consumable: bool,
    pub parameter_type: i32,
    pub parameter_points: i32,
    pub recover_hp_rate: i32,
    pub recover_hp: i32,
    // These fields are missing in rmxp data *sometimes*.
    // Why? Who knows!
    #[serde(default)]
    pub recover_sp_rate: i32,
    #[serde(default)]
    pub recover_sp: i32,
    pub hit: i32,
    pub pdef_f: i32,
    pub mdef_f: i32,
    pub variance: i32,
    pub element_set: Vec<i32>,
    pub plus_state_set: Vec<i32>,
    pub minus_state_set: Vec<i32>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Weapon")]
pub struct Weapon {
    pub id: i32,
    pub name: String,
    pub icon_name: String,
    pub description: String,
    pub animation1_id: i32,
    pub animation2_id: i32,
    pub price: i32,
    pub atk: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub str_plus: i32,
    pub dex_plus: i32,
    pub agi_plus: i32,
    pub int_plus: i32,
    pub element_set: Vec<i32>,
    pub plus_state_set: Vec<i32>,
    pub minus_state_set: Vec<i32>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Armor")]
pub struct Armor {
    pub id: i32,
    pub name: String,
    pub icon_name: String,
    pub description: String,
    pub kind: i32,
    pub auto_state_id: i32,
    pub price: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub eva: i32,
    pub str_plus: i32,
    pub dex_plus: i32,
    pub agi_plus: i32,
    pub int_plus: i32,
    pub guard_element_set: Vec<i32>,
    pub guard_state_set: Vec<i32>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Enemy::Action")]
pub struct Action {
    pub kind: i32,
    pub basic: i32,
    pub skill_id: i32,
    pub condition_turn_a: i32,
    pub condition_turn_b: i32,
    pub condition_hp: i32,
    pub condition_level: i32,
    pub condition_switch_id: i32,
    pub rating: i32,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Enemy")]
pub struct Enemy {
    pub id: i32,
    pub name: String,
    pub battler_name: String,
    pub battler_hue: i32,
    pub maxhp: i32,
    pub maxsp: i32,
    pub str: i32,
    pub dex: i32,
    pub agi: i32,
    pub int: i32,
    pub atk: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub eva: i32,
    pub animation1_id: i32,
    pub animation2_id: i32,
    pub element_ranks: Table1,
    pub state_ranks: Table1,
    pub actions: Vec<Action>,
    pub exp: i32,
    pub gold: i32,
    pub item_id: i32,
    pub weapon_id: i32,
    pub armor_id: i32,
    pub treasure_prob: i32,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Troop::Member")]
pub struct Member {
    pub enemy_id: i32,
    pub x: i32,
    pub y: i32,
    pub hidden: bool,
    pub immortal: bool,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Troop::Page::Condition")]
pub struct TroopCondition {
    pub turn_valid: bool,
    pub enemy_valid: bool,
    pub actor_valid: bool,
    pub switch_valid: bool,
    pub turn_a: i32,
    pub turn_b: i32,
    pub enemy_index: i32,
    pub enemy_hp: i32,
    pub actor_id: i32,
    pub actor_hp: i32,
    pub switch_id: i32,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Troop::Page")]
pub struct TroopPage {
    pub condition: TroopCondition,
    pub span: i32,
    pub list: Vec<EventCommand>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Troop")]
pub struct Troop {
    pub id: i32,
    pub name: String,
    pub members: Vec<Member>,
    pub pages: Vec<TroopPage>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::State")]
pub struct State {
    pub id: i32,
    pub name: String,
    pub animation_id: i32,
    pub restriction: i32,
    pub nonresistance: bool,
    pub zero_hp: bool,
    pub cant_get_exp: bool,
    pub cant_evade: bool,
    pub slip_damage: bool,
    pub rating: i32,
    pub hit_rate: i32,
    pub maxhp_rate: i32,
    pub maxsp_rate: i32,
    pub str_rate: i32,
    pub dex_rate: i32,
    pub agi_rate: i32,
    pub int_rate: i32,
    pub atk_rate: i32,
    pub pdef_rate: i32,
    pub mdef_rate: i32,
    pub eva: i32,
    pub battle_only: bool,
    pub hold_turn: i32,
    pub auto_release_prob: i32,
    pub shock_release_prob: i32,
    pub guard_element_set: Vec<i32>,
    pub plus_state_set: Vec<i32>,
    pub minus_state_set: Vec<i32>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Animation::Frame")]
pub struct Frame {
    pub cell_max: i32,
    pub cell_data: Table2,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Animation::Timing")]
pub struct Timing {
    pub frame: i32,
    pub se: AudioFile,
    pub flash_scope: i32,
    pub flash_color: Color,
    pub flash_duration: i32,
    pub condition: i32,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Animation")]
pub struct Animation {
    pub id: i32,
    pub name: String,
    pub animation_name: String,
    pub animation_hue: i32,
    pub position: i32,
    pub frame_max: i32,
    pub frames: Vec<Frame>,
    pub timings: Vec<Timing>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::Tileset")]
pub struct Tileset {
    pub id: i32,
    pub name: String,
    pub tileset_name: String,
    pub autotile_names: Vec<String>,
    pub panorama_name: String,
    pub panorama_hue: i32,
    pub fog_name: String,
    pub fog_hue: i32,
    pub fog_opacity: i32,
    pub fog_blend_type: i32,
    pub fog_zoom: i32,
    pub fog_sx: i32,
    pub fog_sy: i32,
    pub battleback_name: String,
    pub passages: Table1,
    pub priorities: Table1,
    pub terrain_tags: Table1,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
#[serde(rename = "RPG::CommonEvent")]
pub struct CommonEvent {
    pub id: usize,
    pub name: String,
    pub trigger: usize,
    pub switch_id: usize,
    pub list: Vec<EventCommand>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
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

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(rename = "RPG::System::TestBattler")]
pub struct TestBattler {
    actor_id: i32,
    level: i32,
    weapon_id: i32,
    armor1_id: i32,
    armor2_id: i32,
    armor3_id: i32,
    armor4_id: i32,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(default)] // ??? rmxp???
#[serde(rename = "RPG::System")]
pub struct System {
    pub magic_number: i32,
    pub party_members: Vec<i32>,
    pub elements: Vec<String>,
    pub switches: NilPadded<String>,
    pub variables: NilPadded<String>,
    pub windowskin_name: String,
    pub title_name: String,
    pub gameover_name: String,
    pub battle_transition: String,
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
    #[serde(skip_deserializing)]
    pub test_battlers: Vec<TestBattler>,
    pub test_troop_id: i32,
    pub start_map_id: i32,
    pub start_x: i32,
    pub start_y: i32,
    pub battleback_name: String,
    pub battler_name: String,
    pub battler_hue: i32,
    pub edit_map_id: i32,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename = "RPG::AudioFile")]
pub struct AudioFile {
    pub name: String,
    pub volume: u8,
    pub pitch: u8,
}

impl From<alox_48::Object> for AudioFile {
    fn from(obj: alox_48::Object) -> Self {
        AudioFile {
            name: obj.fields["name"]
                .clone()
                .into_string()
                .unwrap()
                .to_string()
                .unwrap(),
            volume: obj.fields["volume"].clone().into_integer().unwrap() as _,
            pitch: obj.fields["pitch"].clone().into_integer().unwrap() as _,
        }
    }
}

impl From<AudioFile> for alox_48::Object {
    fn from(a: AudioFile) -> Self {
        let mut fields = alox_48::value::RbFields::with_capacity(3);
        fields.insert("name".into(), a.name.into());
        fields.insert("volume".into(), alox_48::Value::Integer(a.volume as _));
        fields.insert("pitch".into(), alox_48::Value::Integer(a.pitch as _));

        alox_48::Object {
            class: "RPG::AudioFile".into(),
            fields,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
#[serde(rename = "RPG::EventCommand")]
pub struct EventCommand {
    pub code: i32,
    pub indent: usize,
    pub parameters: Vec<ParameterType>,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq)]
#[allow(missing_docs)]
#[serde(rename = "RPG::MoveCommand")]
pub struct MoveCommand {
    pub code: i32,
    pub parameters: Vec<ParameterType>,
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

// FIXME: add custom serialize impl
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct Script {
    pub id: usize,
    pub name: String,
    pub data: Vec<u8>,
}

impl<'de> serde::Deserialize<'de> for Script {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Script;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                use serde::de::Error;

                let Some(id) = seq.next_element()? else {
                        return Err(A::Error::missing_field("id"));
                    };

                let Some(name) = seq.next_element()? else {
                        return Err(A::Error::missing_field("name"));
                    };

                let Some(data) = seq.next_element::<alox_48::RbString>()? else {
                        return Err(A::Error::missing_field("data"));
                    };

                Ok(Script {
                    id,
                    name,
                    data: data.data,
                })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl Serialize for Script {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(3))?;

        seq.serialize_element(&self.id)?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&alox_48::RbString {
            data: self.data.clone(),
            ..Default::default()
        })?;

        seq.end()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, EnumAsInner, PartialEq)]
#[allow(missing_docs)]
#[serde(from = "alox_48::Value")]
#[serde(into = "alox_48::Value")]
pub enum ParameterType {
    Integer(i32),
    String(String),
    Color(Color),
    Tone(Tone),
    AudioFile(AudioFile),
    Float(f32),
    MoveRoute(MoveRoute),
    MoveCommand(MoveCommand),
    Array(Vec<String>),
    Bool(bool),
}

impl From<alox_48::Value> for ParameterType {
    fn from(value: alox_48::Value) -> Self {
        match value {
            alox_48::Value::Integer(i) => Self::Integer(i as _),
            alox_48::Value::String(str) => Self::String(str.to_string().unwrap()),
            // Value::Symbol(sym) => Self::String(sym),
            alox_48::Value::Object(obj) if obj.class == "RPG::AudioFile" => {
                Self::AudioFile(obj.into())
            }
            alox_48::Value::Object(obj) if obj.class == "RPG::MoveRoute" => {
                Self::MoveRoute(obj.into())
            }
            alox_48::Value::Object(obj) if obj.class == "RPG::MoveCommand" => {
                Self::MoveCommand(obj.into())
            }
            alox_48::Value::Float(f) => Self::Float(f as _),
            alox_48::Value::Array(ary) => Self::Array(
                ary.into_iter()
                    .map(|v| v.into_string().unwrap().to_string().unwrap())
                    .collect(),
            ),
            alox_48::Value::Bool(b) => Self::Bool(b),
            alox_48::Value::Userdata(data) if data.class == "Color" => {
                Self::Color(Color::from(data))
            }
            alox_48::Value::Userdata(data) if data.class == "Tone" => Self::Tone(Tone::from(data)),
            _ => panic!("Unexpected type {value:#?}"),
        }
    }
}

impl From<ParameterType> for alox_48::Value {
    fn from(value: ParameterType) -> Self {
        match value {
            ParameterType::Integer(i) => alox_48::Value::Integer(i as _),
            ParameterType::String(s) => alox_48::Value::String(s.into()),
            ParameterType::Color(c) => c.into(),
            ParameterType::Tone(t) => t.into(),
            ParameterType::Float(f) => alox_48::Value::Float(f as _),
            ParameterType::Array(a) => {
                alox_48::Value::Array(a.into_iter().map(Into::into).collect())
            }
            ParameterType::Bool(b) => alox_48::Value::Bool(b),

            ParameterType::MoveRoute(r) => alox_48::Value::Object(r.into()),
            ParameterType::MoveCommand(c) => alox_48::Value::Object(c.into()),
            ParameterType::AudioFile(a) => alox_48::Value::Object(a.into()),
        }
    }
}

impl From<String> for ParameterType {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}
