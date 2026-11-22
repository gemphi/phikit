//! A screen this caller shares: one of the system's screens, taken by
//! the library's own capturer, the one a browser shares a screen with.
//! Which screen, and how many pictures of it a second, are
//! `data/peer.tsv`'s. It is taken only while it is shared.
//!
//! The capturer lives on a thread of its own, which asks it for a
//! picture at a time. Each is sent at the screen's own size, under a
//! stream of its own so the other end can tell it from a camera, and a
//! smaller copy is shown in the window. Nothing is kept.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use libwebrtc::desktop_capturer::{CaptureError, DesktopCaptureSourceType, DesktopCapturer, DesktopCapturerOptions, DesktopFrame};
use libwebrtc::native::yuv_helper;
use libwebrtc::peer_connection_factory::native::PeerConnectionFactoryExt;
use libwebrtc::video_frame::{I420Buffer, VideoBuffer, VideoFrame, VideoRotation};
use libwebrtc::video_source::VideoResolution;
use libwebrtc::video_source::native::NativeVideoSource;
use libwebrtc::video_track::RtcVideoTrack;

use super::ear::bytes;
use super::link::Kit;
use super::sent::{Hold, Slot, film};
use crate::board::lock;

/// A picture of the screen as the library sends it.
fn frame(taken: &DesktopFrame) -> Option<I420Buffer> {
    let (width, height) = (u32::try_from(taken.width()).ok()?, u32::try_from(taken.height()).ok()?);
    let mut buffer = I420Buffer::new(width, height);
    let (stride_y, stride_u, stride_v) = buffer.strides();
    let (y, u, v) = buffer.data_mut();
    yuv_helper::argb_to_i420(taken.data(), taken.stride(), y, stride_y, u, stride_u, v, stride_v, width as i32, height as i32);
    Some(buffer)
}

/// The screen's thread: find the screen, say how that went, and take a
/// picture of it at a time until told to stop.
fn take(kit: Arc<Kit>, went: std::sync::mpsc::Sender<Result<NativeVideoSource, String>>, stop: Arc<AtomicBool>, name: String) {
    let set = &kit.set;
    // What sends pictures is made inside the engine's runtime.
    let _inside = kit.handle.enter();
    // The system's own capturer asks for its services on this thread.
    #[cfg(target_os = "windows")]
    unsafe {
        _ = windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED);
    }
    let mut option = DesktopCapturerOptions::new(DesktopCaptureSourceType::Screen);
    option.set_include_cursor(true);
    let capturer = DesktopCapturer::new(option).ok_or_else(|| set.word("scene.none").to_string());
    let found = capturer.and_then(|capturer| {
        let mut list = capturer.get_source_list();
        let pick = (set.count("scene.pick") as usize).min(list.len().saturating_sub(1));
        (!list.is_empty()).then(|| (capturer, list.swap_remove(pick))).ok_or_else(|| set.word("none").to_string())
    });
    let (mut capturer, screen) = match found {
        Ok(found) => found,
        Err(why) => {
            _ = went.send(Err(why));
            return;
        }
    };
    // What sends the pictures is made once the first has said how large
    // the screen is: until then a picture only says its size.
    let sender: Arc<Mutex<Option<NativeVideoSource>>> = Arc::default();
    let (sized, size) = std::sync::mpsc::channel();
    let (shown, film, wide, held) = (name, kit.film.clone(), set.count("scene.show").max(1.0), Arc::clone(&sender));
    capturer.start_capture(Some(screen), move |taken: Result<DesktopFrame, CaptureError>| {
        let Some(mut buffer) = taken.ok().as_ref().and_then(frame) else { return };
        let Some(source) = lock(&held).clone() else {
            _ = sized.send((buffer.width(), buffer.height()));
            return;
        };
        // The copy the window shows is no wider than the table says.
        let (width, height) = (buffer.width() as f32, buffer.height() as f32);
        let share = (wide / width).min(1.0);
        let small = buffer.scale((width * share) as i32, (height * share) as i32);
        film.show(&shown, small.width(), small.height(), bytes(&small));
        source.capture_frame(&VideoFrame { rotation: VideoRotation::VideoRotation0, timestamp_us: 0, buffer });
    });
    let wait = Duration::from_millis((1000.0 / set.count("scene.frame").max(1.0)) as u64);
    let first = (0..set.count("scene.try") as u32).find_map(|_| {
        capturer.capture_frame();
        size.recv_timeout(wait).ok()
    });
    let source = first.map(|(width, height)| NativeVideoSource::new(VideoResolution { width, height }, true));
    lock(&sender).clone_from(&source);
    _ = went.send(source.ok_or_else(|| set.word("scene.none").to_string()));
    while lock(&sender).is_some() && !stop.load(Ordering::SeqCst) {
        capturer.capture_frame();
        std::thread::sleep(wait);
    }
}

/// Share a screen: what keeps it shared, and its track. `Err` where
/// the system has no screen to give.
pub fn open(kit: &Arc<Kit>) -> Result<(Hold, RtcVideoTrack), String> {
    let set = &kit.set;
    let name = film(set, set.word("film.own"), set.word(Slot::Scene.stream()));
    let (went, how) = std::sync::mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let (inside, flag, shown) = (Arc::clone(kit), Arc::clone(&stop), name.clone());
    let thread = std::thread::Builder::new().spawn(move || take(inside, went, flag, shown)).map_err(|error| error.to_string())?;
    let hold = Hold::thread(stop, thread).shown(&kit.film, name);
    let source = how.recv().map_err(|error| error.to_string())??;
    Ok((hold, kit.factory.create_video_track(set.word(Slot::Scene.track()), source)))
}
