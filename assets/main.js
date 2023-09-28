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

// NOTE: Firefox support for `type: 'module'` in Web Workers was added in early June 2023.
const canvas = document.getElementById('luminol-canvas').transferControlToOffscreen();
const worker = new Worker('worker.js', { name: 'luminol-main', type: 'module' });
worker.postMessage({
    type: 'init',
    canvas,
    devicePixelRatio: window.devicePixelRatio,
    prefersColorSchemeDark: window.matchMedia('(prefers-color-scheme: dark)')?.matches,
}, [canvas]);
