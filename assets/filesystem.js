export async function js_open_project() {
    const pickerOpts = {
      mode: "readwrite"
    }
    let [fileHandle] = await window.showDirectoryPicker(pickerOpts);
    return fileHandle;
}

export function js_filesystem_supported() {
    if (typeof window?.showOpenFilePicker === 'function') {
        return true;
    }

    return false;
}