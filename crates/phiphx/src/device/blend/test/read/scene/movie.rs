use crate::device::blend::read::scene::movie::*;

#[test]
fn a_movie_is_named_after_its_range_and_container() {
    let m = BlendMovie::default();
    assert_eq!(m.file_of(std::path::Path::new("/renders/shot_.png"), 1, 250), std::path::PathBuf::from("/renders/shot_0001-0250.mkv"));
    assert_eq!(BlendMovie { container: BlendMovie::MPEG4, ..m }.file_of(std::path::Path::new("C:/r/Untitled"), 8, 12), std::path::PathBuf::from("C:/r/Untitled0008-0012.mp4"));
    assert_eq!((m.codec_name(), m.crf_value(), m.preset_name()), ("libx264", Some(23), Some(("medium", "good"))));
    assert_eq!(BlendMovie { crf: 128, custom_crf: 17, ..m }.crf_value(), Some(17));
    assert_eq!(BlendMovie { crf: -1, ..m }.crf_value(), None, "a bitrate instead");
}
