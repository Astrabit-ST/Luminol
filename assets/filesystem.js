let handle;

export async function js_open_project() {
    const pickerOpts = {
        mode: "readwrite"
    }

    handle = await window.showDirectoryPicker(pickerOpts);
    return handle.name;
}

export async function js_read_file(path) {
    try {
        console.log(handle);
        console.log(path);
        console.log(typeof path)

        var file_handle_hjfaajv = await handle.getFileHandle(path);
        console.log(file_handle_hjfaajv);
        let file = await file_handle.getFile();
        console.log(file);

        return await file.text();
    } catch (exception) {
        console.log(exception);
        throw exception;
    }
}

export function js_filesystem_supported() {
    if (typeof window?.showOpenFilePicker === 'function') {
        return true;
    }

    return false;
}