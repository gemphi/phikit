//! A finished render is written in the format Properties → Output asks for.
//!
//! The thing worth guarding is not that a file appears — it is *which buffer*
//! each format reads. An EXR must carry scene-linear radiance and a PNG the
//! display image, and because a view transform is usually close to a gamma
//! curve the two look similar enough that a mix-up survives eyeballing. So
//! these tests read the files back and compare the numbers.

use phiber::v2::scene::render_job::BRenderShot;
use phiber::v2::scene::render_save::{BSaveKind, kind_of, save_to};
use phiphx::blend::{BlendImageFormat, BlendOutput, BlendPassSet};

/// A 2 × 2 render: a dark pixel, a mid one, a bright one, and one well over
/// 1.0 — the last is the one that separates a deep format from a shallow one,
/// since only a deep file can keep it.
fn a_shot() -> BRenderShot {
    let linear: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [0.25, 0.5, 0.75], [1.0, 1.0, 1.0], [4.0, 2.0, 8.0]];
    // What a Standard view transform makes of those, near enough for a test.
    let display: Vec<[f32; 3]> = linear.iter().map(|px| px.map(|v| v.clamp(0.0, 1.0).powf(1.0 / 2.2))).collect();
    let bgra: Vec<u8> = display.iter().flat_map(|px| [px[2], px[1], px[0]].map(|v| (v * 255.0).round() as u8).into_iter().chain([255u8])).collect();
    BRenderShot { w: 2, h: 2, bgra, display, linear, albedo: Vec::new(), normal: Vec::new(), depth: Vec::new(), taken: 16, target: 16, done: true }
}

fn output_of(format: BlendImageFormat, depth: u8) -> BlendOutput {
    BlendOutput { format, depth, ..Default::default() }
}

fn temp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("phiber_render_save");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn the_format_and_depth_together_choose_the_write() {
    assert_eq!(kind_of(&output_of(BlendImageFormat::Png, 8)), BSaveKind::Shallow);
    assert_eq!(kind_of(&output_of(BlendImageFormat::Png, 16)), BSaveKind::DeepPng);
    assert_eq!(kind_of(&output_of(BlendImageFormat::OpenExr, 16)), BSaveKind::Exr { half: true });
    assert_eq!(kind_of(&output_of(BlendImageFormat::OpenExr, 32)), BSaveKind::Exr { half: false });
    assert_eq!(kind_of(&output_of(BlendImageFormat::RadianceHdr, 32)), BSaveKind::Hdr);
    // A depth the format does not offer falls back to its first, the way
    // Blender's own `BKE_image_format_media_type_set` clamps it — an 8-bit
    // EXR asked for by an old file is written half, not refused.
    assert_eq!(kind_of(&output_of(BlendImageFormat::OpenExr, 8)), BSaveKind::Exr { half: true });
    assert_eq!(kind_of(&output_of(BlendImageFormat::Jpeg, 16)), BSaveKind::Shallow);
}

#[test]
fn an_exr_keeps_the_scene_linear_values_including_those_over_one() {
    let shot = a_shot();
    for (name, depth) in [("float.exr", 32u8), ("half.exr", 16)] {
        let file = temp(name);
        save_to(&shot, &output_of(BlendImageFormat::OpenExr, depth), BlendPassSet::default(), &file).unwrap();
        let read = exr::prelude::read_first_rgba_layer_from_file(
            &file,
            |size, _| vec![[0.0f32; 4]; size.width() * size.height()],
            |buf: &mut Vec<[f32; 4]>, pos, px: (f32, f32, f32, f32)| {
                buf[pos.y() * 2 + pos.x()] = [px.0, px.1, px.2, px.3];
            },
        )
        .unwrap();
        let px = &read.layer_data.channel_data.pixels;
        // Half has about three decimal digits, so the tolerance follows the
        // depth rather than being one loose figure for both.
        let slack = if depth == 16 { 0.01 } else { 1e-6 };
        for (got, want) in px.iter().zip(shot.linear.iter()) {
            for channel in 0..3 {
                let diff = (got[channel] - want[channel]).abs();
                assert!(diff <= slack * want[channel].max(1.0), "{name}: {got:?} is not {want:?}");
            }
        }
        // The over-bright pixel is the point of the format: a shallow write
        // would have clipped it to 1.0.
        assert!(px[3][2] > 7.0, "{name}: the bright pixel was clipped to {}", px[3][2]);
    }
    // Half really is half: the same image is appreciably smaller.
    let (half, float) = (std::fs::metadata(temp("half.exr")).unwrap().len(), std::fs::metadata(temp("float.exr")).unwrap().len());
    assert!(half < float, "a half EXR ({half} bytes) should be smaller than a float one ({float})");
}

