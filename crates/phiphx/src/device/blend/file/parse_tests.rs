//! Header / BHead decoding tests (split from `parse.rs`).

use super::super::types::*;
use super::super::types::BlendFile;

    #[test]
    fn legacy_header_decodes() {
        let mut c = std::io::Cursor::new(b"BLENDER-v305".to_vec());
        let h = BlendFile::read_header(&mut c).unwrap();
        assert_eq!(h.pointer_size, 8);
        assert!(h.little_endian);
        assert_eq!(h.version_number, 305);
        assert_eq!(h.file_format_version, 0);
        assert!(matches!(h.bhead_type(), BHeadType::SmallBHead8));
    }

    #[test]
    fn legacy_32bit_header_decodes() {
        let mut c = std::io::Cursor::new(b"BLENDER_V249".to_vec());
        let h = BlendFile::read_header(&mut c).unwrap();
        assert_eq!(h.pointer_size, 4);
        assert!(!h.little_endian);
        assert!(matches!(h.bhead_type(), BHeadType::BHead4));
    }

    #[test]
    fn v1_header_decodes() {
        let mut c = std::io::Cursor::new(b"BLENDER17-01v0502".to_vec());
        let h = BlendFile::read_header(&mut c).unwrap();
        assert_eq!(h.pointer_size, 8);
        assert!(h.little_endian);
        assert_eq!(h.version_number, 502);
        assert_eq!(h.file_format_version, 1);
        assert!(matches!(h.bhead_type(), BHeadType::LargeBHead8));
    }

    #[test]
    fn unknown_header_is_rejected() {
        let mut c = std::io::Cursor::new(b"BLENDER99-01v0502".to_vec());
        assert!(BlendFile::read_header(&mut c).is_err());
    }

    #[test]
    fn large_bhead8_block_roundtrip() {
        // Header + one LargeBHead8 'DATA' block of 8 bytes + ENDB.
        let mut bytes = b"BLENDER17-01v0502".to_vec();
        bytes.extend_from_slice(b"DATA");
        bytes.extend_from_slice(&7u32.to_le_bytes()); // SDNAnr
        bytes.extend_from_slice(&0xDEADBEEFu64.to_le_bytes()); // old
        bytes.extend_from_slice(&8i64.to_le_bytes()); // len
        bytes.extend_from_slice(&1i64.to_le_bytes()); // nr
        bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        bytes.extend_from_slice(b"ENDB");
        bytes.extend_from_slice(&[0u8; 28]);
        let f = BlendFile::parse_bytes(&bytes).unwrap();
        assert_eq!(f.blocks.len(), 1);
        let b = f.block_at(0xDEADBEEF).unwrap();
        assert_eq!(b.code_str, "DATA");
        assert_eq!(b.sdna_index, 7);
        assert_eq!(b.data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }
