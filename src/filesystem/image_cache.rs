use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use egui_extras::RetainedImage;

use super::Filesystem;

#[derive(Default)]
pub struct ImageCache {
    images: RefCell<HashMap<String, RetainedImage>>,
}

impl ImageCache {
    pub fn load_image(&self, path: String, filesystem: &Filesystem) -> RefMut<'_, RetainedImage> {
        RefMut::map(self.images.borrow_mut(), |images| {
            images.entry(path.clone()).or_insert_with(|| {
                egui_extras::RetainedImage::from_image_bytes(
                    "",
                    &filesystem
                        .read_bytes(&format!("{}.png", path))
                        .unwrap_or_else(|_| {
                            filesystem
                                .read_bytes(&format!("{}.jpg", path))
                                .unwrap_or_else(|_| {
                                    filesystem
                                        .read_bytes(&format!("{}.jpg", path))
                                        .expect("Failed to read image from path")
                                })
                        }),
                )
                .expect("Failed to load image")
            })
        })
    }
}
