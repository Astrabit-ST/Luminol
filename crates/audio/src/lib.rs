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

mod midi;

mod error;
pub use error::{Error, Result};

#[cfg(target_arch = "wasm32")]
mod wrapper;
#[cfg(target_arch = "wasm32")]
pub use wrapper::*;

use strum::Display;
use strum::EnumIter;

/// A struct for playing Audio.
pub struct Audio {
    inner: parking_lot::Mutex<Inner>,
}

struct Inner {
    output_stream_handle: rodio::OutputStreamHandle,
    sinks: std::collections::HashMap<Source, rodio::Sink>,
}

/// Different sound sources.
#[derive(EnumIter, Display, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(clippy::upper_case_acronyms)]
#[allow(missing_docs)]
pub enum Source {
    BGM,
    BGS,
    ME,
    SE,
}

impl Default for Audio {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        if web_sys::window().is_none() {
            panic!("in web builds, `Audio` can only be created on the main thread");
        }

        let (output_stream, output_stream_handle) = rodio::OutputStream::try_default().unwrap();
        std::mem::forget(output_stream); // Prevent the stream from being dropped
        Self {
            inner: parking_lot::Mutex::new(Inner {
                output_stream_handle,
                sinks: std::collections::HashMap::default(),
            }),
        }
    }
}

impl Audio {
    /// Play a sound on a source.
    pub fn play<T>(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        filesystem: &T,
        volume: u8,
        pitch: u8,
        source: Source,
    ) -> Result<()>
    where
        T: luminol_filesystem::FileSystem,
        T::File: 'static,
    {
        let path = path.as_ref();
        let file = filesystem.open_file(path, luminol_filesystem::OpenFlags::Read)?;

        let is_midi = path
            .extension()
            .is_some_and(|e| matches!(e, "mid" | "midi"));

        self.play_from_file(file, is_midi, volume, pitch, source)
    }

    pub fn play_from_file(
        &self,
        file: impl std::io::Read + std::io::Seek + Send + Sync + 'static,
        is_midi: bool,
        volume: u8,
        pitch: u8,
        source: Source,
    ) -> Result<()> {
        let mut inner = self.inner.lock();
        // Create a sink
        let sink = rodio::Sink::try_new(&inner.output_stream_handle)?;

        // Select decoder type based on sound source
        match source {
            Source::SE | Source::ME => {
                // Non looping
                if is_midi {
                    sink.append(midi::MidiSource::new(file, false)?);
                } else {
                    sink.append(rodio::Decoder::new(file)?);
                }
            }
            _ => {
                // Looping
                if is_midi {
                    sink.append(midi::MidiSource::new(file, true)?);
                } else {
                    sink.append(rodio::Decoder::new_looped(file)?);
                }
            }
        }

        // Set pitch and volume
        sink.set_speed(f32::from(pitch) / 100.);
        sink.set_volume(f32::from(volume) / 100.);
        // Play sound.
        sink.play();
        // Add sink to hash, stop the current one if it's there.
        if let Some(s) = inner.sinks.insert(source, sink) {
            s.stop();
            #[cfg(not(target_arch = "wasm32"))]
            s.sleep_until_end(); // wait for the sink to stop, there is a ~5ms delay where it will not
        };

        Ok(())
    }

    /// Set the pitch of a source.
    pub fn set_pitch(&self, pitch: u8, source: &Source) {
        let mut inner = self.inner.lock();
        if let Some(s) = inner.sinks.get_mut(source) {
            s.set_speed(f32::from(pitch) / 100.);
        }
    }

    /// Set the volume of a source.
    pub fn set_volume(&self, volume: u8, source: &Source) {
        let mut inner = self.inner.lock();
        if let Some(s) = inner.sinks.get_mut(source) {
            s.set_volume(f32::from(volume) / 100.);
        }
    }

    pub fn clear_sinks(&self) {
        let mut inner = self.inner.lock();
        for (_, sink) in inner.sinks.iter_mut() {
            sink.stop();
            #[cfg(not(target_arch = "wasm32"))]
            // Sleeping ensures that the inner file is dropped. There is a delay of ~5ms where it is not dropped and this could lead to a panic
            sink.sleep_until_end();
        }
        inner.sinks.clear();
    }

    /// Stop a source.
    pub fn stop(&self, source: &Source) {
        let mut inner = self.inner.lock();
        if let Some(s) = inner.sinks.get_mut(source) {
            s.stop();
        }
    }
}

impl Source {
    pub fn as_path(&self) -> &camino::Utf8Path {
        camino::Utf8Path::new(match self {
            Source::BGM => "BGM",
            Source::BGS => "BGS",
            Source::ME => "ME",
            Source::SE => "SE",
        })
    }
}
