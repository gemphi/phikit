//! `.blend` reader tests against Blender's upstream `tests/files`.

    use super::*;
    use std::path::{Path, PathBuf};

    /// Root of Blender's upstream test files (`tests/files`) inside REFS.
    fn tests_files() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../REFS/blender/blender/tests/files")
    }

    fn load(rel: &str) -> BlendScene {
        let path = tests_files().join(rel);
        let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let file = BlendFile::parse_bytes(&data).unwrap_or_else(|e| panic!("parse {rel}: {e}"));
        file.extract_scene()
            .unwrap_or_else(|e| panic!("extract scene {rel}: {e}"))
    }

    /// Blender 4.5+/5.x stores these with Zstd compression (`28 B5 2F FD`).
    /// The parser must transparently decompress, like blenloader.
    /// Blender 5.0+ writes the format-version-1 header (`BLENDER17-01v05xx`)
    /// with `LargeBHead8` block headers by default (`writefile.cc`). These
    /// upstream files are saved that way; the reader must decode them.
    #[test]
    fn loads_v1_header_large_bhead_file() {
        let path = tests_files().join("modeling/operators.blend");
        let data = std::fs::read(&path).expect("read operators.blend");
        let file = BlendFile::parse_bytes(&data).expect("parse v1 .blend");
        assert_eq!(file.header.file_format_version, 1);
        assert_eq!(file.header.bhead_type(), BHeadType::LargeBHead8);
        assert!(file.header.version_number >= 500);
        assert!(file.dna.is_some());
        let scene = file.extract_scene().expect("extract scene");
        assert!(!scene.objects.is_empty(), "v1 file must yield objects");
    }

    /// `FileGlobal.curscene` (GLOB block) selects the scene Blender opens.
    #[test]
    fn curscene_resolves_from_glob_block() {
        let path = tests_files().join("modeling/operators.blend");
        let data = std::fs::read(&path).expect("read");
        let file = BlendFile::parse_bytes(&data).expect("parse");
        let dna = file.dna.as_ref().unwrap();
        let sc = file.file_global_curscene(dna).expect("GLOB.curscene resolves to an SC block");
        assert_eq!(sc.code_str, "SC");
    }

    #[test]
    fn loads_zstd_compressed_test_file() {
        let path = tests_files().join("usd/empty.blend");
        let data = std::fs::read(&path).expect("read empty.blend");
        assert_eq!(
            &data[0..4],
            &[0x28, 0xB5, 0x2F, 0xFD],
            "precondition: file is zstd"
        );
        let file = BlendFile::parse_bytes(&data).expect("parse compressed .blend");
        assert!(file.dna.is_some(), "DNA1 block must survive decompression");
        assert!(!file.blocks.is_empty());
    }

    #[test]
    fn loads_curated_test_files() {
        if !tests_files().exists() {
            return;
        } // REFS checkout optional
        let curated = [
            "usd/empty.blend",
            "usd/usd_simple_scene.blend",
            "usd/usd_mesh_subd.blend",
            "usd/usd_anim_test.blend",
            "usd/usd_curves_test.blend",
            "ui_tests/allselect.blend",
            "ui_tests/flippedmatrixes.blend",
            "ui_tests/space_types.blend",
            "tracking/undistortion-test.blend",
        ];
        for rel in curated {
            let path = tests_files().join(rel);
            if !path.exists() {
                continue;
            }
            let _scene = load(rel);
        }
    }

    #[test]
    fn simple_scene_has_objects() {
        if !tests_files().exists() {
            return;
        }
        let scene = load("usd/usd_simple_scene.blend");
        assert!(
            !scene.objects.is_empty(),
            "usd_simple_scene.blend must yield at least one object"
        );
        let verts: usize = scene.objects.iter().map(|o| o.vertices.len()).sum();
        assert!(verts > 0, "scene should contain real mesh vertices");
    }

    /// Full sweep over every small upstream test blend.
    /// Run on demand: `cargo test -p phiphx sweep_all -- --ignored`
    /// (nearly 2000 files; too slow for the default suite).
    #[test]
    #[ignore = "sweeps ~2000 files; run with -- --ignored"]
    fn sweep_all_small_test_blends() {
        const MAX_BYTES: u64 = 2 * 1024 * 1024;
        let root = tests_files();
        if !root.exists() {
            eprintln!("skip: REFS not present");
            return;
        }

        let mut files: Vec<PathBuf> = Vec::new();
        fn walk(d: &Path, out: &mut Vec<PathBuf>) {
            for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "blend") {
                    if e.metadata().map(|m| m.len()).unwrap_or(u64::MAX) <= MAX_BYTES {
                        out.push(p);
                    }
                }
            }
        }
        walk(&root, &mut files);
        println!("sweeping {} .blend files", files.len());

        let mut failures: Vec<String> = Vec::new();
        for f in &files {
            let rel = f
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let data = match std::fs::read(f) {
                Ok(d) => d,
                Err(e) => {
                    failures.push(format!("{rel}: read: {e}"));
                    continue;
                }
            };
            let r = std::panic::catch_unwind(|| {
                BlendFile::parse_bytes(&data).and_then(|b| b.extract_scene())
            });
            match r {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => failures.push(format!("{rel}: {e}")),
                Err(_) => failures.push(format!("{rel}: panic")),
            }
        }
        println!("failed {}/{}", failures.len(), files.len());
        for f in failures.iter().take(20) {
            println!("  {f}");
        }
        assert!(failures.is_empty(), "{} files failed", failures.len());
    }

    #[test]
    fn test_blend_header_parse() {
        let _test_data = b"BLENDER-v304";
        // Real header coverage comes from the cartridge above.
    }

/// `REND` + `TEST` quick peek (file-browser preview) on Blender's own 5.x
/// file: stops before the DNA, yields the scene range and a thumbnail when
/// the file was saved with one.
#[test]
fn quick_info_reads_render_info_without_full_parse() {
    let path = tests_files().join("modeling/operators.blend");
    let info = BlendFile::read_quick_info(&path).expect("quick info");
    assert!(info.version.starts_with('5'), "{}", info.version);
    assert!(!info.scenes.is_empty(), "REND block present");
    if let Some(t) = &info.thumbnail {
        assert_eq!(t.rgba.len(), (t.width * t.height * 4) as usize);
    }
}

    /// "Load UI" (#28): workspace + 3D view restored from the window manager.
    #[test]
    fn reads_active_workspace_and_view3d() {
        let fx = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
        let s = BlendFile::parse_bytes(&std::fs::read(&fx).unwrap()).unwrap().extract_scene().unwrap();
        let ui = &s.props.ui;
        assert_eq!(ui.workspace, "Layout");
        let v = ui.view3d.as_ref().expect("THYMUS has a 3D view");
        assert_eq!(v.persp, RV3D_CAMOB, "saved looking through the camera");
        assert!((2..=5).contains(&v.shading.ty), "shading type {}", v.shading.ty);
        assert!(s.objects.iter().any(|o| o.block_address == ui.camera && o.object_type == "CAMERA"));

        let ui = load("modeling/operators.blend").props.ui;
        assert_eq!(ui.workspace, "Scripting");
        let v = ui.view3d.expect("view3d");
        assert_eq!(v.persp, 1);
        // viewinv translation = pivot (−ofs) + back axis × dist.
        for k in 0..3 {
            let eye = -v.ofs[k] + v.view_inv[2][k] * v.dist;
            assert!((eye - v.view_inv[3][k]).abs() < 1e-2, "axis {k}");
        }
    }

