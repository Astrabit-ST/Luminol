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

use crate::{native::Audio as NativeAudio, Result, Source, VolumeScale};

/// A struct for playing Audio.
pub struct Audio {
    tx: flume::Sender<Command>,
}

enum Command {
    Play {
        slice: std::sync::Arc<[u8]>,
        volume: u8,
        pitch: u8,
        source: Option<Source>,
        scale: VolumeScale,
        oneshot_tx: oneshot::Sender<Result<()>>,
    },
    SetPitch {
        pitch: u8,
        source: Source,
        oneshot_tx: oneshot::Sender<()>,
    },
    SetVolume {
        volume: u8,
        source: Source,
        scale: VolumeScale,
        oneshot_tx: oneshot::Sender<()>,
    },
    ClearSinks {
        oneshot_tx: oneshot::Sender<()>,
    },
    Stop {
        source: Source,
        oneshot_tx: oneshot::Sender<()>,
    },
    Drop,
}

impl Audio {
    pub fn new() -> Self {
        Default::default()
    }

    /// Play a sound on a source.
    pub fn play(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        filesystem: &impl luminol_filesystem::FileSystem,
        volume: u8,
        pitch: u8,
        source: Option<Source>,
        scale: VolumeScale,
    ) -> Result<()> {
        // We have to load the file on the current thread,
        // otherwise if we read the file in the main thread of a web browser
        // we will block the main thread
        let path = path.as_ref();
        let slice: std::sync::Arc<[u8]> = filesystem.read(path)?.into();

        self.play_from_slice(slice, volume, pitch, source, scale)
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
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(Command::Play {
                slice: slice.as_ref().into(),
                volume,
                pitch,
                source,
                scale,
                oneshot_tx,
            })
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    /// Set the pitch of a source.
    pub fn set_pitch(&self, pitch: u8, source: Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(Command::SetPitch {
                pitch,
                source,
                oneshot_tx,
            })
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    /// Set the volume of a source.
    pub fn set_volume(&self, volume: u8, source: Source, scale: VolumeScale) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(Command::SetVolume {
                volume,
                source,
                scale,
                oneshot_tx,
            })
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    pub fn clear_sinks(&self) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx.send(Command::ClearSinks { oneshot_tx }).unwrap();
        oneshot_rx.recv().unwrap()
    }

    /// Stop a source.
    pub fn stop(&self, source: Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx.send(Command::Stop { source, oneshot_tx }).unwrap();
        oneshot_rx.recv().unwrap()
    }
}

impl Default for Audio {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        if web_sys::window().is_none() {
            panic!("in web builds, `Audio` can only be created on the main thread");
        }

        let (tx, rx) = flume::unbounded::<Command>();
        let mut maybe_audio = None;

        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let Ok(command) = rx.recv_async().await else {
                    return;
                };

                let audio = if let Some(audio) = &maybe_audio {
                    audio
                } else {
                    maybe_audio = Some(NativeAudio::default());
                    maybe_audio.as_ref().unwrap()
                };

                match command {
                    Command::Play {
                        slice,
                        volume,
                        pitch,
                        source,
                        scale,
                        oneshot_tx,
                    } => {
                        oneshot_tx
                            .send(audio.play_from_slice(slice, volume, pitch, source, scale))
                            .unwrap();
                    }

                    Command::SetPitch {
                        pitch,
                        source,
                        oneshot_tx,
                    } => {
                        audio.set_pitch(pitch, source);
                        oneshot_tx.send(()).unwrap();
                    }

                    Command::SetVolume {
                        volume,
                        source,
                        scale,
                        oneshot_tx,
                    } => {
                        audio.set_volume(volume, source, scale);
                        oneshot_tx.send(()).unwrap();
                    }

                    Command::ClearSinks { oneshot_tx } => {
                        audio.clear_sinks();
                        oneshot_tx.send(()).unwrap();
                    }

                    Command::Stop { source, oneshot_tx } => {
                        audio.stop(source);
                        oneshot_tx.send(()).unwrap();
                    }

                    Command::Drop => {
                        break;
                    }
                }
            }
        });

        Self { tx }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        self.tx.send(Command::Drop).unwrap();
    }
}
