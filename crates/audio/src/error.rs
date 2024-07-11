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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An error occured while decoding an audio file: {0}")]
    DecoderError(#[from] rodio::decoder::DecoderError),
    #[error("An error occured while creating a synthesizer: {0}")]
    SynthesizerError(#[from] rustysynth::SynthesizerError),
    #[error("An error occured while playing a midi track: {0}")]
    MidiError(#[from] rustysynth::MidiFileError),
    #[error("An error occured while playing a track: {0}")]
    PlayError(#[from] rodio::PlayError),
    #[error("An error occured while reading a file from the filesystem: {0}")]
    FileSystem(#[from] luminol_filesystem::Error),
}

pub use color_eyre::Result;
