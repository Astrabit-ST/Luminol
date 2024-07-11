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
use once_cell::sync::Lazy;
use std::{io::Cursor, sync::Arc};

use crate::Result;

pub struct MidiSource {
    // These are each 4410 long
    left: Vec<f32>,
    right: Vec<f32>,
    sample_read_count: usize,
    sequencer: rustysynth::MidiFileSequencer,
}

impl MidiSource {
    pub fn new(mut file: impl std::io::Read, looping: bool) -> Result<Self> {
        let midi_file = Arc::new(rustysynth::MidiFile::new(&mut file)?);

        let settings = rustysynth::SynthesizerSettings::new(44100);
        let synthesizer = rustysynth::Synthesizer::new(&SOUND_FONT, &settings)?;
        let mut sequencer = rustysynth::MidiFileSequencer::new(synthesizer);

        sequencer.play(&midi_file, looping);

        Ok(Self::new_sequencer(sequencer))
    }

    pub fn new_sequencer(sequencer: rustysynth::MidiFileSequencer) -> Self {
        Self {
            left: vec![0.; 4410],
            right: vec![0.; 4410],
            sample_read_count: 0,
            sequencer,
        }
    }
}

pub static SOUND_FONT: Lazy<Arc<rustysynth::SoundFont>> = Lazy::new(|| {
    let soundfont = include_bytes!("GMGSx.sf2");
    let mut cursor = Cursor::new(soundfont);

    rustysynth::SoundFont::new(&mut cursor)
        .expect("failed to load sound font")
        .into()
});

impl Iterator for MidiSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sequencer.end_of_sequence() {
            return None;
        }

        let result = if self.sample_read_count % 2 == 0 {
            self.left[self.sample_read_count / 2]
        } else {
            self.right[self.sample_read_count / 2]
        };

        self.sample_read_count += 1;
        if self.sample_read_count >= 4410 * 2 {
            self.sample_read_count = 0;
            self.sequencer.render(&mut self.left, &mut self.right);
        }

        Some(result)
    }
}

impl rodio::Source for MidiSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(4410 * 2 - self.sample_read_count)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        44100
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
