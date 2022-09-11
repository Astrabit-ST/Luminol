use std::cell::RefCell;
use std::path::PathBuf;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use super::data_cache::DataCache;

/// Arbitrary stack size of 50kib.
const ASYNCIFY_STACK_SIZE: usize = 50 * 1024;
/// Scratch space used by Asyncify to save/restore stacks.
static ASYNCIFY_STACK: [u8; ASYNCIFY_STACK_SIZE] = [0; ASYNCIFY_STACK_SIZE];

#[no_mangle]
extern "C" fn get_asyncify_stack_space_ptr() -> i32 {
    ASYNCIFY_STACK.as_ptr() as i32
}

#[no_mangle]
extern "C" fn get_asyncify_stack_space_size() -> i32 {
    ASYNCIFY_STACK_SIZE as i32
}

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    fn js_open_project() -> JsValue;
    fn js_filesystem_supported() -> bool;
}

pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    handle: RefCell<Option<JsValue>>,
}

impl Filesystem {
    pub fn new() -> Self {
        if !js_filesystem_supported() {
            rfd::MessageDialog::new()
                .set_description("Filesystem not supported on this browser")
                .show();
            panic!("Filesystem not supported on this browser");
        }
        Self {
            project_path: RefCell::new(None),
            handle: RefCell::new(None),
        }
    }

    pub fn unload_project(&self) {
        *self.project_path.borrow_mut() = None;
    }

    pub fn project_loaded(&self) -> bool {
        self.project_path.borrow().is_some()
    }

    pub fn project_path(&self) -> Option<PathBuf> {
        self.project_path.borrow().clone()
    }

    pub fn load_project(&self, path: PathBuf, cache: &DataCache) {
        *self.project_path.borrow_mut() = Some(path);
        cache.load(self);
    }

    pub fn read_data<T>(&self, _path: &str) -> Result<T, &str>
    where
        T: serde::de::DeserializeOwned,
    {
        Err("NYI for wasm32")
    }

    pub fn save_cached(&self, data_cache: &super::data_cache::DataCache) {
        data_cache.save();
    }

    pub fn try_open_project(&self, cache: &DataCache) {
        *self.handle.borrow_mut() = Some(js_open_project());
        self.load_project(PathBuf::from("Project"), cache);
    }
}
