#![allow(dead_code)]
// FIXME: i32 is too big for most values.
// We should use u16 or u8 for most things.
pub mod rpg {
    use eframe::epaint::ahash::HashMap;

    use crate::data::rgss_structs::*;
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct Map {
        tileset_id: i32,
        width: usize,
        height: usize,
        autoplay_bgm: bool,
        bgm: AudioFile,
        autoplay_bgs: bool,
        bgs: AudioFile,
        encounter_list: Vec<i32>,
        encounter_step: i32,
        data: Table3,
        events: HashMap<String, event::Event>,
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct MapInfo {
        name: String,
        parent_id: i32,
        order: i32,
        expanded: bool,
        scroll_x: i32,
        scroll_y: i32,
    }

    // FIXME: Use something else instead of modules to group structs like this.
    mod event {
        use serde::{Deserialize, Serialize};
        mod page {
            use crate::data::rmxp_structs::rpg::EventCommand;
            use crate::data::rmxp_structs::rpg::MoveRoute;
            use serde::{Deserialize, Serialize};

            #[derive(Default, Debug, Deserialize, Serialize)]
            pub struct Condition {
                switch1_valid: bool,
                switch2_valid: bool,
                variable_valid: bool,
                self_switch_valid: bool,
                switch1_id: i32,
                switch2_id: i32,
                variable_id: i32,
                variable_value: i32,
                self_switch_ch: char,
            }

            #[derive(Default, Debug, Deserialize, Serialize)]
            pub struct Graphic {
                tile_id: i32,
                character_name: String,
                character_hue: i32,
                direction: i32,
                pattern: i32,
                opacity: i32,
                blend_type: i32,
            }

            #[derive(Default, Debug, Deserialize, Serialize)]
            pub struct Page {
                conditon: Condition,
                graphic: Graphic,
                move_type: i32,
                move_speed: i32,
                move_frequency: i32,
                move_route: MoveRoute,
                walk_anime: bool,
                step_anime: bool,
                direction_fix: bool,
                through: bool,
                always_on_top: bool,
                trigger: i32,
                list: Vec<EventCommand>,
            }
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Event {
            id: i32,
            name: String,
            x: i32,
            y: i32,
            pages: Vec<page::Page>,
        }
    }

    // FIXME: Add more parameter types, 2 is not enough.
    // TODO: Make commands an enum instead of a struct.
    // This would be better for serialization, performance, and readability.
    // For now I'm not messing with the RMXP data format, but I will eventually.
    // TODO: I'd like to add a custom *.lumina format in the future that is built from the ground up. No more rmxp garbage.
    #[derive(Debug, Deserialize, Serialize)]
    pub enum ParameterType {
        Number(i32),
        String(String),
        Color(Color),
        Tone(Tone),
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct EventCommand {
        code: i32,
        indent: i32,
        parameters: Vec<ParameterType>,
    }
    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct MoveRoute {
        repeat: bool,
        skippable: bool,
        list: Vec<MoveCommand>,
    }
    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct MoveCommand {
        code: i32,
        parameters: Vec<ParameterType>,
    }
    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct Actor {
        id: i32,
        name: String,
        class_id: i32,
        initial_level: i32,
        final_level: i32,
        exp_basis: i32,
        exp_inflation: i32,
        character_name: String,
        character_hue: i32,
        battler_name: String,
        battler_hue: i32,
        parameters: Table2,
        weapon_id: i32,
        armor1_id: i32,
        armor2_id: i32,
        armor3_id: i32,
        armor4_id: i32,
        weapon_fix: bool,
        armor1_fix: bool,
        armor2_fix: bool,
        armor3_fix: bool,
        armor4_fix: bool,
    }

    mod class {
        use crate::data::rgss_structs::Table1;
        use serde::{Deserialize, Serialize};
        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Learning {
            level: i32,
            skill_id: i32,
        }
        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Class {
            id: i32,
            name: String,
            position: i32,
            weapon_set: Vec<i32>,
            armor_set: Vec<i32>,
            element_ranks: Table1,
            state_ranks: Table1,
            learnings: Vec<Learning>,
        }
    }

    // FIXME: I don't use the battle system, so I'm unsure what some of these types *should* be.
    // I plan to support the battle system but that comes after everything else.
    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct Skill {
        id: i32,
        name: String,
        icon_name: String,
        description: String,
        scope: i32,
        occasion: i32,
        animation1_id: i32,
        animation2_id: i32,
        menu_se: AudioFile,
        common_event_id: i32,
        sp_cost: i32,
        power: i32,
        atk_f: i32,
        eva_f: i32,
        str_f: i32,
        dex_f: i32,
        agi_f: i32,
        int_f: i32,
        hit: i32,
        pdef_f: i32,
        mdef_f: i32,
        variance: i32,
        element_set: Vec<i32>,
        plus_state_set: Vec<i32>,
        minus_state_set: Vec<i32>,
    }

    // FIXME: There are a lot of repeated fields here. I should probably make a trait for them.
    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct Item {
        id: i32,
        name: String,
        icon_name: String,
        description: String,
        scope: i32,
        occasion: i32,
        animation1_id: i32,
        animation2_id: i32,
        menu_se: AudioFile,
        common_event_id: i32,
        price: i32,
        consumable: bool,
        parameter_type: i32,
        parameter_points: i32,
        recover_hp_rate: i32,
        recover_hp: i32,
        recover_sp_rate: i32,
        recover_sp: i32,
        hit: i32,
        pdef_f: i32,
        mdef_f: i32,
        variance: i32,
        element_set: Vec<i32>,
        plus_state_set: Vec<i32>,
        minus_state_set: Vec<i32>,
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct Weapon {
        id: i32,
        name: String,
        icon_name: String,
        description: String,
        animation1_id: i32,
        animation2_id: i32,
        price: i32,
        atk: i32,
        pdef: i32,
        mdef: i32,
        str_plus: i32,
        dex_plus: i32,
        agi_plus: i32,
        int_plus: i32,
        element_set: Vec<i32>,
        plus_state_set: Vec<i32>,
        minus_state_set: Vec<i32>,
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct Armor {
        id: i32,
        name: String,
        icon_name: String,
        description: String,
        kind: i32,
        auto_state_id: i32,
        price: i32,
        pdef: i32,
        mdef: i32,
        eva: i32,
        str_plus: i32,
        dex_plus: i32,
        agi_plus: i32,
        int_plus: i32,
        guard_element_set: Vec<i32>,
        guard_state_set: Vec<i32>,
    }

    pub mod enemy {
        use crate::data::rgss_structs::Table1;
        use serde::{Deserialize, Serialize};

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Action {
            kind: i32,
            basic: i32,
            skill_id: i32,
            condition_turn_a: i32,
            condition_turn_b: i32,
            condition_hp: i32,
            condition_level: i32,
            condition_switch_id: i32,
            rating: i32,
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Enemy {
            id: i32,
            name: String,
            battler_name: String,
            battler_hue: i32,
            maxhp: i32,
            maxsp: i32,
            str: i32,
            dex: i32,
            agi: i32,
            int: i32,
            atk: i32,
            pdef: i32,
            mdef: i32,
            eva: i32,
            animation1_id: i32,
            animation2_id: i32,
            element_ranks: Table1,
            state_ranks: Table1,
            actions: Vec<Action>,
            exp: i32,
            gold: i32,
            item_id: i32,
            weapon_id: i32,
            armor_id: i32,
            treasure_prob: i32,
        }
    }

    pub mod troop {
        use serde::{Deserialize, Serialize};
        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Member {
            enemy_id: i32,
            x: i32,
            y: i32,
            hidden: bool,
            immortal: bool,
        }

        pub mod page {
            use crate::data::rmxp_structs::rpg::EventCommand;
            use serde::{Deserialize, Serialize};

            #[derive(Default, Debug, Deserialize, Serialize)]
            pub struct Condition {
                turn_valid: bool,
                enemy_valid: bool,
                actor_valid: bool,
                switch_valid: bool,
                turn_a: i32,
                turn_b: i32,
                enemy_index: i32,
                enemy_hp: i32,
                actor_id: i32,
                actor_hp: i32,
                switch_id: i32,
            }

            #[derive(Default, Debug, Deserialize, Serialize)]
            pub struct Page {
                condition: Condition,
                span: i32,
                list: Vec<EventCommand>,
            }
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Troop {
            id: i32,
            name: String,
            members: Vec<Member>,
            pages: Vec<page::Page>,
        }
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct State {
        id: i32,
        name: String,
        animation_id: i32,
        restriction: i32,
        nonresistance: bool,
        zero_hp: bool,
        cant_get_exp: bool,
        cant_evade: bool,
        slip_damage: bool,
        rating: i32,
        hit_rate: i32,
        maxhp_rate: i32,
        maxsp_rate: i32,
        str_rate: i32,
        dex_rate: i32,
        agi_rate: i32,
        int_rate: i32,
        atk_rate: i32,
        pdef_rate: i32,
        mdef_rate: i32,
        eva: i32,
        battle_only: bool,
        hold_turn: i32,
        auto_release_prob: i32,
        shock_release_prob: i32,
        guard_element_set: Vec<i32>,
        plus_state_set: Vec<i32>,
        minus_state_set: Vec<i32>,
    }

    pub mod animation {
        use crate::data::rgss_structs::{Color, Table2};
        use serde::{Deserialize, Serialize};

        use super::AudioFile;

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Frame {
            cell_max: i32,
            cell_data: Table2,
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Timing {
            frame: i32,
            se: AudioFile,
            flash_scope: i32,
            flash_color: Color,
            flash_duration: i32,
            condition: i32,
        }

        #[derive(Default, Debug, Deserialize, Serialize)]
        pub struct Animation {
            id: i32,
            name: String,
            animation_name: String,
            animation_hue: i32,
            position: i32,
            frame_max: i32,
            frames: Vec<Frame>,
            timings: Vec<i32>,
        }
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct Tileset {
        id: i32,
        name: String,
        tileset_name: String,
        autotile_names: Vec<String>,
        panorama_name: String,
        panorama_hue: i32,
        fog_name: i32,
        fog_hue: i32,
        fog_opacity: i32,
        fog_blend_type: i32,
        fog_zoom: i32,
        fog_sx: i32,
        fog_sy: i32,
        battleback_name: String,
        passages: Table1,
        priorities: Table1,
        terrain_tags: Table1,
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
            magic_number: i32,
            party_members: Vec<i32>,
            elements: Vec<String>,
            switches: Vec<String>,
            variables: Vec<String>,
            windowskin_name: String,
            title_name: String,
            gameover_name: String,
            battle_transition: String,
            title_bgm: AudioFile,
            battle_bgm: AudioFile,
            battle_end_me: AudioFile,
            gameover_me: AudioFile,
            cursor_se: AudioFile,
            decision_se: AudioFile,
            cancel_se: AudioFile,
            buzzer_se: AudioFile,
            equip_se: AudioFile,
            shop_se: AudioFile,
            save_se: AudioFile,
            load_se: AudioFile,
            battle_start_se: AudioFile,
            escape_se: AudioFile,
            actor_collapse_se: AudioFile,
            enemy_collapse_se: AudioFile,
            words: Words,
            test_battlers: Vec<TestBattler>,
            test_troop_id: i32,
            start_map_id: i32,
            start_x: i32,
            start_y: i32,
            battleback_name: String,
            battler_name: String,
            battler_hue: i32,
            edit_map_id: i32,
        }
    }

    #[derive(Default, Debug, Deserialize, Serialize)]
    pub struct AudioFile {
        name: String,
        volume: u8,
        pitch: u8,
    }
}
