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

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/windows-icon.ico");
        res.set("ProductVersion", env!("CARGO_PKG_VERSION"));
        res.set_icon_with_id("assets/rxproj-icon.ico", "2");
        res.set_icon_with_id("assets/rxdata-icon.ico", "3");
        res.set_icon_with_id("assets/lumproj-icon.ico", "4");
        res.set_language(0x0009);

        let _ = res.compile();
    }
}
