use kira::{
    manager::{backend::cpal::CpalBackend, AudioManager, AudioManagerSettings},
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings},
        FromFileError,
    },
    tween::Tween,
};

use std::{cell::RefCell, collections::HashMap, time::Duration};
use strum::Display;
use strum::EnumIter;

use crate::filesystem::Filesystem;

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
    manager: AudioManager,
    sounds: HashMap<Source, StreamingSoundHandle<FromFileError>>,
}

impl Default for Audio {
    fn default() -> Self {
        let manager = AudioManager::<CpalBackend>::new(AudioManagerSettings::default())
            .expect("Failed to create audio manager");
        Self {
            inner: RefCell::new(Inner {
                manager,
                sounds: HashMap::new(),
            }),
        }
    }
}

const TWEEN: Tween = Tween {
    duration: Duration::ZERO,
    start_time: kira::StartTime::Immediate,
    easing: kira::tween::Easing::Linear,
};

impl Audio {
    pub fn play(
        &self,
        filesystem: &Filesystem,
        path: &str,
        volume: u8,
        pitch: u8,
        source: &Source,
    ) {
        let mut inner = self.inner.borrow_mut();
        // Play sound
        let sound = inner
            .manager
            .play(
                StreamingSoundData::from_file(
                    filesystem.path_to(path),
                    StreamingSoundSettings::default()
                        .volume(volume as f64 / 100.)
                        .playback_rate(pitch as f64 / 100.),
                )
                .expect("Failed to load sound"),
            )
            .expect("Failed to create sound");
        // Add it to hash, stop the current one if it's playing.
        if let Some(mut s) = inner.sounds.insert(*source, sound) {
            s.stop(TWEEN).expect("Failed to stop sound");
        };
    }

    pub fn set_pitch(&self, pitch: u8, source: &Source) {
        let mut inner = self.inner.borrow_mut();
        if let Some(s) = inner.sounds.get_mut(source) {
            s.set_playback_rate(pitch as f64 / 100., TWEEN)
                .expect("Failed to change sound pitch");
        }
    }

    pub fn set_volume(&self, volume: u8, source: &Source) {
        let mut inner = self.inner.borrow_mut();
        if let Some(s) = inner.sounds.get_mut(source) {
            s.set_volume(volume as f64 / 100., TWEEN)
                .expect("Failed to change sound volume");
        }
    }

    pub fn stop(&self, source: &Source) {
        let mut inner = self.inner.borrow_mut();
        if let Some(s) = inner.sounds.get_mut(source) {
            s.stop(TWEEN).expect("Failed to stop sound");
        }
    }
}
