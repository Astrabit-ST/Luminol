// Copyright (C) 2024 Melody Madeline Lyons
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use std::sync::Arc;

use dashmap::{DashMap, DashSet};
use egui::load::{Bytes, BytesPoll, LoadError};

#[derive(Default)]
pub struct Loader {
    loaded_files: DashMap<camino::Utf8PathBuf, Arc<[u8]>>,
    errored_files: DashMap<camino::Utf8PathBuf, color_eyre::Report>,
    unloaded_files: DashSet<camino::Utf8PathBuf>,
}

pub const BYTES_LOADER_ID: &str = egui::load::generate_loader_id!(BytesLoader);

pub const PROTOCOL: &str = "project://";

fn supported_uri_to_path(uri: &str) -> Option<&camino::Utf8Path> {
    uri.strip_prefix(PROTOCOL).map(camino::Utf8Path::new)
}

impl Loader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_unloaded_files(&self, ctx: &egui::Context, filesystem: &impl crate::FileSystem) {
        // dashmap has no drain iterator unfortunately so this is the best we can do
        for path in self.unloaded_files.iter() {
            let path = path.to_path_buf();
            match filesystem.read(&path) {
                Ok(bytes) => {
                    self.loaded_files.insert(path, bytes.into());
                }
                Err(err) => {
                    self.errored_files.insert(path, err);
                }
            };
        }
        if !self.unloaded_files.is_empty() {
            ctx.request_repaint();
        }
        self.unloaded_files.clear();
    }
}

impl egui::load::BytesLoader for Loader {
    fn id(&self) -> &str {
        BYTES_LOADER_ID
    }

    fn load(&self, _: &egui::Context, uri: &str) -> egui::load::BytesLoadResult {
        let Some(path) = supported_uri_to_path(uri) else {
            return Err(LoadError::NotSupported);
        };

        if let Some(bytes) = self.loaded_files.get(path) {
            return Ok(BytesPoll::Ready {
                size: None,
                bytes: Bytes::Shared(bytes.clone()),
                mime: None,
            });
        }

        if let Some(error) = self.errored_files.get(path) {
            return Err(LoadError::Loading(error.to_string()));
        }

        self.unloaded_files.insert(path.to_path_buf());
        Ok(BytesPoll::Pending { size: None })
    }

    fn forget(&self, uri: &str) {
        let Some(path) = supported_uri_to_path(uri) else {
            return;
        };

        self.loaded_files.remove(path);
        self.errored_files.remove(path);
        self.unloaded_files.remove(path);
    }

    fn forget_all(&self) {
        self.loaded_files.clear();
        self.errored_files.clear();
        self.unloaded_files.clear();
    }

    fn byte_size(&self) -> usize {
        self.loaded_files.iter().map(|e| e.len()).sum()
    }
}
