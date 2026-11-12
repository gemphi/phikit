// SPDX-License-Identifier: Apache-2.0
//! What the harness can fail at, apart from a render failing the judge.
use crate::device::DeviceError;

#[derive(Debug, Clone, PartialEq)]
pub enum TruthError {
    /// An image could not be read or written.
    Image { path: String, message: String },
    /// The two images are not the same size, which `oiiotool --diff` fails.
    Size { reference: (usize, usize), render: (usize, usize) },
    /// The folder of the test scenes was not found.
    NoRoot(String),
    /// The device stopped the render: the kernels are not all ported.
    Device(DeviceError),
    /// The scene asks for what the render does not do yet.
    Unsupported(String),
}

impl std::fmt::Display for TruthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Image { path, message } => write!(f, "image {path}: {message}"),
            Self::Size { reference, render } => write!(f, "the reference is {}x{} and the render {}x{}", reference.0, reference.1, render.0, render.1),
            Self::NoRoot(place) => write!(f, "no test scenes at {place}"),
            Self::Device(error) => write!(f, "device: {error}"),
            Self::Unsupported(what) => write!(f, "not supported yet: {what}"),
        }
    }
}

impl std::error::Error for TruthError {}

impl From<DeviceError> for TruthError {
    fn from(error: DeviceError) -> Self {
        Self::Device(error)
    }
}
