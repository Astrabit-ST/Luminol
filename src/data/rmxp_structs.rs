#![allow(dead_code, missing_docs)]
// FIXME: i32 is too big for most values.
// We should use u16 or u8 for most things.
pub mod rpg {
    use eframe::epaint::ahash::HashMap;

    use crate::data::rgss_structs::*;
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Deserialize, Serialize)]
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
        pub events: HashMap<i32, event::Event>,
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct MapInfo {
        pub name: String,
        pub parent_id: i32,
        pub order: i32,
        pub expanded: bool,
        pub scroll_x: i32,
        pub scroll_y: i32,
    }

    // FIXME: Use something else instead of modules to group structs like this.
    pub mod event {
        use serde::{Deserialize, Serialize};
        mod page {
            use crate::data::rmxp_structs::rpg::EventCommand;
            use crate::data::rmxp_structs::rpg::MoveRoute;
            use serde::{Deserialize, Serialize};

            #[derive(Default, Debug, Deserialize, Serialize, Clone)]
            pub struct Condition {
                pub switch1_valid: bool,
                pub switch2_valid: bool,
                pub variable_valid: bool,
                pub self_switch_valid: bool,
                pub switch1_id: usize,
                pub switch2_id: usize,
                pub variable_id: usize,
                pub variable_value: usize,
                pub self_switch_ch: String,
            }

            #[derive(Default, Debug, Deserialize, Serialize, Clone)]
            pub struct Graphic {
                pub tile_id: i32,
                pub character_name: String,
                pub character_hue: i32,
                pub direction: i32,
                pub pattern: i32,
                pub opacity: i32,
                pub blend_type: i32,
            }

            #[derive(Default, Debug, Deserialize, Serialize, Clone)]
            pub struct Page {
                pub condition: Condition,
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
        }

        #[derive(Default, Debug, Deserialize, Serialize, Clone)]
        pub struct Event {
            pub id: i32,
            pub name: String,
            pub x: i32,
            pub y: i32,
            pub pages: Vec<page::Page>,
        }
    }

    // FIXME: Add more parameter types, 2 is not enough.
    // TODO: Make commands an enum instead of a struct.
    // This would be better for serialization, performance, and readability.
    // For now I'm not messing with the RMXP data format, but I will eventually.
    // TODO: I'd like to add a custom *.lumina format in the future that is built from the ground up. No more rmxp garbage.
    #[derive(Debug, Deserialize, Serialize, Clone)]
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
        TrueClass(bool),
        FalseClass(bool),
    }

    #[derive(Default, Debug, Deserialize, Serialize, Clone)]
    pub struct EventCommand {
        pub code: i32,
        pub indent: i32,
        pub parameters: Vec<ParameterType>,
    }
    #[derive(Default, Debug, Deserialize, Serialize, Clone)]
    pub struct MoveRoute {
        pub repeat: bool,
        pub skippable: bool,
        pub list: Vec<MoveCommand>,
    }
    #[derive(Default, Debug, Deserialize, Serialize, Clone)]
    pub struct MoveCommand {
        pub code: i32,
        pub parameters: Vec<ParameterType>,
    }
    #[derive(Default, Debug, Deserialize, Serialize)]
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

    mod class {
        use crate::data::rgss_structs::Table1;
        use serde::{Deserialize, Serialize};
        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Learning {
            pub level: i32,
            pub skill_id: i32,
        }
        #[derive(Default, Debug, Deserialize, Serialize)]
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
    }

    // FIXME: I don't use the battle system, so I'm unsure what some of these types *should* be.
    // I plan to support the battle system but that comes after everything else.
    #[derive(Default, Debug, Deserialize, Serialize)]
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

    // FIXME: There are a lot of repeated fields here. I should probably make a trait for them.
    #[derive(Default, Debug, Deserialize, Serialize)]
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
        pub recover_sp_rate: i32,
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

    pub mod enemy {
        use crate::data::rgss_structs::Table1;
        use serde::{Deserialize, Serialize};

        #[derive(Default, Debug, Deserialize, Serialize)]
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
    }

    pub mod troop {
        use serde::{Deserialize, Serialize};
        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Member {
            pub enemy_id: i32,
            pub x: i32,
            pub y: i32,
            pub hidden: bool,
            pub immortal: bool,
        }

        pub mod page {
            use crate::data::rmxp_structs::rpg::EventCommand;
            use serde::{Deserialize, Serialize};

            #[derive(Default, Debug, Deserialize, Serialize)]
            pub struct Condition {
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
            pub struct Page {
                pub condition: Condition,
                pub span: i32,
                pub list: Vec<EventCommand>,
            }
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Troop {
            pub id: i32,
            pub name: String,
            pub members: Vec<Member>,
            pub pages: Vec<page::Page>,
        }
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
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

    pub mod animation {
        use crate::data::rgss_structs::{Color, Table2};
        use serde::{Deserialize, Serialize};

        use super::AudioFile;

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Frame {
            pub cell_max: i32,
            pub cell_data: Table2,
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Timing {
            pub frame: i32,
            pub se: AudioFile,
            pub flash_scope: i32,
            pub flash_color: Color,
            pub flash_duration: i32,
            pub condition: i32,
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Animation {
            pub id: i32,
            pub name: String,
            pub animation_name: String,
            pub animation_hue: i32,
            pub position: i32,
            pub frame_max: i32,
            pub frames: Vec<Frame>,
            pub timings: Vec<i32>,
        }
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
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

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct CommonEvent {
        id: i32,
        name: String,
        trigger: i32,
        switch_id: i32,
        list: Vec<EventCommand>,
    }

    pub mod system {
        use super::AudioFile;
        use serde::{Deserialize, Serialize};

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Words {
            gold: String,
            hp: String,
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
        pub struct System {
            pub magic_number: i32,
            pub party_members: Vec<i32>,
            pub elements: Vec<String>,
            pub switches: Vec<String>,
            pub variables: Vec<String>,
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
    }

    #[derive(Default, Debug, Deserialize, Serialize, Clone)]
    pub struct AudioFile {
        name: String,
        volume: u8,
        pitch: u8,
    }
}
