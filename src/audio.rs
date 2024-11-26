use std::time::Duration;

use bevy::{audio::Source, prelude::*};

// This struct usually contains the data for the audio being played.
// This is where data read from an audio file would be stored, for example.
// Implementing `TypeUuid` will automatically implement `Asset`.
// This allows the type to be registered as an asset.
#[derive(Asset, TypePath)]
pub struct SineAudio {
    pub frequency: f32,
}

#[derive(Component, Default)]
pub struct AudioEmitter {
    // stopped: bool,
}

// This decoder is responsible for playing the audio,
// and so stores data about the audio being played.
pub struct SineDecoder {
    // how far along one period the wave is (between 0 and 1)
    current_progress: f32,
    current_progress_two: f32,
    // how much we move along the period every frame
    progress_per_frame: f32,
    progress_per_frame_two: f32,
    // how long a period is
    period: f32,
    sample_rate: u32,
}

impl SineDecoder {
    fn new(frequency: f32) -> Self {
        // standard sample rate for most recordings
        let sample_rate = 44_100.0;
        SineDecoder {
            current_progress: 0.0,
            current_progress_two: 0.0,
            progress_per_frame: frequency / sample_rate,
            progress_per_frame_two: frequency * 0.8 / sample_rate,
            period: std::f32::consts::PI * 2.0,
            sample_rate: sample_rate as u32,
        }
    }
}

// The decoder must implement iterator so that it can implement `Decodable`.
impl Iterator for SineDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_progress += self.progress_per_frame;
        self.current_progress_two += self.progress_per_frame_two;
        // we loop back round to 0 to avoid floating point inaccuracies
        self.current_progress %= 1.0;
        self.current_progress_two %= 1.0;
        // TODO: cexp imaginary/real oscillator thing.
        Some(f32::clamp(
            f32::sin(self.period * self.current_progress)
                + f32::sin(self.period * self.current_progress_two),
            -1.0,
            1.0,
        ))
    }
}
// `Source` is what allows the audio source to be played by bevy.
// This trait provides information on the audio.
impl Source for SineDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// Finally `Decodable` can be implemented for our `SineAudio`.
impl Decodable for SineAudio {
    type Decoder = SineDecoder;

    type DecoderItem = <SineDecoder as Iterator>::Item;

    fn decoder(&self) -> Self::Decoder {
        SineDecoder::new(self.frequency)
    }
}
