/// The map picker window.
/// Displays a list of maps in a tree.
/// Maps can be double clicked to open them in a map editor.
use crate::data::rmxp_structs::rpg::MapInfo;

pub struct MapPicker {

}

impl super::window::Window for MapPicker {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        
    }

    fn name(&self) -> String {
        "Map Picker".to_string()
    }
}