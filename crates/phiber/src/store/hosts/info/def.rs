//! Info bar host definitions matching Blender 5.2 `SpaceInfo` in `DNA_space_types.h`.

pub const INFO_RPT_DEBUG: u8 = 1 << 0;
pub const INFO_RPT_INFO: u8 = 1 << 1;
pub const INFO_RPT_OPERATOR: u8 = 1 << 2;
pub const INFO_RPT_WARNING: u8 = 1 << 3;
pub const INFO_RPT_ERROR: u8 = 1 << 4;

#[derive(Clone, Debug, PartialEq)]
pub struct InfoConfig {
    pub report_mask: u8,
}

impl Default for InfoConfig {
    fn default() -> Self {
        Self {
            report_mask: INFO_RPT_INFO | INFO_RPT_OPERATOR | INFO_RPT_WARNING | INFO_RPT_ERROR,
        }
    }
}
