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

use color_eyre::eyre::WrapErr;
#[cfg(target_arch = "wasm32")]
use itertools::Itertools;

use crate::FileSystem as _;
use crate::{archiver, host, list, path_cache};
use crate::{DirEntry, Error, Metadata, OpenFlags, Result};

#[derive(Default)]
pub enum FileSystem {
    #[default]
    Unloaded,
    HostLoaded(host::FileSystem),
    Loaded {
        filesystem: path_cache::FileSystem<list::FileSystem>,
        host_filesystem: host::FileSystem,
        project_path: camino::Utf8PathBuf,
    },
}

pub enum File {
    Host(<host::FileSystem as crate::FileSystem>::File),
    Loaded(<path_cache::FileSystem<list::FileSystem> as crate::FileSystem>::File),
}

#[must_use = "contains potential warnings generated while loading a project"]
pub struct LoadResult {
    pub missing_rtps: Vec<String>,
}

impl FileSystem {
    pub fn new() -> Self {
        Self::default()
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

    pub fn rebuild_path_cache(&mut self) {
        let FileSystem::Loaded { filesystem, .. } = self else {
            return;
        };
        filesystem.rebuild();
    }
}

// Not platform specific
impl FileSystem {
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
        let c = "While loading project configuration";
        self.create_dir(".luminol").wrap_err(c)?;

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
                    .set("Scripts", "Data/Scripts.rxdata")
                    .set("Title", "")
                    .set("RTP1", "")
                    .set("RTP2", "")
                    .set("RTP3", "");

                let mut file = self.open_file(
                    "Game.ini",
                    OpenFlags::Write | OpenFlags::Create | OpenFlags::Truncate,
                )?;
                ini.write_to(&mut file)?;

                ini
            }
        };

        let pretty_config = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .enumerate_arrays(true);

        let project = match self
            .read_to_string(".luminol/config")
            .ok()
            .and_then(|s| ron::from_str::<luminol_config::project::Project>(&s).ok())
        {
            Some(config) if config.persistence_id != 0 => config,
            Some(mut config) => {
                while config.persistence_id == 0 {
                    config.persistence_id = rand::random();
                }
                self.write(
                    ".luminol/config",
                    ron::ser::to_string_pretty(&config, pretty_config.clone()).wrap_err(c)?,
                )
                .wrap_err(c)?;
                config
            }
            None => {
                let Some(editor_ver) = self.detect_rm_ver() else {
                    return Err(Error::UnableToDetectRMVer).wrap_err(c);
                };
                let project_name = game_ini
                    .general_section()
                    .get("Title")
                    .unwrap_or("Untitled Project")
                    .to_string();
                let config = luminol_config::project::Project {
                    editor_ver,
                    project_name,
                    ..Default::default()
                };
                self.write(
                    ".luminol/config",
                    ron::ser::to_string_pretty(&config, pretty_config.clone()).wrap_err(c)?,
                )
                .wrap_err(c)?;
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
                self.write(
                    ".luminol/commands",
                    ron::ser::to_string_pretty(&command_db, pretty_config.clone()).wrap_err(c)?,
                )
                .wrap_err(c)?;
                command_db
            }
        };

        Ok(luminol_config::project::Config {
            project,
            command_db,
            game_ini,
        })
    }

    pub fn debug_ui(&self, ui: &mut egui::Ui) {
        ui.set_width(ui.available_width());

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

    pub fn load_project(
        &mut self,
        host: host::FileSystem,
        project_config: &mut Option<luminol_config::project::Config>,
        global_config: &mut luminol_config::global::Config,
    ) -> Result<LoadResult> {
        let c = "While loading project data";

        *self = FileSystem::HostLoaded(host);
        let config = self.load_project_config().wrap_err(c)?;

        let Self::HostLoaded(host) = std::mem::take(self) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Unable to fetch host filesystem",
            )
            .into());
        };

        let result = self
            .load_partially_loaded_project(host, &config, global_config)
            .wrap_err(c)?;

        *project_config = Some(config);

        Ok(result)
    }

    pub fn host(&self) -> Option<host::FileSystem> {
        match self {
            FileSystem::Unloaded => None,
            FileSystem::HostLoaded(host) => Some(host.clone()),
            FileSystem::Loaded {
                host_filesystem, ..
            } => Some(host_filesystem.clone()),
        }
    }
}

// Specific to windows
#[cfg(windows)]
impl FileSystem {
    fn find_rtp_paths(
        filesystem: &host::FileSystem,
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

                let path = filesystem.root_path().join("RTP").join(rtp);
                if let Ok(exists) = filesystem.exists(&path) {
                    if exists {
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
}

// Specific to anything BUT windows
#[cfg(not(any(windows, target_arch = "wasm32")))]
impl FileSystem {
    fn find_rtp_paths(
        filesystem: &host::FileSystem,
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

                let path = filesystem.root_path().join("RTP").join(rtp);
                if let Ok(exists) = filesystem.exists(&path) {
                    if exists {
                        paths.push(path);
                        continue;
                    }
                }

                missing_rtps.push(rtp.to_string());
            }
        }
        (paths, missing_rtps)
    }
}

// Specific to native
#[cfg(not(target_arch = "wasm32"))]
impl FileSystem {
    pub fn load_project_from_path(
        &mut self,
        project_config: &mut Option<luminol_config::project::Config>,
        global_config: &mut luminol_config::global::Config,
        project_path: impl AsRef<camino::Utf8Path>,
    ) -> Result<LoadResult> {
        let host = host::FileSystem::new(project_path);
        self.load_project(host, project_config, global_config)
    }

