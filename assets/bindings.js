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

export function is_worker() {
    return typeof DedicatedWorkerGlobalScope === 'function'
        && self instanceof DedicatedWorkerGlobalScope;
}

export function worker() {
    return is_worker() ? self : null;
}

// A binding for this attribute was added in July 2023 but hasn't made its way into a release of
// web-sys as of September 2023
export function performance(worker) {
    return worker.performance;
}

export function filesystem_supported() {
    return typeof window?.showOpenFilePicker === 'function'
        && typeof window?.showDirectoryPicker === 'function'
        && typeof FileSystemFileHandle === 'function'
        && typeof FileSystemWritableFileStream === 'function'
        && typeof FileSystemFileHandle?.prototype?.remove === 'function'
        && typeof FileSystemDirectoryHandle?.prototype?.remove === 'function';
}

export async function _show_directory_picker() {
    return await window.showDirectoryPicker({ mode: 'readwrite' });
}

export async function _remove_file(file) {
    await file.remove();
}

export async function _remove_dir(dir) {
    await dir.remove();
}

export function dir_values(dir) {
    return dir.values();
}
