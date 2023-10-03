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

use super::FileSystem as FileSystemTrait;
use super::{archiver, host, list, path_cache, DirEntry, Error, Metadata, OpenFlags};

#[cfg(target_arch = "wasm32")]
use super::web;

#[derive(Default)]
pub struct FileSystem {
    state: AtomicRefCell<State>,
}

#[derive(Default)]
enum State {
    #[default]
    Unloaded,
    #[cfg(not(target_arch = "wasm32"))]
    HostLoaded(host::FileSystem),
    #[cfg(target_arch = "wasm32")]
    HostLoaded(web::FileSystem),
    Loaded {
        filesystem: path_cache::FileSystem<list::FileSystem>,
        project_path: camino::Utf8PathBuf,
    },
}

#[ouroboros::self_referencing]
pub struct File<'fs> {
    state: AtomicRef<'fs, State>,
    #[borrows(state)]
    #[not_covariant]
    file: FileType<'this>,
}

enum FileType<'fs> {
    #[cfg(not(target_arch = "wasm32"))]
    Host(<host::FileSystem as FileSystemTrait>::File<'fs>),
    #[cfg(target_arch = "wasm32")]
    Host(<web::FileSystem as FileSystemTrait>::File<'fs>),
    Loaded(<path_cache::FileSystem<list::FileSystem> as FileSystemTrait>::File<'fs>),
}

impl FileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_data<T>(&self, path: impl AsRef<camino::Utf8Path>) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.read(path).map_err(|e| e.to_string())?;

        alox_48::from_bytes(&data).map_err(|e| e.to_string())
    }

    pub fn read_nil_padded<T>(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<T>, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.read(path).map_err(|e| e.to_string())?;

        alox_48::Deserializer::new(&data)
            .and_then(|mut de| rmxp_types::nil_padded::deserialize(&mut de))
            .map_err(|e| e.to_string())
    }

    pub fn save_data<T>(&self, path: impl AsRef<camino::Utf8Path>, data: &T) -> Result<(), String>
    where
        T: serde::ser::Serialize,
    {
        self.write(path, alox_48::to_bytes(data).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }

    pub fn save_nil_padded<T>(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        data: &[T],
    ) -> Result<(), String>
    where
        T: serde::ser::Serialize,
    {
        let mut ser = alox_48::Serializer::new();
        rmxp_types::nil_padded::serialize(data, &mut ser).map_err(|e| e.to_string())?;
        self.write(path, ser.output).map_err(|e| e.to_string())
    }

    pub fn project_path(&self) -> Option<camino::Utf8PathBuf> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => None,
            State::HostLoaded(h) => Some(h.root_path().to_path_buf()),
            State::Loaded { project_path, .. } => Some(project_path.clone()),
        }
    }

    pub fn project_loaded(&self) -> bool {
        !matches!(&*self.state.borrow(), State::Unloaded)
    }

    pub fn unload_project(&self) {
        *self.state.borrow_mut() = State::Unloaded;
    }

    pub fn detect_rm_ver(&self) -> Option<config::RMVer> {
        if self.exists("Data/Actors.rxdata").ok()? {
            return Some(config::RMVer::XP);
        }

        if self.exists("Data/Actors.rvdata").ok()? {
            return Some(config::RMVer::VX);
        }

        if self.exists("Data/Actors.rvdata2").ok()? {
            return Some(config::RMVer::Ace);
        }

        for path in self.read_dir("").ok()? {
            let path = path.path();
            if path.extension() == Some("rgssad") {
                return Some(config::RMVer::XP);
            }

            if path.extension() == Some("rgss2a") {
                return Some(config::RMVer::VX);
            }

            if path.extension() == Some("rgss3a") {
                return Some(config::RMVer::Ace);
            }
        }

        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn spawn_project_file_picker(&self) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "rvproj", "rvproj2", "lumproj"])
            .pick_file()
            .await
        {
            self.load_project(path.path())
        } else {
            Err("Cancelled loading project".to_string())
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn spawn_project_file_picker(&self) -> Result<(), String> {
        let tx = crate::GLOBAL_STATE.get().unwrap().filesystem_tx.clone();
        if !web::FileSystem::filesystem_supported(tx.clone()) {
            return Err("Your browser does not support File System API".to_string());
        }
        if let Some(dir) = web::FileSystem::from_directory_picker(tx).await {
            self.load_project(dir)
        } else {
            Err("Cancelled loading project".to_string())
        }
    }

    #[cfg(windows)]
    fn find_rtp_paths() -> Vec<camino::Utf8PathBuf> {
        let ini = game_ini!();
        let Some(section) = ini.section(Some("Game")) else {
            return vec![];
        };
        let mut paths = vec![];
        let mut seen_rtps = vec![];
        // FIXME: handle vx ace?
        for rtp in ["RTP1", "RTP2", "RTP3"] {
            if let Some(rtp) = section.get(rtp) {
                if seen_rtps.contains(&rtp) || rtp.is_empty() {
                    continue;
                }
                seen_rtps.push(rtp);

                let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
                if let Ok(value) = hklm
                    .open_subkey("SOFTWARE\\WOW6432Node\\Enterbrain\\RGSS\\RTP")
                    .and_then(|key| key.get_value::<String, _>(rtp))
                {
                    let path = camino::Utf8PathBuf::from(value);
                    if path.exists() {
                        paths.push(path);
                        continue;
                    }
                }

                if let Ok(value) = hklm
                    .open_subkey("SOFTWARE\\WOW6432Node\\Enterbrain\\RPGXP")
                    .and_then(|key| key.get_value::<String, _>("ApplicationPath"))
                {
                    let path = camino::Utf8PathBuf::from(value).join("rtp");
                    if path.exists() {
                        paths.push(path);
                        continue;
                    }
                }

                if let Ok(value) = hklm
                    .open_subkey(
                        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 235900",
                    )
                    .and_then(|key| key.get_value::<String, _>("InstallLocation"))
                {
                    let path = camino::Utf8PathBuf::from(value).join("rtp");
                    if path.exists() {
                        paths.push(path);
                        continue;
                    }
                }

                if let Some(path) = global_config!().rtp_paths.get(rtp) {
                    let path = camino::Utf8PathBuf::from(path);
                    if path.exists() {
                        paths.push(path);
                        continue;
                    }
                }

                state!()
                    .toasts
                    .warning(format!("Failed to find suitable path for the RTP {rtp}"));
                state!()
                    .toasts
                    .info(format!("You may want to set an RTP path for {rtp}"));
            }
        }
        paths
    }

    #[cfg(not(windows))]
    fn find_rtp_paths() -> Vec<camino::Utf8PathBuf> {
        let ini = game_ini!();
        let Some(section) = ini.section(Some("Game")) else {
            return vec![];
        };
        let mut paths = vec![];
        let mut seen_rtps = vec![];
        // FIXME: handle vx ace?
        for rtp in ["RTP1", "RTP2", "RTP3"] {
            if let Some(rtp) = section.get(rtp) {
                if seen_rtps.contains(&rtp) || rtp.is_empty() {
                    continue;
                }
                seen_rtps.push(rtp);

                if let Some(path) = global_config!().rtp_paths.get(rtp) {
                    let path = camino::Utf8PathBuf::from(path);
                    if path.exists() {
                        paths.push(path);
                        continue;
                    }
                }

                state!()
                    .toasts
                    .warning(format!("Failed to find suitable path for  the RTP {rtp}"));
                state!()
                    .toasts
                    .info(format!("You may want to set an RTP path for {rtp}"));
            }
        }
        paths
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_project(&self, project_path: impl AsRef<Path>) -> Result<(), String> {
        let original_path = camino::Utf8Path::from_path(project_path.as_ref()).unwrap();
        let path = original_path.parent().unwrap_or(original_path);

        *self.state.borrow_mut() = State::HostLoaded(host::FileSystem::new(path));

        config::Project::load()?;

        let mut list = list::FileSystem::new();

        list.push(host::FileSystem::new(path));

        for path in Self::find_rtp_paths() {
            list.push(host::FileSystem::new(path))
        }

        match archiver::FileSystem::new(path) {
            Ok(a) => list.push(a),
            Err(Error::NotExist) => (),
            Err(e) => return Err(e.to_string()),
        }

        let path_cache = path_cache::FileSystem::new(list).map_err(|e| e.to_string())?;

        *self.state.borrow_mut() = State::Loaded {
            filesystem: path_cache,
            project_path: path.to_path_buf(),
        };

        let mut projects: std::collections::VecDeque<_> = global_config!()
            .recent_projects
            .iter()
            .filter(|p| p.as_str() != original_path)
            .cloned()
            .collect();
        projects.push_front(original_path.to_string());
        global_config!().recent_projects = projects;

        if let Err(e) = state!().data_cache.load() {
            *self.state.borrow_mut() = State::Unloaded;
            return Err(e);
        }

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_project(&self, dir: web::FileSystem) -> Result<(), String> {
        let entries = dir.read_dir("").map_err(|e| e.to_string())?;
        let Some(entry) = entries.iter().find(|e| {
            if let Some(extension) = e.path.extension() {
                e.metadata.is_file
                    && (extension == "rxproj"
                        || extension == "rvproj"
                        || extension == "rvproj2"
                        || extension == "lumproj")
            } else {
                false
            }
        }) else {
            return Err("Invalid project folder".to_string());
        };

        *self.state.borrow_mut() = State::HostLoaded(dir);
        config::Project::load()?;
        let State::HostLoaded(dir) =
            std::mem::replace(&mut *self.state.borrow_mut(), State::Unloaded)
        else {
            unreachable!();
        };

        let mut list = list::FileSystem::new();

        list.push(dir);

        // TODO: handle RTPs

        // TODO: handle reading from archives

        let path_cache = path_cache::FileSystem::new(list).map_err(|e| e.to_string())?;

        *self.state.borrow_mut() = State::Loaded {
            filesystem: path_cache,
            project_path: entry.path.clone(),
        };

        if let Err(e) = state!().data_cache.load() {
            *self.state.borrow_mut() = State::Unloaded;
            return Err(e);
        }

        Ok(())
    }

    pub fn debug_ui(&self, ui: &mut egui::Ui) {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => {
                ui.label("Unloaded");
            }
            State::HostLoaded(fs) => {
                ui.label("Host Filesystem Loaded");
                ui.horizontal(|ui| {
                    ui.label("Project path: ");
                    ui.label(fs.root_path().as_str());
                });
            }
            State::Loaded { filesystem, .. } => {
                ui.label("Loaded");
                filesystem.debug_ui(ui);
            }
        }
    }
}

impl<'fs> std::io::Write for File<'fs> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.write(buf),
            FileType::Loaded(f) => f.write(buf),
        })
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.write_vectored(bufs),
            FileType::Loaded(f) => f.write_vectored(bufs),
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.flush(),
            FileType::Loaded(f) => f.flush(),
        })
    }
}

