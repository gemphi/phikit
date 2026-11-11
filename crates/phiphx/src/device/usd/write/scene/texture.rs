//! The image nodes of a `UsdPreviewSurface`: a UV reader and `UsdUVTexture`s.
use super::out::UsdOut;
use super::text::UsdText;

/// One image feeding one input of the surface.
pub(super) struct UsdTexture<'a> {
    /// Shader prim name under the material ("Base", "Rough", ...).
    pub name: &'static str,
    /// Image key as the scene holds it: a Blender path or an image name.
    pub file: &'a str,
    /// Colour data (base colour, emission) is sRGB; roughness, metallic and
    /// normal maps are raw numbers.
    pub srgb: bool,
    /// The input it replaces, e.g. `color3f inputs:diffuseColor`.
    pub input: &'static str,
    /// The output it feeds it from: `rgb` or `r`.
    pub channel: &'static str,
}

impl UsdTexture<'_> {
    /// The `UsdPrimvarReader_float2` every texture of a material shares; it
    /// reads the `st` primvar named on the material (`inputs:frame:stPrimvarName`).
    pub fn write_reader(out: &mut UsdOut, material: &str) {
        out.open("Shader", "Texcoord", &[]);
        out.line("uniform token info:id = \"UsdPrimvarReader_float2\"");
        out.line(&format!("token inputs:varname.connect = <{material}.inputs:frame:stPrimvarName>"));
        out.line("float2 outputs:result");
        out.close();
    }

    /// The `UsdUVTexture`; a normal map also decodes `[0, 1]` to `[-1, 1]`.
    pub fn write(&self, out: &mut UsdOut, material: &str) {
        out.open("Shader", self.name, &[]);
        out.line("uniform token info:id = \"UsdUVTexture\"");
        out.line(&format!("asset inputs:file = {}", UsdText::asset(self.file)));
        out.line(&format!("float2 inputs:st.connect = <{material}/Texcoord.outputs:result>"));
        out.line("token inputs:wrapS = \"repeat\"");
        out.line("token inputs:wrapT = \"repeat\"");
        out.line(&format!("token inputs:sourceColorSpace = \"{}\"", if self.srgb { "sRGB" } else { "raw" }));
        if self.input.contains("normal") {
            out.line("float4 inputs:scale = (2, 2, 2, 1)");
            out.line("float4 inputs:bias = (-1, -1, -1, 0)");
        }
        let ty = if self.channel == "rgb" { "float3" } else { "float" };
        out.line(&format!("{ty} outputs:{}", self.channel));
        out.close();
    }
}
