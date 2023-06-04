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
use crate::prelude::*;
use config::RMVer;

mod rgss2a;
mod rgss3a;
mod rgssad;

#[derive(Default, Debug)]
pub struct Filesystem {
    state: AtomicRefCell<State>,
}

#[derive(Default, Debug)]
pub enum State {
    #[default]
    Unloaded,
    Loading {
        path: vfs::VfsPath,
        project_path: PathBuf,
    },
    Loaded {
        path: vfs::VfsPath,
        project_path: PathBuf,
    },
}

impl Filesystem {
    pub fn read_bytes(&self, path: impl AsRef<str>) -> Result<Vec<u8>, String> {
        let state = self.state.borrow();
        let vfs_path = match &*state {
            State::Loaded { path, .. } => path,
            State::Loading { path, .. } => path,
            State::Unloaded => return Err("Project not loaded".to_string()),
        };

        let mut file = vfs_path
            .join(path)
            .map_err(|e| e.to_string())?
            .open_file()
            .map_err(|e| e.to_string())?;
        let mut buf = vec![];
        file.read_to_end(&mut buf).map_err(|e| e.to_string())?;

        Ok(buf)
    }

    pub fn read_data<T>(&self, path: impl AsRef<str>) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.read_bytes(path)?;

        alox_48::from_bytes(&data).map_err(|e| e.to_string())
    }

    pub fn save_data(&self, path: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> Result<(), String> {
        let state = self.state.borrow();
        let vfs_path = match &*state {
            State::Loaded { path, .. } => path,
            State::Loading { path, .. } => path,
            State::Unloaded => return Err("Project not loaded".to_string()),
        };
        let mut file = vfs_path
            .join(path)
            .map_err(|e| e.to_string())?
            .create_file()
            .map_err(|e| e.to_string())?;
        file.write_all(bytes.as_ref()).map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn create_directory(&self, path: impl AsRef<str>) -> Result<(), String> {
        let state = self.state.borrow();
        let vfs_path = match &*state {
            State::Loaded { path, .. } => path,
            State::Loading { path, .. } => path,
            State::Unloaded => return Err("Project not loaded".to_string()),
        };
        vfs_path
            .join(path)
            .map_err(|e| e.to_string())?
            .create_dir()
            .map_err(|e| e.to_string())
    }

    pub fn path_exists(&self, path: impl AsRef<str>) -> Result<bool, String> {
        let state = self.state.borrow();
        let vfs_path = match &*state {
            State::Loaded { path, .. } => path,
            State::Loading { path, .. } => path,
            State::Unloaded => return Err("Project not loaded".to_string()),
        };
        vfs_path
            .join(path)
            .map_err(|e| e.to_string())?
            .exists()
            .map_err(|e| e.to_string())
    }

    pub fn dir_children(
        &self,
        path: impl AsRef<str>,
    ) -> Result<impl Iterator<Item = camino::Utf8PathBuf> + Send, String> {
        let state = self.state.borrow();
        let vfs_path = match &*state {
            State::Loaded { path, .. } => path,
            State::Loading { path, .. } => path,
            State::Unloaded => return Err("Project not loaded".to_string()),
        };
        vfs_path
            .join(path)
            .map_err(|e| e.to_string())?
            .read_dir()
            .map(|i| i.map(|i| i.as_str().into())) // hnggggg kill me
            .map_err(|e| e.to_string())
    }

    pub fn project_loaded(&self) -> bool {
        matches!(&*self.state.borrow(), State::Loaded { .. })
    }

    pub fn project_path(&self) -> Option<PathBuf> {
        match &*self.state.borrow() {
            State::Loaded { project_path, .. } | State::Loading { project_path, .. } => {
                Some(project_path.clone())
            }
            State::Unloaded => None,
        }
    }

    pub async fn spawn_project_file_picker(&self) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "lumproj"])
            .pick_file()
            .await
        {
            self.load_project(path.path())
        } else {
            Err("Cancelled loading project".to_string())
        }
    }

    pub fn start_loading(&self, project_path: PathBuf) {
        let mut state = self.state.borrow_mut();
        *state = State::Loading {
            path: vfs::PhysicalFS::new(&project_path).into(),
            project_path,
        };
    }

    pub fn load_project(&self, project_path: impl AsRef<Path>) -> Result<(), String> {
        let project_path = project_path.as_ref();
        let project_path = project_path.parent().unwrap_or(project_path).to_path_buf();

        self.start_loading(project_path.clone());

        config::Project::load()?;

        let path = {
            let config = project_config!();
            let archiver = match config.editor_ver {
                RMVer::XP => rgssad::Archiver::new(&project_path).into(),
                RMVer::VX => rgss2a::Archiver::new(&project_path).into(),
                RMVer::Ace => rgss3a::Archiver::new(&project_path).into(),
            };
            let physical_fs = vfs::PhysicalFS::new(&project_path).into();
            if config.prefer_rgssad {
                vfs::OverlayFS::new(&[archiver, physical_fs])
            } else {
                vfs::OverlayFS::new(&[physical_fs, archiver])
            }
            .into()
        };

        *self.state.borrow_mut() = State::Loaded { path, project_path };

        state!().data_cache.load()
    }

    pub fn unload_project(&self) {
        *self.state.borrow_mut() = State::Unloaded;
    }

    pub fn detect_rm_ver(&self) -> Option<RMVer> {
        let state = self.state.borrow();
        let path = match &*state {
            State::Loaded { path, .. } => path,
            State::Loading { path, .. } => path,
            State::Unloaded => return None,
        };

        if path.join("Data/Actors.rxdata").ok()?.exists().ok()? {
            return Some(RMVer::XP);
        }

        if path.join("Data/Actors.rvdata").ok()?.exists().ok()? {
            return Some(RMVer::VX);
        }

        if path.join("Data/Actors.rvdata2").ok()?.exists().ok()? {
            return Some(RMVer::Ace);
        }

        for path in path.read_dir().ok()? {
            if path.as_str().ends_with(".rgssad") {
                return Some(RMVer::XP);
            }

            if path.as_str().ends_with(".rgss2a") {
                return Some(RMVer::VX);
            }

            if path.as_str().ends_with(".rgss3a") {
                return Some(RMVer::Ace);
            }
        }

        None
    }
}
