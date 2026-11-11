//! The devices, in a fixed order: USD, Blend, PHX. A device whose feature is
//! off is not in the list. PHX joins in phase 3.
use super::{Device, DeviceKind};

/// Every device this build has, in the order they are asked.
pub struct DeviceList {
    device: Vec<&'static dyn Device>,
}

impl DeviceList {
    /// The devices of this build.
    pub fn new() -> Self {
        Self { device: vec![&super::usd::UsdDevice, &super::blend::BlendDevice] }
    }

    pub fn iter(&self) -> impl Iterator<Item = &'static dyn Device> + '_ {
        self.device.iter().copied()
    }

    pub fn get(&self, kind: DeviceKind) -> Option<&'static dyn Device> {
        self.iter().find(|d| d.kind() == kind)
    }

    /// The first device whose `detect` takes the head of a file.
    pub fn detect(&self, head: &[u8]) -> Option<&'static dyn Device> {
        self.iter().find(|d| d.detect(head))
    }

    /// The device that owns a file extension (any case, no dot).
    pub fn by_extension(&self, ext: &str) -> Option<&'static dyn Device> {
        let ext = ext.to_ascii_lowercase();
        self.iter().find(|d| d.extension().contains(&ext.as_str()))
    }
}

impl Default for DeviceList {
    fn default() -> Self {
        Self::new()
    }
}
