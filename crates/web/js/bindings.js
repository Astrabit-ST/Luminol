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

export function filesystem_supported() {
    return typeof window?.showOpenFilePicker === 'function'
        && typeof window?.showDirectoryPicker === 'function'
        && typeof FileSystemFileHandle === 'function'
        && typeof FileSystemWritableFileStream === 'function';
}

export async function _show_directory_picker() {
    return await window.showDirectoryPicker({ mode: 'readwrite' });
}

export async function _show_file_picker(filter_name, extensions) {
    return (await window.showOpenFilePicker({
        types: [{
            description: filter_name,
            accept: { 'application/x-empty': extensions },
        }],
        excludeAcceptAllOption: true,
    }))[0];
}

export function dir_values(dir) {
    return dir.values();
}

export async function _request_permission(handle) {
    if (typeof window?.__FILE_SYSTEM_TOOLS__?.parseHandle === 'function') {
        // If the user is using https://github.com/ichaoX/ext-file without enabling `FS_CONFIG.CLONE_ENABLED`,
        // this is required to restore the `.requestPermission` method on a dir handle restored from IndexedDB
        handle = window.__FILE_SYSTEM_TOOLS__.parseHandle(handle);
    }
    return (await handle.requestPermission({ mode: 'readwrite' })) === 'granted';
}

export function cross_origin_isolated() {
    return crossOriginIsolated === true;
}
