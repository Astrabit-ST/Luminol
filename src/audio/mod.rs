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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use crate::prelude::*;

mod midi;

use strum::Display;
use strum::EnumIter;

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

/// A struct for playing Audio.
pub struct Audio {
    inner: Mutex<Inner>,
}

struct Inner {
    // OutputStream is lazily evaluated specifically for wasm. web prevents autoplay without user interaction, this is a way of dealing with that.
    // To actually play tracks the user will have needed to interact with the ui.
    _output_stream: rodio::OutputStream,
    output_stream_handle: rodio::OutputStreamHandle,
    sinks: HashMap<Source, rodio::Sink>,
}

/// # Safety
/// cpal claims that Stream (which is why Inner is not send) is not thread safe on android, which is why it is not Send anywhere else.
/// We don't support android. The only other solution would be to use thread_local and... no.
#[allow(unsafe_code)]
unsafe impl Send for Inner {}

impl Default for Audio {
    fn default() -> Self {
        let (output_stream, output_stream_handle) = rodio::OutputStream::try_default().unwrap();
        Self {
            inner: Mutex::new(Inner {
                _output_stream: output_stream,
                output_stream_handle,
                sinks: HashMap::default(),
            }),
        }
    }
}

impl Audio {
    /// Play a sound on a source.
    pub fn play(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        volume: u8,
        pitch: u8,
        source: Source,
    ) -> Result<(), String> {
        let mut inner = self.inner.lock();
        // Create a sink
        let sink = rodio::Sink::try_new(&inner.output_stream_handle).map_err(|e| e.to_string())?;

        let path = path.as_ref();
        let file = state!()
            .filesystem
            .open_file(path, filesystem::OpenFlags::Read)
            .map_err(|e| e.to_string())?;

        // Select decoder type based on sound source
        match source {
            Source::SE | Source::ME => {
                // Non looping
                if path
                    .extension()
                    .is_some_and(|e| matches!(e, "mid" | "midi"))
                {
                    sink.append(midi::MidiSource::new(file, false)?);
                } else {
                    sink.append(rodio::Decoder::new(file).map_err(|e| e.to_string())?);
                }
            }
            _ => {
                // Looping
                if path
                    .extension()
                    .is_some_and(|e| matches!(e, "mid" | "midi"))
                {
                    sink.append(midi::MidiSource::new(file, true)?);
                } else {
                    sink.append(rodio::Decoder::new_looped(file).map_err(|e| e.to_string())?);
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
