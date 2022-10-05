let handle;

export async function js_open_project() {
    const pickerOpts = {
        mode: "readwrite"
    }

    handle = await window.showDirectoryPicker(pickerOpts);
    return handle.name;
}

async function get_file_handle(path) {
    let local_handle = handle;
    let split_path = path.split("/");
    for (let i = 0; i < split_path.length - 1; i++) {
        local_handle = await local_handle.getDirectoryHandle(split_path[i]);
    }

    let file_handle = await local_handle.getFileHandle(split_path[split_path.length - 1]);
    return await file_handle.getFile();
}

async function get_dir_handle(path) {
    let local_handle = handle;
    let split_path = path.split("/");
    for (let i = 0; i < split_path.length; i++) {
        local_handle = await local_handle.getDirectoryHandle(split_path[i]);
    }

    return local_handle;
}

export async function js_dir_children(path) {
    let children = [];
    let handle = await get_dir_handle(path);

    for await (let child of handle.values()) {
        children.push(child.name);
    }

    return children;
}

export async function js_read_file(path) {
    let file = await get_file_handle(path);

    return await file.text();
}

export async function js_read_bytes(path) {
    let file = await get_file_handle(path);

    let arraybuffer = await file.arrayBuffer();
    return new Uint8Array(arraybuffer);
}

export function js_filesystem_supported() {
    if (typeof window?.showOpenFilePicker === 'function') {
        return true;
    }

    return false;
}