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

use poll_promise::Promise;

use super::{Audio, Result, Source};

thread_local! {
    static SLAB: once_cell::sync::Lazy<std::cell::RefCell<slab::Slab<Promise<()>>>> =
        once_cell::sync::Lazy::new(|| std::cell::RefCell::new(slab::Slab::new()));
}

#[derive(Debug)]
pub struct AudioWrapper {
    key: usize,
    tx: flume::Sender<AudioWrapperCommand>,
}

pub struct AudioWrapperCommand(AudioWrapperCommandInner);

enum AudioWrapperCommandInner {
    Play {
        cursor: std::io::Cursor<Vec<u8>>,
        is_midi: bool,
        volume: u8,
        pitch: u8,
        source: Source,
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
        oneshot_tx: oneshot::Sender<()>,
    },
    ClearSinks {
        oneshot_tx: oneshot::Sender<()>,
    },
    Stop {
        source: Source,
        oneshot_tx: oneshot::Sender<()>,
    },
    Drop {
        key: usize,
        oneshot_tx: oneshot::Sender<bool>,
    },
}

impl AudioWrapper {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn play(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        filesystem: &impl luminol_filesystem::FileSystem,
        volume: u8,
        pitch: u8,
        source: Source,
    ) -> Result<()> {
        // We have to load the file on the current thread,
        // otherwise if we read the file in the main thread of a web browser
        // we will block the main thread
        let path = path.as_ref();
        let file = filesystem.read(path)?;
        let cursor = std::io::Cursor::new(file);

        let is_midi = path
            .extension()
            .is_some_and(|e| matches!(e, "mid" | "midi"));

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::Play {
                cursor,
                is_midi,
                volume,
                pitch,
                source,
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    pub fn set_pitch(&self, pitch: u8, source: &Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::SetPitch {
                pitch,
                source: *source,
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    pub fn set_volume(&self, volume: u8, source: &Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::SetVolume {
                volume,
                source: *source,
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    pub fn clear_sinks(&self) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::ClearSinks {
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    pub fn stop(&self, source: &Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::Stop {
                source: *source,
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.recv().unwrap()
    }
}

impl Default for AudioWrapper {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        if web_sys::window().is_none() {
            panic!("in web builds, `AudioWrapper` can only be created on the main thread");
        }

        let (tx, rx) = flume::unbounded::<AudioWrapperCommand>();
        let mut maybe_audio = None;

        let promise = poll_promise::Promise::spawn_local(async move {
            loop {
                let Ok(command) = rx.recv_async().await else {
                    return;
                };

                let audio = if let Some(audio) = &maybe_audio {
                    audio
                } else {
                    maybe_audio = Some(Audio::default());
                    maybe_audio.as_ref().unwrap()
                };

                match command.0 {
                    AudioWrapperCommandInner::Play {
                        cursor,
                        is_midi,
                        volume,
                        pitch,
                        source,
                        oneshot_tx,
                    } => {
                        oneshot_tx
                            .send(audio.play_from_file(cursor, is_midi, volume, pitch, source))
                            .unwrap();
                    }

                    AudioWrapperCommandInner::SetPitch {
                        pitch,
                        source,
                        oneshot_tx,
                    } => {
                        audio.set_pitch(pitch, &source);
                        oneshot_tx.send(()).unwrap();
                    }

                    AudioWrapperCommandInner::SetVolume {
                        volume,
                        source,
                        oneshot_tx,
                    } => {
                        audio.set_volume(volume, &source);
                        oneshot_tx.send(()).unwrap();
                    }

                    AudioWrapperCommandInner::ClearSinks { oneshot_tx } => {
                        audio.clear_sinks();
                        oneshot_tx.send(()).unwrap();
                    }

                    AudioWrapperCommandInner::Stop { source, oneshot_tx } => {
                        audio.stop(&source);
                        oneshot_tx.send(()).unwrap();
                    }

                    AudioWrapperCommandInner::Drop { key, oneshot_tx } => {
                        let promise = SLAB.with(|slab| slab.borrow_mut().try_remove(key));
                        oneshot_tx.send(promise.is_some()).unwrap();
                        return;
                    }
                }
            }
        });

        Self {
            key: SLAB.with(|slab| slab.borrow_mut().insert(promise)),
            tx,
        }
    }
}

impl Drop for AudioWrapper {
    fn drop(&mut self) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::Drop {
                key: self.key,
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.recv().unwrap();
    }
}
