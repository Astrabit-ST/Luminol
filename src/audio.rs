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

use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};
use strum::Display;
use strum::EnumIter;

/// Different sound sources.
#[derive(EnumIter, Display, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum Source {
    BGM,
    BGS,
    ME,
    SE,
}

pub struct Audio {
    inner: RefCell<Inner>,
}

struct Inner {
    outputstream: (OutputStream, OutputStreamHandle),
    sinks: HashMap<Source, Sink>,
}

impl Default for Audio {
    fn default() -> Self {
        let outputstream = OutputStream::try_default().unwrap();
        Self {
            inner: RefCell::new(Inner {
                outputstream,
                sinks: HashMap::new(),
            }),
        }
    }
}

impl Audio {
    pub async fn play(
        &self,
        filesystem: Rc<crate::filesystem::Filesystem>,
        path: &str,
        volume: u8,
        pitch: u8,
        source: &Source,
    ) -> Result<(), String> {
        let mut inner = self.inner.borrow_mut();
        // Create a sink
        let sink = Sink::try_new(&inner.outputstream.1).map_err(|e| e.to_string())?;
        // Append the sound
        let cursor = filesystem.bufreader(path).await?;
        // Select decoder type based on sound source
        match source {
            Source::SE | Source::ME => {
                // Non looping
                sink.append(Decoder::new(cursor).map_err(|e| e.to_string())?)
            }
            _ => {
                // Looping
                sink.append(Decoder::new_looped(cursor).map_err(|e| e.to_string())?)
            }
        }

        // Set pitch and volume
        sink.set_speed(pitch as f32 / 100.);
        sink.set_volume(volume as f32 / 100.);
        // Play sound.
        sink.play();
        // Add sink to hash, stop the current one if it's there.
        if let Some(s) = inner.sinks.insert(*source, sink) {
            s.stop();
        };

        Ok(())
    }

    pub fn set_pitch(&self, pitch: u8, source: &Source) {
        let mut inner = self.inner.borrow_mut();
        if let Some(s) = inner.sinks.get_mut(source) {
            s.set_speed(pitch as f32 / 100.);
        }
    }

    pub fn set_volume(&self, volume: u8, source: &Source) {
        let mut inner = self.inner.borrow_mut();
        if let Some(s) = inner.sinks.get_mut(source) {
            s.set_volume(volume as f32 / 100.);
        }
    }

    pub fn stop(&self, source: &Source) {
        let mut inner = self.inner.borrow_mut();
        if let Some(s) = inner.sinks.get_mut(source) {
            s.stop();
        }
    }
}
