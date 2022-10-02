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

use std::{cell::RefCell, collections::HashMap};
use strum::Display;
use strum::EnumIter;

use crate::UpdateInfo;

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
    pub fn play(&self, info: &UpdateInfo<'_>, path: &str, volume: u8, pitch: u8, source: &Source) {
        let mut inner = self.inner.borrow_mut();
        // Create a sink
        let sink = match Sink::try_new(&inner.outputstream.1) {
            Ok(s) => s,
            Err(e) => {
                info.toasts.error(e.to_string());
                return;
            }
        };
        // Append the sound
        let bufreader = match info.filesystem.bufreader(path) {
            Ok(b) => b,
            Err(e) => {
                info.toasts.error(e);
                return;
            }
        };
        // Select decoder type based on sound source
        match source {
            Source::SE | Source::ME => {
                // Non looping
                sink.append(match Decoder::new(bufreader) {
                    Ok(d) => d,
                    Err(e) => {
                        info.toasts.error(e.to_string());
                        return;
                    }
                })
            }
            _ => {
                // Looping
                sink.append(match Decoder::new_looped(bufreader) {
                    Ok(d) => d,
                    Err(e) => {
                        info.toasts.error(e.to_string());
                        return;
                    }
                })
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
