//! `SceneWorld` as a `DomeLight`.
use super::out::UsdOut;
use super::text::UsdText;
use crate::scene::SceneWorld;

/// Writes the world's light.
pub(super) struct UsdWorld;

impl UsdWorld {
    /// A DomeLight of the Background node: its colour times its strength,
    /// or, when the world names an Environment Texture, that image
    /// (`latlong`, spun about Z by the Mapping node's rotation) with the
    /// colour as its tint. A Sky Texture world is not written as a sky: it
    /// is baked to an image only at render time, so it falls back to the
    /// flat colour. Nothing is written for a black, textureless world.
    ///
    /// Whether a viewer puts the image's pole on +Z follows the stage's
    /// `upAxis`; that is not checked against a USD viewer here.
    pub fn write(out: &mut UsdOut, name: &str, world: &SceneWorld) {
        let bg = &world.background;
        if bg.texture.is_none() && (bg.strength <= 0.0 || bg.color == [0.0; 3]) {
            return;
        }
        out.open("DomeLight", name, &[]);
        if bg.texture.is_some() && bg.rotation != 0.0 {
            out.line(&format!("float xformOp:rotateZ = {}", UsdText::float(bg.rotation.to_degrees())));
            out.line("uniform token[] xformOpOrder = [\"xformOp:rotateZ\"]");
        }
        out.line(&format!("color3f inputs:color = {}", UsdText::vec3(bg.color)));
        out.line(&format!("float inputs:intensity = {}", UsdText::float(bg.strength)));
        if let Some(file) = &bg.texture {
            out.line(&format!("asset inputs:texture:file = {}", UsdText::asset(file)));
            out.line("token inputs:texture:format = \"latlong\"");
        }
        out.close();
    }
}
