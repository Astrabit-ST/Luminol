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

use crate::{
    components::command_view::MOVE_ROUTE,
    data::{
        commands::{MoveCommand::*, MOVE_FREQS, MOVE_SPEEDS},
        rmxp_structs::rpg,
    },
    UpdateInfo,
};

/// Move route display
pub struct MoveDisplay<'route> {
    route: &'route rpg::MoveRoute,
}

impl<'route> MoveDisplay<'route> {
    /// Create a new move display
    pub fn new(route: &'route rpg::MoveRoute) -> Self {
        Self { route }
    }

    /// Display it
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        selected_index: &mut usize,
        index: &mut usize,
        info: &'static UpdateInfo,
    ) {
        let system = info.data_cache.system();
        let system = system.as_ref().unwrap();

        for command in self.route.list.iter() {
            let label = match command {
                                Down => "Move Down".to_string(),
                                Left => "Move Left".to_string(),
                                Right => "Move Right".to_string(),
                                Up => "Move Up".to_string(),
                                LowerLeft => "Move Lower Left".to_string(),
                                LowerRight => "Move Lower Right".to_string(),
                                UpperLeft => "Move Upper Left".to_string(),
                                UpperRight => "Move Upper Right".to_string(),
                                Random => "Move Random".to_string(),
                                MoveTowards => "Move Towards Player".to_string(),
                                MoveAway => "Move Away from Player".to_string(),
                                Forward => "Move Forwards".to_string(),
                                Backwards => "Move Backwards".to_string(),
                                Jump { x_plus, y_plus } => format!("Jump ({x_plus},{y_plus})px"),
                                Wait { time } => format!("Wait {time} frames"),
                                TurnDown => "Turn Down".to_string(),
                                TurnLeft => "Turn Left".to_string(),
                                TurnRight => "Turn Right".to_string(),
                                TurnUp => "Turn Up".to_string(),
                                TurnRight90 => "Turn Right 90deg".to_string(),
                                TurnLeft90 => "Turn Left 90deg".to_string(),
                                Turn180 => "Turn 180deg".to_string(),
                                TurnRightOrLeft => "Turn Right or Left".to_string(),
                                TurnRandom => "Turn Randomly".to_string(),
                                TurnTowardsPlayer => "Turn Towards Player".to_string(),
                                TurnAwayFromPlayer => "Turn Away from Player".to_string(),
                                SwitchON { switch_id } => {
                                    format!("Switch [{switch_id}: {}] ON", system.switches[*switch_id])
                                }
                                SwitchOFF { switch_id } => {
                                    format!("Switch [{switch_id}: {}] OFF", system.switches[*switch_id])
                                }
                                ChangeSpeed { speed } => {
                                    format!("Set Speed to {speed}: {}", MOVE_SPEEDS[*speed - 1])
                                }
                                ChangeFreq { freq } => {
                                    format!("Set Frequency to {freq}: {}", MOVE_FREQS[*freq - 1])
                                }
                                MoveON => "Set Move Animation ON".to_string(),
                                MoveOFF => "Set Move Animation OFF".to_string(),
                                StopON => "Set Stop Animation ON".to_string(),
                                StopOFF => "Set Stop Animation OFF".to_string(),
                                DirFixON => "Set Direction Fix ON".to_string(),
                                DirFixOFF => "Set Direction Fix OFF".to_string(),
                                ThroughON => "Set Through ON".to_string(),
                                ThroughOFF => "Set Through OFF".to_string(),
                                AlwaysTopON => "Set Always on Top ON".to_string(),
                                AlwaysTopOFF => "Set Always on Top OFF".to_string(),
                                ChangeGraphic {
                                    character_name,
                                    character_hue,
                                    direction,
                                    pattern
                                } => format!("Set graphic to '{character_name}' with hue: {character_hue}, direction: {direction}, pattern: {pattern}"),
                                ChangeOpacity { opacity } => format!("Set opacity to {opacity}"),
                                ChangeBlend { blend } => format!(
                                    "Set blend type to {}",
                                    match blend {
                                        0 => "Normal",
                                        1 => "Additive",
                                        2 => "Subtractive",
                                        _ => unreachable!(),
                                    }
                                ),
                                PlaySE { file } => format!(
                                    "Play SE \"{}\", vol: {}, pitch: {}",
                                    file.name, file.volume, file.pitch
                                ),
                                Script { text } => format!("Script: {text}"),

                                Break => "".to_string(),
                                Invalid { code, parameters } => {
                                    format!("Invalid command {code} {:#?}", parameters)
                                }
                            };
            ui.selectable_value(
                selected_index,
                *index,
                egui::RichText::new(format!("$> {label}")).color(MOVE_ROUTE),
            );
            *index += 1;
        }
    }
}
