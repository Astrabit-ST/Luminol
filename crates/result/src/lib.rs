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

use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[cfg(feature = "steamworks")]
    #[error("Steam error: {0}\nPerhaps you want to compile yourself a free copy?")]
    Steamworks(#[from] steamworks::SteamError),
    #[error("Failed to install color-eyre hooks")]
    ColorEyreInstall(#[from] color_eyre::eyre::InstallError),
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),
    #[error("Temporary file error: {0}")]
    TempFilePersist(#[from] tempfile::PersistError),
    #[error("Image loader error: {0}")]
    Image(#[from] image::ImageError),
    #[cfg(target_arch = "wasm32")]
    #[error("Failed to initialise tracing-log")]
    Tracing(#[from] tracing_log::log::SetLoggerError),

    #[error("Could not get path to the current executable")]
    ExePathQueryFailed,
    #[error("Egui context cell has been already set (this shouldn't happen!)")]
    EguiContextCellAlreadySet,
}

pub type Result<T> = core::result::Result<T, Error>;
