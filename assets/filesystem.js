export async function js_open_project() {
    const pickerOpts = {
      mode: "readwrite"
    }
    
    let result = await window.showDirectoryPicker(pickerOpts);
    return result;
}

export async function js_read_file(handle, path) {
    let file_handle = await handle.getFileHandle(path);
    let file = await file_handle.getFile();
    return await file.text();
}

export function js_filesystem_supported() {
    if (typeof window?.showOpenFilePicker === 'function') {
        return true;
    }

    return false;
}