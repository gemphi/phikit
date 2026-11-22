//! The system's camera: its first one, through Windows' Media
//! Foundation. It is open only while the camera is on in a call: turned
//! off, the device is closed, so its light goes out and nothing of the
//! room is taken.
//!
//! The camera is read on a thread of its own, a picture at a time, in
//! the form the library sends best (a plane of light and a plane of
//! colour, NV12), at the size the camera itself gives. Each picture is
//! sent, and shown in the window as a mirror shows it. Nothing is kept.
//!
//! On a system that is not Windows no camera is opened yet.

/// Open the camera and send its pictures: what keeps it open, and its
/// track. `Err` with why, where there is none or the system refuses it.
#[cfg(not(target_os = "windows"))]
pub fn open(_kit: &std::sync::Arc<super::link::Kit>) -> Result<(super::sent::Hold, libwebrtc::video_track::RtcVideoTrack), String> {
    Err(String::new())
}

#[cfg(target_os = "windows")]
pub use system::open;

#[cfg(target_os = "windows")]
mod system {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use libwebrtc::native::yuv_helper;
    use libwebrtc::peer_connection_factory::native::PeerConnectionFactoryExt;
    use libwebrtc::video_frame::{NV12Buffer, VideoFrame, VideoRotation};
    use libwebrtc::video_source::VideoResolution;
    use libwebrtc::video_source::native::NativeVideoSource;
    use libwebrtc::video_track::RtcVideoTrack;
    use windows::Win32::Media::MediaFoundation::{
        IMFActivate, IMFAttributes, IMFMediaSource, IMFSourceReader, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID, MF_MT_FRAME_SIZE, MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE,
        MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, MF_SOURCE_READER_FIRST_VIDEO_STREAM, MF_VERSION, MFCreateAttributes, MFCreateMediaType, MFCreateSourceReaderFromMediaSource, MFEnumDeviceSources, MFMediaType_Video,
        MFSTARTUP_FULL, MFShutdown, MFStartup, MFVideoFormat_NV12,
    };
    use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize};

    use super::super::link::Kit;
    use super::super::sent::{Hold, Slot, film};

    /// The camera's stream of pictures, as the reader counts streams.
    const FIRST: u32 = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;

    /// The system's first camera, opened, with what reads it and how
    /// wide and high its pictures are.
    struct Open {
        source: IMFMediaSource,
        reader: IMFSourceReader,
        size: (u32, u32),
    }

    /// Open the first camera the system lists, and ask it for pictures
    /// as a plane of light and a plane of colour.
    unsafe fn first() -> windows::core::Result<Option<Open>> {
        unsafe {
            let mut wanted: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut wanted, 1)?;
            let Some(wanted) = wanted else { return Ok(None) };
            wanted.SetGUID(&MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID)?;
            let (mut list, mut count) = (std::ptr::null_mut::<Option<IMFActivate>>(), 0u32);
            MFEnumDeviceSources(&wanted, &mut list, &mut count)?;
            // Every camera listed is let go again but the first.
            let found: Vec<IMFActivate> = (0..count as usize).filter_map(|index| (*list.add(index)).take()).collect();
            CoTaskMemFree(Some(list as *const _));
            let Some(camera) = found.first() else { return Ok(None) };
            let source: IMFMediaSource = camera.ActivateObject()?;
            // The reader may change the camera's own form into the one asked for.
            let mut how: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut how, 1)?;
            how.iter().try_for_each(|how| how.SetUINT32(&MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, 1))?;
            let reader = MFCreateSourceReaderFromMediaSource(&source, how.as_ref())?;
            let form = MFCreateMediaType()?;
            form.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
            form.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)?;
            reader.SetCurrentMediaType(FIRST, None, &form)?;
            let packed = reader.GetCurrentMediaType(FIRST)?.GetUINT64(&MF_MT_FRAME_SIZE)?;
            Ok(Some(Open { source, reader, size: ((packed >> 32) as u32, packed as u32) }))
        }
    }

    /// The next picture of the camera, as its bytes; `None` when it
    /// gave none this time.
    unsafe fn next(reader: &IMFSourceReader) -> windows::core::Result<Option<Vec<u8>>> {
        unsafe {
            let mut sample = None;
            reader.ReadSample(FIRST, 0, None, None, None, Some(&mut sample))?;
            let Some(sample) = sample else { return Ok(None) };
            let buffer = sample.ConvertToContiguousBuffer()?;
            let (mut data, mut length) = (std::ptr::null_mut::<u8>(), 0u32);
            buffer.Lock(&mut data, None, Some(&mut length))?;
            let bytes = std::slice::from_raw_parts(data, length as usize).to_vec();
            buffer.Unlock()?;
            Ok(Some(bytes))
        }
    }

    /// A picture's bytes for the window: as a mirror shows it, four
    /// bytes a point.
    fn mirror(light: &[u8], colour: &[u8], (width, height): (u32, u32)) -> Vec<u8> {
        let mut pixel = vec![0u8; (width * height * 4) as usize];
        yuv_helper::nv12_to_argb(light, width, colour, width, &mut pixel, width * 4, width as i32, height as i32);
        for row in pixel.chunks_exact_mut((width * 4) as usize) {
            let mut point: Vec<[u8; 4]> = row.chunks_exact(4).map(|point| [point[0], point[1], point[2], u8::MAX]).collect();
            point.reverse();
            row.copy_from_slice(point.as_flattened());
        }
        pixel
    }

    /// The camera's thread: open it, say how that went, and read it
    /// until told to stop. The device is closed before the thread ends.
    fn read(kit: Arc<Kit>, went: std::sync::mpsc::Sender<Result<NativeVideoSource, String>>, stop: Arc<AtomicBool>, name: String) {
        // What sends pictures is made inside the engine's runtime.
        let _inside = kit.handle.enter();
        unsafe {
            _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let begun = MFStartup(MF_VERSION, MFSTARTUP_FULL);
            let opened = begun.and_then(|()| first()).map_err(|error| error.message().to_string()).and_then(|open| open.ok_or_else(|| kit.set.word("none").to_string()));
            let source = opened.as_ref().map(|open| NativeVideoSource::new(VideoResolution { width: open.size.0, height: open.size.1 }, false)).map_err(String::clone);
            _ = went.send(source.clone());
            if let (Ok(open), Ok(source)) = (opened, source) {
                let (width, height) = open.size;
                let plane = (width * height) as usize;
                while !stop.load(Ordering::SeqCst) {
                    // A picture of another size than was agreed is not sent.
                    let Ok(Some(bytes)) = next(&open.reader) else { continue };
                    if bytes.len() < plane + plane / 2 {
                        continue;
                    }
                    let (light, colour) = (&bytes[..plane], &bytes[plane..plane + plane / 2]);
                    let mut buffer = NV12Buffer::new(width, height);
                    let (stride, stride_colour) = buffer.strides();
                    let (to_light, to_colour) = buffer.data_mut();
                    light.chunks_exact(width as usize).zip(to_light.chunks_mut(stride as usize)).for_each(|(from, to)| to[..from.len()].copy_from_slice(from));
                    colour.chunks_exact(width as usize).zip(to_colour.chunks_mut(stride_colour as usize)).for_each(|(from, to)| to[..from.len()].copy_from_slice(from));
                    source.capture_frame(&VideoFrame { rotation: VideoRotation::VideoRotation0, timestamp_us: 0, buffer });
                    kit.film.show(&name, width, height, mirror(light, colour, open.size));
                }
                drop(open.reader);
                // The device itself is closed: its light goes out.
                _ = open.source.Shutdown();
            }
            _ = MFShutdown();
            CoUninitialize();
        }
    }

    pub fn open(kit: &Arc<Kit>) -> Result<(Hold, RtcVideoTrack), String> {
        let set = &kit.set;
        let name = film(set, set.word("film.own"), set.word(Slot::Picture.stream()));
        let (went, how) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let (inside, flag, shown) = (Arc::clone(kit), Arc::clone(&stop), name.clone());
        let thread = std::thread::Builder::new().spawn(move || read(inside, went, flag, shown)).map_err(|error| error.to_string())?;
        let hold = Hold::thread(stop, thread).shown(&kit.film, name);
        let source = how.recv().map_err(|error| error.to_string())??;
        Ok((hold, kit.factory.create_video_track(set.word(Slot::Picture.track()), source)))
    }
}
