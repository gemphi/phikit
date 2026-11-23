#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SCNShaderModifierEntryPoint {
    Geometry,
    Surface,
    LightingModel,
    Fragment,
}

impl SCNShaderModifierEntryPoint {
    pub fn as_str(&self) -> &'static str {
        match self {
            SCNShaderModifierEntryPoint::Geometry => "SCNShaderModifierEntryPointGeometry",
            SCNShaderModifierEntryPoint::Surface => "SCNShaderModifierEntryPointSurface",
            SCNShaderModifierEntryPoint::LightingModel => "SCNShaderModifierEntryPointLightingModel",
            SCNShaderModifierEntryPoint::Fragment => "SCNShaderModifierEntryPointFragment",
        }
    }
}
