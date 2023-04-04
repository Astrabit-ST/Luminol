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

// Handle to the directory we're working in
let dirHandle;

// Check if the browser supports the File System Access API
export function js_filesystem_supported() {
    if (typeof window?.showOpenFilePicker === 'function' && typeof FileSystemWritableFileStream == 'function') {
        return true;
    }

    return false;
}

// Try opening a folder
export async function tryOpenFolder() {
    dirHandle = await window.showDirectoryPicker({
        mode: "readwrite"
    });

    // return directory name
    return dirHandle.name;
}


// Get the children of a directory
export async function dirChildren(path) {
    let handle = await getDirHandle(path);
    let children = [];

    for await (let entry of handle.values()) {
        children.push(entry.name);
    }

    return children;
}

// Get the bytes of a file
export async function readFile(filename) {
    let handle = await getFileHandle(filename);
    let file = await handle.getFile();
    let contents = await file.arrayBuffer();

    return new Uint8Array(contents);
}

// Write bytes to a file
// Also creates the file if it doesn't exist
export async function writeFile(filename, data) {
    let handle = await getFileHandle(filename, true);
    let writable = await handle.createWritable();

    await writable.write(data);
    await writable.close();
}

// Create a directory
export async function createDir(path) {
    await getDirHandle(path, true);
}

// Get a file handle from a path
async function getFileHandle(filename, create = false) {
    let handle = dirHandle;
    let split = filename.split('/');
    for (let i = 0; i < split.length - 1; i++) {
        let name = split[i];
        handle = await handle.getDirectoryHandle(name, { create });
    }

    handle = await handle.getFileHandle(split[split.length - 1], { create });

    return handle;
}

// Get a directory handle from a path
async function getDirHandle(filename, create = false) {
    let handle = dirHandle;
    let split = filename.split('/');
    for (let i = 0; i < split.length; i++) {
        let name = split[i];
        handle = await handle.getDirectoryHandle(name, { create });
    }

    return handle;
}