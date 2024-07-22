// Copyright (C) 2024 Melody Madeline Lyons
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

const APPID: u32 = 2501490;

pub struct Steamworks {
    pub single: parking_lot::Mutex<steamworks::SingleClient<steamworks::ClientManager>>,
}

impl Steamworks {
    pub fn new() -> Result<Self, steamworks::SteamError> {
        let (_, single) = steamworks::Client::init_app(APPID)?;
        let single = parking_lot::Mutex::new(single);

        let steamworks = Steamworks { single };

        Ok(steamworks)
    }

    pub fn update(&self) {
        let single = self.single.lock();

        single.run_callbacks();
    }
}
