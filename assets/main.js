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

// Check if the user's browser supports WebGPU
console.log('Checking for WebGPU support…');
let gpu = false;
try {
    let adapter = await navigator.gpu?.requestAdapter();
    gpu = typeof GPUAdapter === 'function' && adapter instanceof GPUAdapter;
} catch (e) {}
if (gpu) {
    console.log('WebGPU is supported. Using WebGPU backend if available.');
} else {
    console.log('No support detected. Using WebGL backend if available.');
}

// If WebGPU is supported, always use luminol.js
// If WebGPU is not supported, use luminol_webgl.js if it's available or fallback to luminol.js
let fallback = false;
let luminol;
if (gpu) {
    luminol = await import('/luminol.js');
} else {
    try {
        luminol = await import('/luminol_webgl.js');
        fallback = true;
    } catch (e) {
        luminol = await import ('/luminol.js');
    }
}

await luminol.default(fallback ? '/luminol_webgl_bg.wasm' : '/luminol_bg.wasm');
luminol.luminol_main_start(fallback);