    pub fn load_partially_loaded_project(
        &mut self,
        host: host::FileSystem,
        project_config: &luminol_config::project::Config,
        global_config: &mut luminol_config::global::Config,
    ) -> Result<LoadResult> {
        let host_clone = host.clone();
        let project_path = host.root_path().to_path_buf();

        let mut list = list::FileSystem::new();

        let archive = host
            .read_dir("")?
            .into_iter()
            .find(|entry| {
                entry.metadata.is_file
                    && matches!(entry.path.extension(), Some("rgssad" | "rgss2a" | "rgss3a"))
            })
            .map(|entry| host.open_file(entry.path, OpenFlags::Read | OpenFlags::Write))
            .transpose()?
            .map(archiver::FileSystem::new)
            .transpose()?;

        // FIXME: handle missing rtps
        let (found_rtps, missing_rtps) = Self::find_rtp_paths(&host, project_config, global_config);

        list.push(host);

        for path in found_rtps {
            list.push(host::FileSystem::new(path))
        }
        if let Some(archive) = archive {
            list.push(archive);
        }

        let path_cache = path_cache::FileSystem::new(list)?;

        *self = FileSystem::Loaded {
            filesystem: path_cache,
            host_filesystem: host_clone,
            project_path: project_path.to_path_buf(),
        };

        // FIXME: handle
        // if let Err(e) = state!().data_cache.load() {
        //     *self = FileSystem::Unloaded;
        //     return Err(e);
        // }

        let mut projects: std::collections::VecDeque<_> = global_config
            .recent_projects
            .iter()
            .filter(|p| p.as_str() != project_path)
            .cloned()
            .collect();
        projects.push_front(project_path.into_string());
        global_config.recent_projects = projects;

        Ok(LoadResult { missing_rtps })
    }
}

// Specific to web
#[cfg(target_arch = "wasm32")]
impl FileSystem {
    fn find_rtp_paths(
        filesystem: &host::FileSystem,
        config: &luminol_config::project::Config,
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

                let path = camino::Utf8PathBuf::from("RTP").join(rtp);
                if let Ok(exists) = filesystem.exists(&path) {
                    if exists {
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
    pub fn load_partially_loaded_project(
        &mut self,
        host: host::FileSystem,
        project_config: &luminol_config::project::Config,
        global_config: &mut luminol_config::global::Config,
    ) -> Result<LoadResult> {
        let entries = host.read_dir("")?;
        if !entries.iter().any(|e| {
            if let Some(extension) = e.path.extension() {
                e.metadata.is_file
                    && (extension == "rxproj"
                        || extension == "rvproj"
                        || extension == "rvproj2"
                        || extension == "lumproj")
            } else {
                false
            }
        }) {
            return Err(Error::InvalidProjectFolder.into());
        };

        let root_path = host.root_path().to_path_buf();

        let mut list = list::FileSystem::new();

        let (found_rtps, missing_rtps) = Self::find_rtp_paths(&host, project_config);
        let rtp_filesystems: Vec<_> = found_rtps
            .into_iter()
            .map(|rtp| host.subdir(rtp))
            .try_collect()?;

        let archive = host
            .read_dir("")?
            .into_iter()
            .find(|entry| {
                entry.metadata.is_file
                    && matches!(entry.path.extension(), Some("rgssad" | "rgss2a" | "rgss3a"))
            })
            .map(|entry| host.open_file(entry.path, OpenFlags::Read | OpenFlags::Write))
            .transpose()?
            .map(archiver::FileSystem::new)
            .transpose()?;

        list.push(host.clone());
        for filesystem in rtp_filesystems {
            list.push(filesystem)
        }
        if let Some(archive) = archive {
            list.push(archive);
        }

        let path_cache = path_cache::FileSystem::new(list)?;

        *self = Self::Loaded {
            filesystem: path_cache,
            host_filesystem: host.clone(),
            project_path: root_path.clone(),
        };

        if let Ok(idb_key) = host.save_to_idb() {
            let mut projects: std::collections::VecDeque<_> = global_config
                .recent_projects
                .iter()
                .filter(|(_, k)| k.as_str() != idb_key)
                .cloned()
                .collect();
            projects.push_front((root_path.to_string(), idb_key.to_string()));
            global_config.recent_projects = projects;
        }

        Ok(LoadResult { missing_rtps })
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
    fn metadata(&self) -> std::io::Result<Metadata> {
        match self {
            File::Host(h) => crate::File::metadata(h),
            File::Loaded(l) => l.metadata(),
        }
    }

    fn set_len(&self, new_size: u64) -> std::io::Result<()> {
        match self {
            File::Host(f) => f.set_len(new_size),
            File::Loaded(f) => f.set_len(new_size),
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
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
            FileSystem::HostLoaded(f) => f.open_file(path, flags).map(File::Host),
            FileSystem::Loaded { filesystem: f, .. } => f.open_file(path, flags).map(File::Loaded),
        }
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
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
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
            FileSystem::HostLoaded(f) => f.rename(from, to),
            FileSystem::Loaded { filesystem, .. } => filesystem.rename(from, to),
        }
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
            FileSystem::HostLoaded(f) => f.exists(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.exists(path),
        }
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
            FileSystem::HostLoaded(f) => f.create_dir(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.create_dir(path),
        }
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
            FileSystem::HostLoaded(f) => f.remove_dir(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.remove_dir(path),
        }
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
            FileSystem::HostLoaded(f) => f.remove_file(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.remove_file(path),
        }
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        match self {
            FileSystem::Unloaded => Err(Error::NotLoaded.into()),
            FileSystem::HostLoaded(f) => f.read_dir(path),
            FileSystem::Loaded { filesystem, .. } => filesystem.read_dir(path),
        }
    }
}
