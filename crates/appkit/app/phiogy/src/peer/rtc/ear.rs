//! What arrives from another caller: their sound is measured and, where
//! the system has a sound out, played; their pictures are counted and
//! shown, each frame put in the window's film under the caller's name
//! and the stream it came under.
//!
//! A camera turned off sends nothing more and says nothing: so a stream
//! that brings no picture for the time `data/peer.tsv` calls `fresh` is
//! over, and its picture is taken out of the film until one comes again.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use libwebrtc::audio_stream::native::NativeAudioStream;
use libwebrtc::audio_track::RtcAudioTrack;
use libwebrtc::native::yuv_helper;
use libwebrtc::video_frame::{I420Buffer, VideoBuffer};
use libwebrtc::video_stream::native::NativeVideoStream;
use libwebrtc::video_track::RtcVideoTrack;

use super::link::Kit;
use super::tone::FULL;
use crate::peer::engine::Made;

/// Add to what a link made.
pub fn push(made: &Mutex<Vec<Made>>, one: Made) {
    crate::board::lock(made).push(one);
}

/// Hear another caller: say how loud each stretch of their sound is,
/// and put it on a line of the mixer to be played.
pub async fn hear(track: RtcAudioTrack, kit: Arc<Kit>, made: Arc<Mutex<Vec<Made>>>) {
    let set = &kit.set;
    let rate = kit.mixer.as_ref().map_or(set.count("rate") as u32, |mixer| mixer.rate);
    let stretch = (rate as f32 * set.count("level") / 1000.0) as u32;
    let line = kit.mixer.as_ref().map(super::speaker::Mixer::line);
    let mut stream = NativeAudioStream::new(track, rate as i32, 1);
    let (mut top, mut count) = (0i16, 0u32);
    while let Some(frame) = stream.next().await {
        line.iter().for_each(|line| line.push(&frame.data));
        top = frame.data.iter().map(|sample| sample.saturating_abs()).fold(top, i16::max);
        count += frame.samples_per_channel;
        if count >= stretch {
            push(&made, Made::Level(top as f32 / FULL));
            (top, count) = (0, 0);
        }
    }
}

/// A frame as the window's film takes it: four bytes a point, blue
/// first, every point solid.
pub fn bytes(buffer: &I420Buffer) -> Vec<u8> {
    let (width, height) = (buffer.width(), buffer.height());
    let mut pixel = vec![0u8; (width * height * 4) as usize];
    let (stride_y, stride_u, stride_v) = buffer.strides();
    let (y, u, v) = buffer.data();
    yuv_helper::i420_to_argb(y, stride_y, u, stride_u, v, stride_v, &mut pixel, width * 4, width as i32, height as i32);
    pixel
}

/// What ends a picture in the film when whoever showed it stops,
/// however it stops.
struct Shown<'a>(&'a Kit, String);

impl Drop for Shown<'_> {
    fn drop(&mut self) {
        self.0.film.end(&self.1);
    }
}

/// See another caller: count the pictures that come under a stream,
/// show each in the film, and say when none comes any more.
pub async fn see(track: RtcVideoTrack, stream: String, name: String, kit: Arc<Kit>, made: Arc<Mutex<Vec<Made>>>) {
    let fresh = Duration::from_millis(kit.set.count("fresh") as u64);
    let shown = Shown(&kit, name);
    let mut arriving = NativeVideoStream::new(track);
    let (mut count, mut live) = (0u64, false);
    loop {
        match tokio::time::timeout(fresh, arriving.next()).await {
            Ok(Some(frame)) => {
                let buffer = frame.buffer.to_i420();
                count += 1;
                live = true;
                kit.film.show(&shown.1, buffer.width(), buffer.height(), bytes(&buffer));
                push(&made, Made::Seen(stream.clone(), count, buffer.width(), buffer.height()));
            }
            // The track is gone for good.
            Ok(None) => break,
            // Nothing came for a while: the camera is off, or the screen
            // no longer shared.
            Err(_) if live => {
                live = false;
                kit.film.end(&shown.1);
                push(&made, Made::Gone(stream.clone()));
            }
            Err(_) => {}
        }
    }
    push(&made, Made::Gone(stream));
}
