//! The stand-ins: a tone in the microphone's place and a grey ramp in
//! the camera's and the screen's. They are what the tests and the
//! pictures in `doc/` are made with, so the other end can tell that
//! sound and pictures arrive and are alive, and nothing of a person's
//! room or screen is taken. Every number is `data/peer.tsv`'s.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use libwebrtc::audio_frame::AudioFrame;
use libwebrtc::peer_connection_factory::native::PeerConnectionFactoryExt;
use libwebrtc::video_frame::{I420Buffer, VideoFrame, VideoRotation};
use libwebrtc::video_source::VideoResolution;
use libwebrtc::video_source::native::NativeVideoSource;
use libwebrtc::video_track::RtcVideoTrack;

use super::link::Kit;
use super::sent::{Hold, Slot, film};

/// The loudest a sample is.
pub const FULL: f32 = i16::MAX as f32;

/// Feed the sound: a tone, a piece at a time, and silence while muted.
/// The source takes a piece when it has room, which is what paces this.
pub async fn tone(kit: Arc<Kit>) {
    let set = &kit.set;
    let (rate, channel) = (kit.voice.sample_rate(), set.count("channel") as u32);
    let each = (rate as f32 * set.count("piece") / 1000.0) as u32;
    let (turn, loud) = (set.count("tone") * std::f32::consts::TAU / (rate as f32).max(1.0), set.count("loud") * FULL);
    let mut at = 0u64;
    loop {
        let level = match kit.state.quiet.load(Ordering::SeqCst) {
            true => 0.0,
            false => loud,
        };
        kit.state.level(level / FULL);
        let one = |index: u32| (((at + index as u64) as f32 * turn).sin() * level) as i16;
        let data: Vec<i16> = (0..each).flat_map(|index| std::iter::repeat_n(one(index), channel as usize)).collect();
        at += each as u64;
        let frame = AudioFrame { data: data.into(), sample_rate: rate, num_channels: channel, samples_per_channel: each };
        if kit.voice.capture_frame(&frame).await.is_err() {
            tokio::time::sleep(Duration::from_millis(set.count("piece") as u64)).await;
        }
    }
}

/// A grey ramp in a camera's or a screen's place: its track, and what
/// keeps it running. Called inside the engine's runtime.
pub fn ramp(kit: &Arc<Kit>, slot: Slot) -> (Hold, RtcVideoTrack) {
    let set = &kit.set;
    let (wide, high) = match slot {
        Slot::Scene => ("scene.width", "scene.height"),
        Slot::Sound | Slot::Picture => ("width", "height"),
    };
    let (width, height) = (set.count(wide) as u32, set.count(high) as u32);
    let source = NativeVideoSource::new(VideoResolution { width, height }, slot == Slot::Scene);
    let track = kit.factory.create_video_track(set.word(slot.track()), source.clone());
    let name = film(set, set.word("film.own"), set.word(slot.stream()));
    let task = tokio::spawn(feed(source, Arc::clone(kit), (width, height), name.clone()));
    (Hold::task(task).shown(&kit.film, name), track)
}

/// Feed the picture: a ramp from dark to light that moves a step each
/// time, sent and shown in the window under its name. It is grey, or
/// in the tint `data/peer.tsv` gives it, so that one caller's stand-in
/// is told from another's.
async fn feed(source: NativeVideoSource, kit: Arc<Kit>, (width, height): (u32, u32), name: String) {
    let wait = Duration::from_millis((1000.0 / kit.set.count("frame").max(1.0)) as u64);
    let mut shift = 0u32;
    // Its blue and its red part, as a picture is sent: the middle is none.
    let (blue_part, red_part) = (kit.set.count("ramp.blue").clamp(0.0, 255.0), kit.set.count("ramp.red").clamp(0.0, 255.0));
    let middle = i8::MAX as f32 + 1.0;
    // The same tint as a screen shows it: blue, green, red, solid.
    let tint = move |light: u8| {
        let (light, blue, red) = (light as f32, blue_part - middle, red_part - middle);
        [light + 1.772 * blue, light - 0.344 * blue - 0.714 * red, light + 1.402 * red, u8::MAX as f32].map(|part| part.clamp(0.0, 255.0) as u8)
    };
    loop {
        let shade = |across: u32| ((across + shift) % width.max(1) * 255 / width.max(1)) as u8;
        let mut buffer = I420Buffer::new(width, height);
        let (stride, ..) = buffer.strides();
        let (light, blue, red) = buffer.data_mut();
        for (index, sample) in light.iter_mut().enumerate() {
            *sample = shade(index as u32 % stride.max(1));
        }
        blue.fill(blue_part as u8);
        red.fill(red_part as u8);
        source.capture_frame(&VideoFrame { rotation: VideoRotation::VideoRotation0, timestamp_us: 0, buffer });
        let row: Vec<u8> = (0..width).flat_map(|across| tint(shade(across))).collect();
        kit.film.show(&name, width, height, row.repeat(height as usize));
        shift = (shift + width / 32) % width.max(1);
        tokio::time::sleep(wait).await;
    }
}
