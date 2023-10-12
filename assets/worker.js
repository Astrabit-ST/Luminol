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
import wasm_bindgen, { luminol_worker_start } from '/luminol.js';

onmessage = async function (e) {
    if (e.data[0] === 'init') {
        await wasm_bindgen(undefined, e.data[1]);
        await luminol_worker_start(e.data[2]);
    }
};
