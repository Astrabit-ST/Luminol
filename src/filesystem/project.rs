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

use super::{
    archiver::Archiver, host::HostFS, list::List, overlay::Overlay, path_cache::PathCache,
    DirEntry, Error, FileSystem, Metadata, OpenFlags,
};

type LoadedFS = PathCache<Overlay<HostFS, List>>;

#[derive(Default)]
pub struct ProjectFS {
    state: AtomicRefCell<State>,
}

impl ProjectFS {
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
        let fs = match &*state {
            State::Unloaded => return None,
            State::HostLoaded(h) => h,
            State::Loaded(p) => p.fs().primary(),
        };
        Some(fs.root_path().to_path_buf())
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
                if seen_rtps.contains(&rtp) {
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
                    .warning(format!("Failed to find suitable path for rtp {rtp}"));
                state!()
                    .toasts
                    .info(format!("You may want to set an rtp path for {rtp}"));
            }
        }
        paths
    }

    pub fn load_project(&self, project_path: impl AsRef<Path>) -> Result<(), String> {
        let path = camino::Utf8Path::from_path(project_path.as_ref()).unwrap();
        let path = path.parent().unwrap_or(path);

        *self.state.borrow_mut() = State::HostLoaded(HostFS::new(path));

        config::Project::load()?;

        let mut list = List::new();

        for path in Self::find_rtp_paths() {
            list.push(HostFS::new(path))
        }

        match Archiver::new(project_config!().editor_ver, path) {
            Ok(a) => list.push(a),
            Err(Error::NotExist) => (),
            Err(e) => return Err(e.to_string()),
        }

        let overlay = Overlay::new(HostFS::new(path), list);
        let patch_cache = PathCache::new(overlay).map_err(|e| e.to_string())?;

        *self.state.borrow_mut() = State::Loaded(patch_cache);

        state!().data_cache.load()?;

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
            State::Loaded(fs) => {
                ui.label("Loaded");
                fs.debug_ui(ui);
            }
        }
    }
}

#[derive(Default)]
enum State {
    #[default]
    Unloaded,
    HostLoaded(HostFS),
    Loaded(LoadedFS),
}

#[ouroboros::self_referencing]
pub struct File<'fs> {
    state: AtomicRef<'fs, State>,
    #[borrows(state)]
    #[not_covariant]
    file: FileType<'this>,
}

enum FileType<'fs> {
    Host(<HostFS as FileSystem>::File<'fs>),
    Loaded(<LoadedFS as FileSystem>::File<'fs>),
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

impl FileSystem for ProjectFS {
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
                State::Loaded(f) => f.open_file(path, flags).map(FileType::Loaded),
            }
        })
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.metadata(path),
            State::Loaded(f) => f.metadata(path),
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
            State::Loaded(f) => f.rename(from, to),
        }
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.exists(path),
            State::Loaded(f) => f.exists(path),
        }
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.create_dir(path),
            State::Loaded(f) => f.create_dir(path),
        }
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.remove_dir(path),
            State::Loaded(f) => f.remove_dir(path),
        }
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.remove_file(path),
            State::Loaded(f) => f.remove_file(path),
        }
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        let state = self.state.borrow();
        match &*state {
            State::Unloaded => Err(Error::NotLoaded),
            State::HostLoaded(f) => f.read_dir(path),
            State::Loaded(f) => f.read_dir(path),
        }
    }
}
