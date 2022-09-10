export async function js_open_project() {
    const pickerOpts = {
      mode: "readwrite"
    }
    
    return window.showDirectoryPicker(pickerOpts).await;
}

export function js_filesystem_supported() {
    if (typeof window?.showOpenFilePicker === 'function') {
        return true;
    }

    return false;
}