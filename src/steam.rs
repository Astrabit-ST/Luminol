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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.
use once_cell::sync::OnceCell;
use parking_lot::Mutex;

const APPID: u32 = 2501490;

pub struct Steamworks {
    pub client: steamworks::Client<steamworks::ClientManager>,
    pub single: Mutex<steamworks::SingleClient<steamworks::ClientManager>>,
}

impl Steamworks {
    pub fn setup() -> Result<(), steamworks::SteamError> {
        let (client, single) = steamworks::Client::init_app(APPID)?;
        let single = Mutex::new(single);

        let steamworks = Steamworks { client, single };

        STEAMWORKS
            .set(steamworks)
            .ok()
            .expect("steamworks already initialized");

        Ok(())
    }

    pub fn get() -> &'static Self {
        STEAMWORKS.get().expect("failed to get steamworks")
    }

    pub fn update() {
        let steamworks = Self::get();
        let single = steamworks.single.lock();

        single.run_callbacks();
    }
}

static STEAMWORKS: OnceCell<Steamworks> = OnceCell::new();
