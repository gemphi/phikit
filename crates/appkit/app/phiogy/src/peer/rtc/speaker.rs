//! The system's sound out: its default one, through `cpal`. What each
//! other caller sends is put on a line of its own in a mixer, and the
//! system takes from the mixer what it plays: the lines added up, at
//! the share of full `data/peer.tsv` gives.
//!
//! A line keeps only a short stretch waiting: where sound arrives
//! faster than it is played, the oldest is let go, so what is heard is
//! never far behind what was said.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use super::Set;
use super::mic::LOOK;
use super::sent::Hold;
use crate::board::lock;

/// What is waiting to be played, a line for each other caller.
#[derive(Debug, Clone, Default)]
pub struct Mixer {
    /// How many samples a second the system plays.
    pub rate: u32,
    line: Arc<Mutex<HashMap<u64, VecDeque<i16>>>>,
    next: Arc<AtomicU64>,
    /// The most samples a line keeps waiting.
    most: usize,
    /// Whether nothing of it is played.
    pub deaf: Arc<AtomicBool>,
}

/// One caller's line in the mixer. Let go, it is taken out.
pub struct Line {
    id: u64,
    mixer: Mixer,
}

impl Mixer {
    /// A line for one more caller.
    pub fn line(&self) -> Line {
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        lock(&self.line).insert(id, VecDeque::new());
        Line { id, mixer: self.clone() }
    }

    /// The next sample to play: every line's next, added up.
    fn next(line: &mut HashMap<u64, VecDeque<i16>>) -> i16 {
        let sum: i32 = line.values_mut().filter_map(VecDeque::pop_front).map(i32::from).sum();
        sum.clamp(i16::MIN as i32, i16::MAX as i32) as i16
    }
}

impl Line {
    /// Sound that arrived, one channel, at the mixer's rate.
    pub fn push(&self, data: &[i16]) {
        let mut line = lock(&self.mixer.line);
        let Some(kept) = line.get_mut(&self.id) else { return };
        kept.extend(data);
        let over = kept.len().saturating_sub(self.mixer.most);
        kept.drain(..over);
    }
}

impl Drop for Line {
    fn drop(&mut self) {
        lock(&self.mixer.line).remove(&self.id);
    }
}

/// The system's sound out, while the call is on.
pub struct Speaker(#[allow(dead_code)] Hold);

/// Give the device what the mixer has, on every channel it plays.
fn give<T>(device: &cpal::Device, config: &cpal::StreamConfig, mixer: Mixer, loud: f32) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample + FromSample<i16>,
{
    let channel = config.channels.max(1) as usize;
    let wanted = move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
        let mut line = lock(&mixer.line);
        // Deaf, what waits is still taken, and played at nothing.
        let loud = match mixer.deaf.load(Ordering::Relaxed) {
            true => 0.0,
            false => loud,
        };
        for frame in data.chunks_mut(channel) {
            let sample = T::from_sample_((Mixer::next(&mut line) as f32 * loud) as i16);
            frame.fill(sample);
        }
    };
    device.build_output_stream(config, wanted, |_| {}, None)
}

impl Speaker {
    /// Open the system's sound out. `Err` with why, where there is none
    /// or the system refuses it.
    pub fn open(set: &Set) -> Result<(Speaker, Mixer), String> {
        let (went, how) = std::sync::mpsc::channel::<Result<Mixer, String>>();
        let stop = Arc::new(AtomicBool::new(false));
        let (flag, loud, most, none) = (Arc::clone(&stop), set.count("play.loud"), set.count("play.most"), set.word("none").to_string());
        let play = move || {
            let open = || -> Result<(cpal::Stream, Mixer), String> {
                let device = cpal::default_host().default_output_device().ok_or(none)?;
                let config = device.default_output_config().map_err(|error| error.to_string())?;
                let rate = config.sample_rate().0;
                let mixer = Mixer { rate, most: (rate as f32 * most / 1000.0) as usize, ..Mixer::default() };
                let stream = match config.sample_format() {
                    SampleFormat::F32 => give::<f32>(&device, &config.config(), mixer.clone(), loud),
                    SampleFormat::I16 => give::<i16>(&device, &config.config(), mixer.clone(), loud),
                    SampleFormat::U16 => give::<u16>(&device, &config.config(), mixer.clone(), loud),
                    other => return Err(other.to_string()),
                };
                let stream = stream.map_err(|error| error.to_string())?;
                stream.play().map_err(|error| error.to_string())?;
                Ok((stream, mixer))
            };
            let opened = open();
            _ = went.send(opened.as_ref().map(|(_, mixer)| mixer.clone()).map_err(String::clone));
            while opened.is_ok() && !flag.load(Ordering::SeqCst) {
                std::thread::sleep(LOOK);
            }
        };
        let thread = std::thread::Builder::new().spawn(play).map_err(|error| error.to_string())?;
        let mixer = how.recv().map_err(|error| error.to_string())??;
        Ok((Speaker(Hold::thread(stop, thread)), mixer))
    }
}
