use crate::UpdateInfo;
use strum::Display;
use strum::EnumIter;
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct Toolbar {
    state: ToolbarState,
}

// TODO: Move to UpdateInfo
#[derive(Default)]
pub struct ToolbarState {
    pub pencil: Pencil,
}

#[derive(Default, EnumIter, Display, PartialEq, Eq, Clone, Copy)]
pub enum Pencil {
    #[default]
    Pen,
    Circle,
    Rectangle,
    Fill,
}

impl Toolbar {
    #[allow(unused_variables)]
    pub fn ui(&mut self, info: &UpdateInfo<'_>, ui: &mut egui::Ui) {
        for e in Pencil::iter() {
            ui.radio_value(&mut self.state.pencil, e, e.to_string());
        }
    }
}
