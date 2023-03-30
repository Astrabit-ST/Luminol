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

use termwiz::caps::Capabilities;
use termwiz::terminal::buffered::BufferedTerminal;
use termwiz::terminal::SystemTerminal;

pub struct Terminal {
    terminal: BufferedTerminal<SystemTerminal>,
}

impl Terminal {
    pub fn new(mut child: std::process::Child) -> Result<Self, termwiz::Error> {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "no stdout on child".to_string())?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "no stdout on child".to_string())?;
        let terminal = SystemTerminal::new_with(Capabilities::new_from_env()?, stdout, stdin)?;
        let terminal = BufferedTerminal::new(terminal)?;

        Ok(Self { terminal })
    }

    pub fn title(&self) -> &str {
        self.terminal.title()
    }

    pub fn ui(ui: &mut egui::Ui) {}
}
