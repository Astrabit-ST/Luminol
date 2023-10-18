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

use crate::FileSystem as _;
use crate::{archiver, host, list, path_cache};
use crate::{DirEntry, Error, Metadata, OpenFlags, Result};

#[cfg(target_arch = "wasm32")]
use super::web;

#[derive(Default)]
pub enum FileSystem {
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

pub enum File {
    #[cfg(not(target_arch = "wasm32"))]
    Host(<host::FileSystem as crate::FileSystem>::File),
    #[cfg(target_arch = "wasm32")]
    Host(<web::FileSystem as crate::FileSystem>::File),
    Loaded(<path_cache::FileSystem<list::FileSystem> as crate::FileSystem>::File),
}

impl FileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_data<T>(&self, path: impl AsRef<camino::Utf8Path>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.read(path)?;

        Ok(alox_48::from_bytes(&data)?)
    }

    pub fn read_nil_padded<T>(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.read(path)?;

        Ok(alox_48::Deserializer::new(&data)
            .and_then(|mut de| luminol_data::nil_padded::deserialize(&mut de))?)
    }

    pub fn save_data<T>(&self, path: impl AsRef<camino::Utf8Path>, data: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        self.write(path, alox_48::to_bytes(data)?)
    }

    pub fn save_nil_padded<T>(&self, path: impl AsRef<camino::Utf8Path>, data: &[T]) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        let mut ser = alox_48::Serializer::new();
        luminol_data::nil_padded::serialize(data, &mut ser)?;
        self.write(path, ser.output)
    }

    pub fn project_path(&self) -> Option<camino::Utf8PathBuf> {
        match self {
            FileSystem::Unloaded => None,
            FileSystem::HostLoaded(h) => Some(h.root_path().to_path_buf()),
            FileSystem::Loaded { project_path, .. } => Some(project_path.clone()),
        }
    }

    pub fn project_loaded(&self) -> bool {
        !matches!(self, FileSystem::Unloaded)
    }

    pub fn unload_project(&mut self) {
        *self = FileSystem::Unloaded;
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn spawn_project_file_picker(
        &mut self,
        project_config: &mut Option<luminol_config::project::Config>,
        global_config: &mut luminol_config::global::Config,
    ) -> Result<()> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "rvproj", "rvproj2", "lumproj"])
            .pick_file()
            .await
        {
            self.load_project(project_config, global_config, path.path())
        } else {
            Err(Error::CancelledLoading)
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn spawn_project_file_picker(&self) -> Result<(), String> {
        if !web::FileSystem::filesystem_supported() {
            return Err("Your browser does not support File System Access API".to_string());
        }
        if let Some(dir) = web::FileSystem::from_directory_picker().await {
            let idb_key = dir.idb_key().map(|k| k.to_string());
            if let Err(e) = self.load_project(dir) {
                if let Some(idb_key) = idb_key {
                    web::FileSystem::idb_drop(idb_key);
                }
                Err(e)
            } else {
                Ok(())
            }
        } else {
            Err("Cancelled loading project".to_string())
        }
    }

    #[cfg(windows)]
    fn find_rtp_paths(
        config: &luminol_config::project::Config,
        global_config: &luminol_config::global::Config,
    ) -> (Vec<camino::Utf8PathBuf>, Vec<String>) {
        let Some(section) = config.game_ini.section(Some("Game")) else {
            return (vec![], vec![]);
        };
        let mut paths = vec![];
        let mut seen_rtps = vec![];
        let mut missing_rtps = vec![];
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

                if let Some(path) = global_config.rtp_paths.get(rtp) {
                    let path = camino::Utf8PathBuf::from(path);
                    if path.exists() {
                        paths.push(path);
                        continue;
                    }
                }

                missing_rtps.push(rtp.to_string());
            }
        }
        (paths, missing_rtps)
    }

    #[cfg(not(any(windows, target_arch = "wasm32")))]
    fn find_rtp_paths(
        config: &luminol_config::project::Config,
        global_config: &luminol_config::global::Config,
    ) -> (Vec<camino::Utf8PathBuf>, Vec<String>) {
        let Some(section) = config.game_ini.section(Some("Game")) else {
            return (vec![], vec![]);
        };
        let mut paths = vec![];
        let mut seen_rtps = vec![];
        let mut missing_rtps = vec![];
        // FIXME: handle vx ace?
        for rtp in ["RTP1", "RTP2", "RTP3"] {
            if let Some(rtp) = section.get(rtp) {
                if seen_rtps.contains(&rtp) || rtp.is_empty() {
                    continue;
                }
                seen_rtps.push(rtp);

                if let Some(path) = global_config.rtp_paths.get(rtp) {
                    let path = camino::Utf8PathBuf::from(path);
                    if path.exists() {
                        paths.push(path);
                        continue;
                    }
                }

                missing_rtps.push(rtp.to_string());
            }
        }
        (paths, missing_rtps)
    }

    #[cfg(target_arch = "wasm32")]
    fn find_rtp_paths(dir: &web::FileSystem) -> Vec<camino::Utf8PathBuf> {
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

                let path = camino::Utf8PathBuf::from("RTP").join(rtp);
                if let Ok(exists) = dir.exists(&path) {
                    if exists {
                        paths.push(path);
                        continue;
                    }
                }

                state!()
                    .toasts
                    .warning(format!("Failed to find suitable path for the RTP {rtp}"));
                state!()
                    .toasts
                    .info(format!("Please place the {rtp} RTP in the 'RTP/{rtp}' subdirectory in your project directory"));
            }
        }
        paths
    }

    fn detect_rm_ver(&self) -> Option<luminol_config::RMVer> {
        if self.exists("Data/Actors.rxdata").ok()? {
            return Some(luminol_config::RMVer::XP);
        }

        if self.exists("Data/Actors.rvdata").ok()? {
            return Some(luminol_config::RMVer::VX);
        }

        if self.exists("Data/Actors.rvdata2").ok()? {
            return Some(luminol_config::RMVer::Ace);
        }

        for path in self.read_dir("").ok()? {
            let path = path.path();
            if path.extension() == Some("rgssad") {
                return Some(luminol_config::RMVer::XP);
            }

            if path.extension() == Some("rgss2a") {
                return Some(luminol_config::RMVer::VX);
            }

            if path.extension() == Some("rgss3a") {
                return Some(luminol_config::RMVer::Ace);
            }
        }

        None
    }

    fn load_project_config(&self) -> Result<luminol_config::project::Config> {
        if !self.exists(".luminol")? {
            self.create_dir(".luminol")?;
        }

        let project = match self
            .read_to_string(".luminol/config")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
        {
            Some(c) => c,
            None => {
                let Some(editor_ver) = self.detect_rm_ver() else {
                    return Err(Error::UnableToDetectRMVer);
                };
                let config = luminol_config::project::Project {
                    editor_ver,
                    ..Default::default()
                };
                self.write(".luminol/config", ron::to_string(&config).unwrap())?;
                config
            }
        };

        let command_db = match self
            .read_to_string(".luminol/commands")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
        {
            Some(c) => c,
            None => {
                let command_db = luminol_config::command_db::CommandDB::new(project.editor_ver);
                self.write(".luminol/commands", ron::to_string(&command_db).unwrap())?;
                command_db
            }
        };

        let game_ini = match self
            .read_to_string("Game.ini")
            .ok()
            .and_then(|i| ini::Ini::load_from_str_noescape(&i).ok())
        {
            Some(i) => i,
            None => {
                let mut ini = ini::Ini::new();
                ini.with_section(Some("Game"))
                    .set("Library", "RGSS104E.dll")
                    .set("Scripts", &project.scripts_path)
                    .set("Title", &project.project_name)
                    .set("RTP1", "")
                    .set("RTP2", "")
                    .set("RTP3", "");

                ini
            }
        };

        Ok(luminol_config::project::Config {
            project,
            command_db,
            game_ini,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_project(
        &mut self,
        project_config: &mut Option<luminol_config::project::Config>,
        global_config: &mut luminol_config::global::Config,
        project_path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        let original_path = camino::Utf8Path::from_path(project_path.as_ref()).unwrap();
        let path = original_path.parent().unwrap_or(original_path);

        *self = FileSystem::HostLoaded(host::FileSystem::new(path));

        let config = self.load_project_config()?;

        let mut list = list::FileSystem::new();

        let dir = host::FileSystem::new(path);
        let archive = dir
            .read_dir("")?
            .into_iter()
            .find(|entry| {
                entry.metadata.is_file
                    && matches!(entry.path.extension(), Some("rgssad" | "rgss2a" | "rgss3a"))
            })
            .map(|entry| dir.open_file(entry.path, OpenFlags::Read | OpenFlags::Write))
            .transpose()?
            .map(archiver::FileSystem::new)
            .transpose()?;

        list.push(dir);
        // FIXME: handle missing rtps
        let (found_rtps, missing_rtps) = Self::find_rtp_paths(&config, global_config);
        for path in found_rtps {
            list.push(host::FileSystem::new(path))
        }
        if let Some(archive) = archive {
            list.push(archive);
        }

        let path_cache = path_cache::FileSystem::new(list)?;

        *self = FileSystem::Loaded {
            filesystem: path_cache,
            project_path: path.to_path_buf(),
        };

        // FIXME: handle
        // if let Err(e) = state!().data_cache.load() {
        //     *self = FileSystem::Unloaded;
        //     return Err(e);
        // }

        let mut projects: std::collections::VecDeque<_> = global_config
            .recent_projects
            .iter()
            .filter(|p| p.as_str() != original_path)
            .cloned()
            .collect();
        projects.push_front(original_path.to_string());
        global_config.recent_projects = projects;

        *project_config = Some(config);

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

        let root_path = dir.root_path().to_path_buf();
        let idb_key = dir.idb_key().map(|k| k.to_string());

        let mut list = list::FileSystem::new();

        let paths = Self::find_rtp_paths(&dir);

        let archive = dir
            .read_dir("")
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|entry| {
                entry.metadata.is_file
                    && matches!(entry.path.extension(), Some("rgssad" | "rgss2a" | "rgss3a"))
            })
            .map(|entry| dir.open_file(entry.path, OpenFlags::Read | OpenFlags::Write))
            .transpose()
            .map_err(|e| e.to_string())?
            .map(archiver::FileSystem::new)
            .transpose()
            .map_err(|e| e.to_string())?;

        list.push(dir);
        for path in paths {
            list.push(host::FileSystem::new(path))
        }
        if let Some(archive) = archive {
            list.push(archive);
        }

        let path_cache = path_cache::FileSystem::new(list).map_err(|e| e.to_string())?;

        *self.state.borrow_mut() = State::Loaded {
            filesystem: path_cache,
            project_path: root_path.clone(),
        };

        if let Some(idb_key) = idb_key {
            let mut projects: std::collections::VecDeque<_> = global_config!()
                .recent_projects
                .iter()
                .filter(|(_, k)| k.as_str() != idb_key.as_str())
                .cloned()
                .collect();
            projects.push_front((root_path.join(&entry.path).to_string(), idb_key));
            global_config!().recent_projects = projects;
        }

        if let Err(e) = state!().data_cache.load() {
            *self.state.borrow_mut() = State::Unloaded;
            return Err(e);
        }

        Ok(())
    }

    pub fn debug_ui(&self, ui: &mut egui::Ui) {
        match self {
            FileSystem::Unloaded => {
                ui.label("Unloaded");
            }
            FileSystem::HostLoaded(fs) => {
                ui.label("Host Filesystem Loaded");
                ui.horizontal(|ui| {
                    ui.label("Project path: ");
                    ui.label(fs.root_path().as_str());
                });
            }
            FileSystem::Loaded { filesystem, .. } => {
                ui.label("Loaded");
                filesystem.debug_ui(ui);
            }
        }
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            File::Host(f) => f.write(buf),
            File::Loaded(f) => f.write(buf),
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        match self {
            File::Host(f) => f.write_vectored(bufs),
            File::Loaded(f) => f.write_vectored(bufs),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            File::Host(f) => f.flush(),
            File::Loaded(f) => f.flush(),
        }
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            File::Host(f) => f.read(buf),
            File::Loaded(f) => f.read(buf),
        }
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        match self {
            File::Host(f) => f.read_vectored(bufs),
            File::Loaded(f) => f.read_vectored(bufs),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        match self {
            File::Host(f) => f.read_exact(buf),
            File::Loaded(f) => f.read_exact(buf),
        }
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            File::Host(f) => f.seek(pos),
            File::Loaded(f) => f.seek(pos),
        }
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        match self {
            File::Host(f) => f.stream_position(),
            File::Loaded(f) => f.stream_position(),
        }
    }
}

impl crate::File for File {
    fn metadata(&self) -> Result<Metadata> {
        match self {
            File::Host(h) => crate::File::metadata(h),
            File::Loaded(l) => l.metadata(),
        }
    }
}

impl crate::FileSystem for FileSystem {
    type File = File;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.open_file(path, flags).map(File::Host),
            FileSystem::Loaded { filesystem: f, .. } => f.open_file(path, flags).map(File::Loaded),
        }
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.metadata(path),
            FileSystem::Loaded { filesystem: f, .. } => f.metadata(path),
        }
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.rename(from, to),
            FileSystem::Loaded { filesystem, .. } => filesystem.rename(from, to),
        }
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.exists(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.exists(path),
        }
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.create_dir(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.create_dir(path),
        }
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.remove_dir(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.remove_dir(path),
        }
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.remove_file(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.remove_file(path),
        }
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded),
            FileSystem::HostLoaded(f) => f.read_dir(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.read_dir(path),
        }
    }
}
