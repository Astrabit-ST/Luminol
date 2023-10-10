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

use poll_promise::Promise;
use slab::Slab;
use std::io::{Cursor, Read};

use super::{Audio, Source};

static_assertions::assert_impl_all!(AudioWrapper: Send, Sync);

thread_local!(static SLAB: Lazy<Mutex<Slab<Promise<()>>>> = Lazy::new(|| Mutex::new(Slab::new())));

#[derive(Debug)]
pub struct AudioWrapper {
    key: usize,
    tx: mpsc::UnboundedSender<AudioWrapperCommand>,
}

pub struct AudioWrapperCommand(AudioWrapperCommandInner);

enum AudioWrapperCommandInner {
    Play {
        vec: Vec<u8>,
        is_midi: bool,
        volume: u8,
        pitch: u8,
        source: Source,
        oneshot_tx: oneshot::Sender<Result<(), String>>,
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
}

impl AudioWrapper {
    pub fn new(audio: Audio) -> Self {
        audio.into()
    }

    pub fn play(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        volume: u8,
        pitch: u8,
        source: Source,
    ) -> Result<(), String> {
        // We have to load the file on the current thread,
        // otherwise if we read the file in the main thread of a web browser
        // we will block the main thread
        let path = path.as_ref();
        let mut file = state!()
            .filesystem
            .open_file(path, filesystem::OpenFlags::Read)
            .map_err(|e| e.to_string())?;
        let length = state!()
            .filesystem
            .metadata(path)
            .map_err(|e| e.to_string())?
            .size as usize;
        let mut vec = vec![0; length];
        file.read(&mut vec[..]).map_err(|e| e.to_string())?;

        let is_midi = path
            .extension()
            .is_some_and(|e| matches!(e, "mid" | "midi"));

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::Play {
                vec,
                is_midi,
                volume,
                pitch,
                source,
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    pub fn set_pitch(&self, pitch: u8, source: &Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::SetPitch {
                pitch,
                source: source.clone(),
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    pub fn set_volume(&self, volume: u8, source: &Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::SetVolume {
                volume,
                source: source.clone(),
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    pub fn clear_sinks(&self) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::ClearSinks {
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    pub fn stop(&self, source: &Source) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(AudioWrapperCommand(AudioWrapperCommandInner::Stop {
                source: source.clone(),
                oneshot_tx,
            }))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }
}

impl From<Audio> for AudioWrapper {
    fn from(audio: Audio) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        let promise = poll_promise::Promise::spawn_local(async move {
            loop {
                let Some(command): Option<AudioWrapperCommand> = rx.recv().await else {
                    return;
                };

                match command.0 {
                    AudioWrapperCommandInner::Play {
                        vec,
                        is_midi,
                        volume,
                        pitch,
                        source,
                        oneshot_tx,
                    } => {
                        oneshot_tx
                            .send(audio.play_from_file(
                                Cursor::new(vec),
                                is_midi,
                                volume,
                                pitch,
                                source,
                            ))
                            .unwrap();
                    }

                    AudioWrapperCommandInner::SetPitch {
                        pitch,
                        source,
                        oneshot_tx,
                    } => {
                        oneshot_tx.send(audio.set_pitch(pitch, &source)).unwrap();
                    }

                    AudioWrapperCommandInner::SetVolume {
                        volume,
                        source,
                        oneshot_tx,
                    } => {
                        oneshot_tx.send(audio.set_volume(volume, &source)).unwrap();
                    }

                    AudioWrapperCommandInner::ClearSinks { oneshot_tx } => {
                        oneshot_tx.send(audio.clear_sinks()).unwrap();
                    }

                    AudioWrapperCommandInner::Stop { source, oneshot_tx } => {
                        oneshot_tx.send(audio.stop(&source)).unwrap();
                    }
                }
            }
        });

        Self {
            key: SLAB.with(|slab| slab.lock().insert(promise)),
            tx,
        }
    }
}

impl Drop for AudioWrapper {
    fn drop(&mut self) {
        let _ = SLAB.with(|slab| slab.lock().remove(self.key));
    }
}
