//! What phiber calls each render pass inside an EXR must be what Blender
//! calls it, because those names are what a compositor looks for.
//!
//! `truth/pass_channels.txt` is the channel list out of a multi-layer EXR
//! Blender 5.0.1 actually rendered with every pass switched on
//! (`truth/blender_pass.py`). Several of the names are not guessable, which
//! is the reason this is a truth file rather than a table written by hand:
//! Mist's single channel is "Z" rather than "V", the two index passes use
//! "X", Ambient Occlusion is three channels, and UV is "U", "V", "A".

use phiphx::blend::BPassKind;
use std::collections::BTreeSet;

fn blender_channels() -> BTreeSet<String> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/pass_channels.txt");
    std::fs::read_to_string(path).unwrap().lines().filter(|l| !l.trim().is_empty()).map(str::to_string).collect()
}

#[test]
fn every_pass_we_name_is_named_the_same_way_by_blender() {
    let theirs = blender_channels();
    let mut checked = 0;
    for kind in BPassKind::list() {
        for channel in kind.channel_list() {
            let full = format!("ViewLayer.{}.{channel}", kind.name());
            assert!(theirs.contains(&full), "Blender's EXR has no channel {full}; it has {:?}", theirs.iter().filter(|c| c.contains(kind.name())).collect::<Vec<_>>());
            checked += 1;
        }
    }
    assert!(checked >= 20, "only {checked} channels were checked");
}

#[test]
fn a_pass_we_name_has_as_many_channels_as_blender_gives_it() {
    let theirs = blender_channels();
    for kind in BPassKind::list() {
        let prefix = format!("ViewLayer.{}.", kind.name());
        let count = theirs.iter().filter(|c| c.starts_with(&prefix)).count();
        assert_eq!(count, kind.channel(), "{}: Blender writes {count} channels, we claim {}", kind.name(), kind.channel());
    }
}

#[test]
fn the_pass_bits_are_distinct_and_combined_is_always_asked_for() {
    use phiphx::blend::BlendPassSet;
    // The flag has holes in it — several bits are passes Blender removed —
    // so a typo would silently alias two passes onto one bit.
    let bits: BTreeSet<u32> = BPassKind::list().iter().map(|k| k.bit()).collect();
    assert_eq!(bits.len(), BPassKind::list().len(), "two passes share a bit");

    // Blender's UI does not let Combined be switched off, so a set that does
    // not mention it still lists it rather than writing an EXR with no image.
    let none = BlendPassSet { flag: 0 };
    assert_eq!(none.list(), vec![BPassKind::Combined]);

    let some = BlendPassSet { flag: BPassKind::Depth.bit() | BPassKind::Normal.bit() };
    assert_eq!(some.list(), vec![BPassKind::Combined, BPassKind::Depth, BPassKind::Normal]);
}
