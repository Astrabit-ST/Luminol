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

use std::io::{Read, Seek};

use crate::{midi, Result, Source, VolumeScale};

/// A struct for playing Audio.
pub struct Audio {
    inner: parking_lot::Mutex<Inner>,
}

struct Inner {
    output_stream_handle: rodio::OutputStreamHandle,
    sinks: std::collections::HashMap<Source, rodio::Sink>,
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

fn apply_scale(volume: u8, scale: VolumeScale) -> f32 {
    let volume = volume.min(100);
    match scale {
        VolumeScale::Linear => volume as f32 / 100.,
        VolumeScale::Db35 => {
            if volume == 0 {
                0.
            } else {
                // -0.35 dB per percent below 100% volume
                10f32.powf(-(0.35 / 20.) * (100 - volume) as f32)
            }
        }
    }
}

impl Audio {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Play a sound on a source.
    pub fn play<T>(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        filesystem: &T,
        volume: u8,
        pitch: u8,
        source: Option<Source>,
        scale: VolumeScale,
    ) -> Result<()>
    where
        T: luminol_filesystem::FileSystem,
        T::File: 'static,
    {
        let path = path.as_ref();
        let file = filesystem.open_file(path, luminol_filesystem::OpenFlags::Read)?;

        self.play_from_file(file, volume, pitch, source, scale)
    }

    /// Play a sound on a source from audio file data.
    pub fn play_from_slice(
        &self,
        slice: impl AsRef<[u8]> + Send + Sync + 'static,
        volume: u8,
        pitch: u8,
        source: Option<Source>,
        scale: VolumeScale,
    ) -> Result<()> {
        self.play_from_file(std::io::Cursor::new(slice), volume, pitch, source, scale)
    }

    fn play_from_file(
        &self,
        mut file: impl Read + Seek + Send + Sync + 'static,
        volume: u8,
        pitch: u8,
        source: Option<Source>,
        scale: VolumeScale,
    ) -> Result<()> {
        let mut magic_header_buf = [0u8; 4];
        file.read_exact(&mut magic_header_buf)?;
        file.seek(std::io::SeekFrom::Current(-4))?;
        let is_midi = &magic_header_buf == b"MThd";

        let mut inner = self.inner.lock();
        // Create a sink
        let sink = rodio::Sink::try_new(&inner.output_stream_handle)?;

        // Select decoder type based on sound source
        match source {
            None | Some(Source::SE | Source::ME) => {
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
        sink.set_speed(pitch as f32 / 100.);
        sink.set_volume(apply_scale(volume, scale));
        // Play sound.
        sink.play();

        if let Some(source) = source {
            // Add sink to hash, stop the current one if it's there.
            if let Some(s) = inner.sinks.insert(source, sink) {
                s.stop();
                #[cfg(not(target_arch = "wasm32"))]
                s.sleep_until_end(); // wait for the sink to stop, there is a ~5ms delay where it will not
            };
        } else {
            sink.detach();
        }

        Ok(())
    }

    /// Set the pitch of a source.
    pub fn set_pitch(&self, pitch: u8, source: Source) {
        let mut inner = self.inner.lock();
        if let Some(s) = inner.sinks.get_mut(&source) {
            s.set_speed(f32::from(pitch) / 100.);
        }
    }

    /// Set the volume of a source.
    pub fn set_volume(&self, volume: u8, source: Source, scale: VolumeScale) {
        let mut inner = self.inner.lock();
        if let Some(s) = inner.sinks.get_mut(&source) {
            s.set_volume(apply_scale(volume, scale));
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
    pub fn stop(&self, source: Source) {
        let mut inner = self.inner.lock();
        if let Some(s) = inner.sinks.get_mut(&source) {
            s.stop();
        }
    }
}
