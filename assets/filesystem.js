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

    let file_handle = await local_handle.getFileHandle(
        split_path[split_path.length - 1],
        {
            "create": true
        }
    );
    return await file_handle;
}

async function get_dir_handle(path) {
    let local_handle = handle;

    if (path != "") {
        let split_path = path.split("/");
        for (let i = 0; i < split_path.length; i++) {
            local_handle = await local_handle.getDirectoryHandle(
                split_path[i],
                {
                    "create": true,
                }
            );
        }
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

    file = await file.getFile()

    return await file.text();
}

export async function js_read_bytes(path) {
    let file = await get_file_handle(path);

    file = await file.getFile();
    let arraybuffer = await file.arrayBuffer();
    return new Uint8Array(arraybuffer);
}

export async function js_save_data(path, data) {
    let file = await get_file_handle(path);

    let stream = await file.createWritable();
    await stream.write(data);

    await stream.close();
}

export async function js_create_directory(path) {
    path = path.replace(/\/\s*$/, "");

    await get_dir_handle(path);
}

export async function js_create_project_dir(path) {
    handle = await handle.getDirectoryHandle(
        path,
        {
            "create": true,
        }
    );
}

export function js_filesystem_supported() {
    if (typeof window?.showOpenFilePicker === 'function' && typeof FileSystemWritableFileStream == 'function') {
        return true;
    }

    return false;
}