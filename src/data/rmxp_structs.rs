#![allow(dead_code)]
pub mod rpg {
    use eframe::epaint::ahash::HashMap;

    use crate::data::rgss_structs::*;

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
        events: HashMap<String, event::Event>
    }


    pub struct MapInfo {
        name: String,
        parent_id: i32,
        order: i32,
        expanded: bool,
        scroll_x : i32,
        scroll_y: i32
    }
    
    mod event {
        pub struct Event {
            id: i32,
            name: String,
            x: i32,
            y: i32,
            pages: Vec<page::Page>
        }

        mod page {
            use crate::data::rmxp_structs::rpg::MoveRoute;
            use crate::data::rmxp_structs::rpg::EventCommand;

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
                list: Vec<EventCommand>
            }

            pub struct Condition {
                switch1_valid: bool,
                switch2_valid: bool,
                variable_valid: bool,
                self_switch_valid: bool,
                switch1_id: i32,
                switch2_id: i32,
                variable_id: i32,
                variable_value: i32,
                self_switch_ch: char
            }

            pub struct Graphic {
                tile_id: i32,
                character_name: String,
                character_hue: i32,
                direction: i32,
                pattern: i32,
                opacity: i32,
                blend_type: i32
            }
        }
    }

    pub struct EventCommand {

    }

    pub struct MoveRoute {

    }

    struct AudioFile {
        name: String,
        volume: u8,
        pitch: u8,
    }
}
