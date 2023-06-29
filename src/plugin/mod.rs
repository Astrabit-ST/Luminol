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
use std::env;

pub mod loader;
pub mod result;
pub mod ui;

#[macro_export]
macro_rules! global_data_path {
    () => {{
        let appdata = $crate::plugin::get_application_data_path();
        let mut buffer = PathBuf::from(appdata);
        buffer.push("Astrabit Studios");
        buffer.push("Luminol");
        buffer
    }};
}

fn get_application_data_path() -> String {
    let mut home_directory = env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap();

    home_directory.push_str(if cfg!(windows) {
        "\\AppData\\LocalLow"
    } else {
        "/.local"
    });

    home_directory
}