#[test]
fn a_sixteen_bit_png_is_the_display_image_at_more_precision() {
    let shot = a_shot();
    let file = temp("deep.png");
    save_to(&shot, &output_of(BlendImageFormat::Png, 16), BlendPassSet::default(), &file).unwrap();
    let read = image::open(&file).unwrap();
    let deep = read.as_rgba16().expect("a 16-bit PNG should read back as 16 bits");
    for (index, px) in deep.pixels().enumerate() {
        let want = shot.display[index];
        for channel in 0..3 {
            let got = f32::from(px.0[channel]) / 65535.0;
            assert!((got - want[channel]).abs() < 1e-4, "pixel {index} channel {channel}: {got} is not {}", want[channel]);
        }
    }
    // Display, not linear: the mid pixel would read 0.25 if the writer had
    // taken the scene-linear buffer by mistake.
    let mid = f32::from(deep.get_pixel(1, 0).0[0]) / 65535.0;
    assert!(mid > 0.4, "the 16-bit PNG looks scene-linear ({mid}), not display-referred");
    // Opaque alpha survives the widening rather than becoming 65280.
    assert_eq!(deep.get_pixel(0, 0).0[3], 65535);
}

#[test]
fn an_eight_bit_png_is_the_image_already_on_screen() {
    let shot = a_shot();
    let file = temp("shallow.png");
    save_to(&shot, &output_of(BlendImageFormat::Png, 8), BlendPassSet::default(), &file).unwrap();
    let read = image::open(&file).unwrap().to_rgba8();
    // The job stores BGRA; the file must not come out with red and blue
    // swapped. Read that off the blue-leaning mid pixel — the bright one is
    // white in display space and would pass either way.
    let mid = read.get_pixel(1, 0).0;
    assert!(mid[2] > mid[0], "red and blue look swapped: {mid:?}");
    for (index, px) in read.pixels().enumerate() {
        let bgra = &shot.bgra[index * 4..index * 4 + 4];
        assert_eq!([px.0[0], px.0[1], px.0[2]], [bgra[2], bgra[1], bgra[0]], "pixel {index}");
    }
}

#[test]
fn a_render_that_has_not_settled_is_not_written() {
    let waiting = BRenderShot { w: 2, h: 2, taken: 3, target: 16, ..Default::default() };
    let file = temp("never.png");
    let _ = std::fs::remove_file(&file);
    assert!(save_to(&waiting, &output_of(BlendImageFormat::Png, 8), BlendPassSet::default(), &file).is_err());
    assert!(!file.exists(), "a partial render should not leave a black file behind");
}

#[test]
fn the_output_path_follows_blenders_own_rules() {
    let folder = std::path::Path::new("/home/someone/shots");
    let at = |path: &str, format| BlendOutput { path: path.to_string(), format, ..Default::default() }.file_of(folder);
    // `//` is relative to the .blend's own folder.
    assert_eq!(at("//render", BlendImageFormat::Png), folder.join("render.png"));
    // A bare folder gets Blender's "Untitled", as `do_makepicstring` does.
    assert_eq!(at("//out/", BlendImageFormat::OpenExr), folder.join("out/Untitled.exr"));
    // The extension follows the format, and is not doubled when already right.
    assert_eq!(at("//render.exr", BlendImageFormat::OpenExr), folder.join("render.exr"));
    // An extension that is already an image format is replaced, not stacked
    // (`do_ensure_image_extension`).
    assert_eq!(at("//render.png", BlendImageFormat::OpenExr), folder.join("render.exr"));
    // One that is not stays put and gains the extension.
    assert_eq!(at("//render.v2", BlendImageFormat::Png), folder.join("render.v2.png"));
    // An absolute path is left where it is.
    assert_eq!(at("/tmp/frame", BlendImageFormat::RadianceHdr), std::path::PathBuf::from("/tmp/frame.hdr"));
}
