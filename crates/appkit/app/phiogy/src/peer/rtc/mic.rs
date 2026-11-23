//! The system's microphone: its default one, through `cpal`. It is open
//! only while this caller is in a call and not muted: muted, it is
//! closed, so nothing of the room is taken and the system shows it is
//! not in use.
//!
//! The system hands over sound on a thread of its own, in whatever
//! form the device has. Here every form becomes one channel of whole
//! numbers, which a task of the engine's cuts into the pieces the
//! library takes, measuring how loud each stretch is on the way.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use libwebrtc::audio_frame::AudioFrame;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use super::link::Kit;
use super::sent::Hold;
use super::tone::FULL;

/// How often a thread that holds a device looks whether it is to stop.
pub const LOOK: Duration = Duration::from_millis(20);

/// How many samples a second the system's microphone gives; `None`
/// where there is none. Asking does not open it.
pub fn rate() -> Option<u32> {
    let device = cpal::default_host().default_input_device()?;
    Some(device.default_input_config().ok()?.sample_rate().0)
}

/// Hand over what the device gives, as one channel of whole numbers.
fn take<T>(device: &cpal::Device, config: &cpal::StreamConfig, tell: UnboundedSender<Vec<i16>>) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample,
    i16: FromSample<T>,
{
    let channel = config.channels.max(1) as usize;
    let given = move |data: &[T], _: &cpal::InputCallbackInfo| {
        let one = |frame: &[T]| (frame.iter().map(|sample| i16::from_sample_(*sample) as i32).sum::<i32>() / channel as i32) as i16;
        _ = tell.send(data.chunks(channel).map(one).collect());
    };
    device.build_input_stream(config, given, |_| {}, None)
}

/// Open the device and keep it open until told to stop. Says first how
/// it went: why not, in the system's words, where it could not.
fn hold(wanted: u32, none: String, tell: UnboundedSender<Vec<i16>>, went: std::sync::mpsc::Sender<Result<(), String>>, stop: Arc<AtomicBool>) {
    let open = || -> Result<cpal::Stream, String> {
        let device = cpal::default_host().default_input_device().ok_or(none)?;
        let config = device.default_input_config().map_err(|error| error.to_string())?;
        // The sound is sent as it was first found: a device that gives
        // it otherwise now cannot feed it.
        if config.sample_rate().0 != wanted {
            return Err(config.sample_rate().0.to_string());
        }
        let stream = match config.sample_format() {
            SampleFormat::F32 => take::<f32>(&device, &config.config(), tell),
            SampleFormat::I16 => take::<i16>(&device, &config.config(), tell),
            SampleFormat::U16 => take::<u16>(&device, &config.config(), tell),
            other => return Err(other.to_string()),
        };
        let stream = stream.map_err(|error| error.to_string())?;
        stream.play().map_err(|error| error.to_string())?;
        Ok(stream)
    };
    let stream = open();
    _ = went.send(stream.as_ref().map(|_| ()).map_err(String::clone));
    while stream.is_ok() && !stop.load(Ordering::SeqCst) {
        std::thread::sleep(LOOK);
    }
}

/// Cut what the microphone gives into the pieces the library takes, and
/// say how loud each stretch was. It ends when the microphone closes.
async fn feed(kit: Arc<Kit>, mut given: UnboundedReceiver<Vec<i16>>) {
    let rate = kit.voice.sample_rate();
    let each = (rate as f32 * kit.set.count("piece") / 1000.0) as usize;
    let stretch = (rate as f32 * kit.set.count("level") / 1000.0) as usize;
    let (mut kept, mut top, mut count) = (Vec::new(), 0i16, 0usize);
    while let Some(more) = given.recv().await {
        kept.extend(more);
        while each > 0 && kept.len() >= each {
            let data: Vec<i16> = kept.drain(..each).collect();
            top = data.iter().map(|sample| sample.saturating_abs()).fold(top, i16::max);
            count += each;
            let frame = AudioFrame { data: data.into(), sample_rate: rate, num_channels: 1, samples_per_channel: each as u32 };
            _ = kit.voice.capture_frame(&frame).await;
        }
        if count >= stretch {
            kit.state.level(top as f32 / FULL);
            (top, count) = (0, 0);
        }
    }
    kit.state.level(0.0);
}

/// Open the system's microphone and feed the sound from it. `Err` with
/// why, where there is none or the system refuses it.
pub fn open(kit: &Arc<Kit>) -> Result<Hold, String> {
    let (tell, given) = unbounded_channel();
    let (went, how) = std::sync::mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let (wanted, none, flag) = (kit.voice.sample_rate(), kit.set.word("none").to_string(), Arc::clone(&stop));
    let thread = std::thread::Builder::new().spawn(move || hold(wanted, none, tell, went, flag)).map_err(|error| error.to_string())?;
    how.recv().map_err(|error| error.to_string())??;
    let task = kit.handle.spawn(feed(Arc::clone(kit), given));
    Ok(Hold::thread(stop, thread).with(task))
}
