// Copyright (C) 2022 Lily Lyons
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

use rodio::Decoder;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::prelude::*;
use std::io::Cursor;

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
    _output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
    sinks: HashMap<Source, Sink>,
}

#[allow(unsafe_code)]
unsafe impl Send for Inner {}

impl Default for Audio {
    fn default() -> Self {
        let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
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
    pub fn play(&self, path: String, volume: u8, pitch: u8, source: Source) -> Result<(), String> {
        let mut inner = self.inner.lock();
        // Create a sink
        let sink = Sink::try_new(&inner.output_stream_handle).map_err(|e| e.to_string())?;

        // Append the sound
        let cursor = Cursor::new(state!().filesystem.read_bytes(path)?);
        // Select decoder type based on sound source
        match source {
            Source::SE | Source::ME => {
                // Non looping
                sink.append(Decoder::new(cursor).map_err(|e| e.to_string())?);
            }
            _ => {
                // Looping
                sink.append(Decoder::new_looped(cursor).map_err(|e| e.to_string())?);
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

    /// Stop a source.
    pub fn stop(&self, source: &Source) {
        let mut inner = self.inner.lock();
        if let Some(s) = inner.sinks.get_mut(source) {
            s.stop();
        }
    }
}
