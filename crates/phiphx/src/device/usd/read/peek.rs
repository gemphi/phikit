//! The header of a USD file, without building a scene.
//!
//! A `.usda` layer opens with a metadata block; the default prim and the
//! time range are read from it. A `.usdc` crate has a version in its first
//! bytes. Neither has a thumbnail to give.
use crate::blend::{BlendQuickInfo, BlendRenderInfo};
use crate::{PointCloudError, Result};

pub fn peek(bytes: &[u8]) -> Result<BlendQuickInfo> {
    if bytes.starts_with(b"PXR-USDC") {
        let v = bytes.get(8..11).unwrap_or(&[0, 0, 0]);
        return Ok(BlendQuickInfo { version: format!("{}.{}.{}", v[0], v[1], v[2]), ..Default::default() });
    }
    if bytes.starts_with(b"PK") {
        return Ok(BlendQuickInfo::default());
    }
    if !bytes.starts_with(b"#usda") {
        return Err(PointCloudError::InvalidFormat("not a USD layer".into()));
    }
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]).into_owned();
    let version = head.lines().next().and_then(|l| l.strip_prefix("#usda ")).unwrap_or("").trim().to_string();
    let block = head.split(')').next().unwrap_or("");
    let value = |key: &str| {
        let at = block.find(key)? + key.len();
        let rest = block[at..].trim_start().strip_prefix('=')?.trim_start();
        let end = rest.find(|c: char| c == '\n' || c == ',').unwrap_or(rest.len());
        Some(rest[..end].trim().trim_matches('"').to_string())
    };
    let frame = |key: &str| value(key).and_then(|v| v.parse::<f64>().ok()).map_or(0, |f| f as i32);
    let scene = BlendRenderInfo {
        scene: value("defaultPrim").unwrap_or_default(),
        frame_start: frame("startTimeCode"),
        frame_end: frame("endTimeCode"),
    };
    Ok(BlendQuickInfo { version, scenes: vec![scene], thumbnail: None })
}
