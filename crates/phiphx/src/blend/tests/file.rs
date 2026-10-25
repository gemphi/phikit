//! Reader tests that need real files: textures (#26) and DNA layout.

    use super::*;
    use std::path::Path;

    fn tests_files() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ext/REFS/blender/blender/tests/files")
    }

    /// #26: packed image bytes and `//`-relative external images reach the
    /// materials (fixture saved by Blender 5.0.1, see fixtures/README.md).
    #[test]
    fn packed_and_external_textures_resolve() {
        use crate::phx::Phx;
        let fx = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/TEXTURED-PACKED.blend");
        let m = Phx::open(&fx).expect("load");
        let tex = |name: &str| m.objects.iter().find(|o| o.name == name).and_then(|o| o.materials[0].texture.clone());
        let packed = tex("PackedPlane").expect("packed image decodes");
        assert_eq!((packed.width, packed.height), (2, 2));
        assert_eq!(packed.sample([0.25, 0.25]), [1.0, 0.0, 0.0, 1.0], "bottom-left red");
        assert_eq!(packed.sample([0.75, 0.75]), [1.0, 1.0, 0.0, 1.0], "top-right yellow");
        let ext = tex("ExternalCube").expect("//tex/magenta.png resolves");
        assert_eq!(ext.sample([0.5, 0.5]), [1.0, 0.0, 1.0, 1.0]);
        assert!(m.report.warnings.is_empty(), "{:?}", m.report.warnings);
    }

    /// Every DNA struct's summed field sizes must equal the size the file
    /// declares — the offset helpers are only right if this holds.
    #[test]
    fn dna_field_sizes_sum_to_struct_sizes() {
        let fx = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let upstream = tests_files().join("modeling/operators.blend");
        for p in [fx.join("THYMUS-BOLDROID-WAVE-003.blend"), fx.join("TEXTURED-PACKED.blend"), upstream] {
            let f = BlendFile::parse_bytes(&std::fs::read(&p).unwrap()).unwrap();
            let dna = f.dna.as_ref().unwrap();
            let ps = f.header.pointer_size as usize;
            let bad: Vec<String> = dna.structs.iter().filter_map(|st| {
                let ty = st.type_index as usize;
                let sum: usize = st.fields.iter().map(|x| {
                    BlendFile::field_size(&dna.types[x.type_index as usize], &dna.names[x.name_index as usize], dna, ps)
                }).sum();
                (sum != dna.type_sizes[ty] as usize).then(|| format!("{} {sum}≠{}", dna.types[ty], dna.type_sizes[ty]))
            }).collect();
            assert!(bad.is_empty(), "{}: {bad:?}", p.display());
        }
    }