impl<'fs> std::io::Read for File<'fs> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.read(buf),
            FileType::Loaded(f) => f.read(buf),
        })
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.read_vectored(bufs),
            FileType::Loaded(f) => f.read_vectored(bufs),
        })
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.read_exact(buf),
            FileType::Loaded(f) => f.read_exact(buf),
        })
    }
}

impl<'fs> std::io::Seek for File<'fs> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.seek(pos),
            FileType::Loaded(f) => f.seek(pos),
        })
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.with_file_mut(|f| match f {
            FileType::Host(f) => f.stream_position(),
            FileType::Loaded(f) => f.stream_position(),
        })
    }
}

impl FileSystemTrait for FileSystem {
    type File<'fs> = File<'fs> where Self: 'fs;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File<'_>, Error> {
        let state = self.state.borrow();
        File::try_new(state, |state| {
            //
            match &**state {
                State::Unloaded => Err(Error::NotLoaded),
                State::HostLoaded(f) => f.open_file(path, flags).map(FileType::Host),
                State::Loaded { filesystem: f, .. } => {
                    f.open_file(path, flags).map(FileType::Loaded)
                }
            }
        })
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.metadata(path),
            State::Loaded { filesystem: f, .. } => f.metadata(path),
        }
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<(), Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.rename(from, to),
            State::Loaded { filesystem, .. } => filesystem.rename(from, to),
        }
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.exists(path),
            State::Loaded { filesystem, .. } => filesystem.exists(path),
        }
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.create_dir(path),
            State::Loaded { filesystem, .. } => filesystem.create_dir(path),
        }
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.remove_dir(path),
            State::Loaded { filesystem, .. } => filesystem.remove_dir(path),
        }
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.remove_file(path),
            State::Loaded { filesystem, .. } => filesystem.remove_file(path),
        }
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.read_dir(path),
            State::Loaded { filesystem, .. } => filesystem.read_dir(path),
        }
    }
}
