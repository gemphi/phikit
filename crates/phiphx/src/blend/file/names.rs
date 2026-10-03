//! `ID.name` decoding (DNA-driven, with a heuristic fallback).

use crate::blend::file::types::*;
use crate::Result;

impl BlendFile {
    /// Extract a name field from a block (ID structure name[66] in Blender 4.x)
    pub(crate) fn extract_name_from_block(
        &self,
        block: &BlendBlock,
        _field_name: &str,
    ) -> Result<String> {
        // Preferred: DNA-driven `ID.name` (char[66] since 2.8x, char[24] in
        // very old files). Blender IDs always carry their two-letter code
        // prefix ("OBCube"), which we strip.
        if let Some(dna) = &self.dna {
            if let Some(off) = self.id_name_offset(dna) {
                let max = block.data.len().min(off + 66);
                if off + 3 <= max {
                    let bytes = &block.data[off..max];
                    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                    if end > 2 {
                        return Ok(String::from_utf8_lossy(&bytes[2..end]).to_string());
                    }
                }
            }
        }

        // Heuristic fallback (no DNA / unexpected layout).
        if block.data.len() < 66 {
            return Ok("Unknown".to_string());
        }

        let mut name = String::new();

        // Try first 66 bytes (Blender 4.x format)
        let name_bytes = &block.data[0..66];
        let end = name_bytes.iter().position(|&b| b == 0).unwrap_or(66);
        let extracted = String::from_utf8_lossy(&name_bytes[0..end]).to_string();

        if !extracted.is_empty() {
            name = extracted;
        } else {
            // Fallback: scan for first printable name in first 200 bytes
            let search_len = block.data.len().min(200);
            for i in 0..search_len.saturating_sub(2) {
                // Look for pattern: printable, printable, null
                let b0 = block.data[i];
                let b1 = block.data[i + 1];
                let b2 = block.data[i + 2];
                if b0 >= 32 && b0 <= 126 && b1 >= 32 && b1 <= 126 && b2 == 0 {
                    let name_end = i + 2;
                    let name_start = i.saturating_sub(62); // 64 max length
                    let mut actual_start = name_start;
                    for j in (name_start..=i).rev() {
                        if block.data[j] == 0 {
                            actual_start = j + 1;
                            break;
                        }
                    }
                    if actual_start < name_end {
                        let candidate =
                            String::from_utf8_lossy(&block.data[actual_start..name_end])
                                .to_string();
                        if candidate.len() > 1
                            && candidate
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || " _.-()[]{}".contains(c))
                        {
                            name = candidate;
                            break;
                        }
                    }
                }
            }
        }

        // Strip Blender's internal 2-character ID prefix (OB, GR, SC, ME, MA)
        if name.len() > 2
            && (name.starts_with("OB")
                || name.starts_with("GR")
                || name.starts_with("SC")
                || name.starts_with("MA")
                || name.starts_with("ME"))
        {
            Ok(name[2..].to_string())
        } else if !name.is_empty() {
            Ok(name)
        } else {
            Ok("Unknown".to_string())
        }
    }
}
