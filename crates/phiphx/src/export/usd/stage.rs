//! The prims a stage is made of, as `UsdaWriter` writes them: a mesh
//! (faces, per-corner normals, UVs, colour, material binding and
//! per-face material subsets) and a `UsdPreviewSurface` material —
//! RealityKit ignores `displayColor`, so the material is what it shows.

use super::UsdaWriter;
use super::name::usd_word;
use crate::format::PreviewSurfaceMaterial;
use std::fmt::Write;

/// One mesh prim as the writer takes it: polygons of `count` corners over
/// `point`, one normal per corner (`normal`, may be empty; one per point
/// where `normal_by_point`), one UV per
/// point (`uv`, may be empty), a constant display colour, the material
/// path it is bound to, and faces bound to other materials.
#[derive(Clone, Debug, Default)]
pub struct UsdMesh<'a> {
    pub name: &'a str,
    pub point: &'a [[f32; 3]],
    pub count: &'a [u32],
    pub index: &'a [u32],
    pub normal: &'a [[f32; 3]],
    pub normal_by_point: bool,
    pub uv: &'a [[f32; 2]],
    pub colour: [f32; 3],
    /// A colour for each point, where its colour turns across it (may
    /// be empty: then `colour`, constant).
    pub colours: &'a [[f32; 3]],
    pub look: Option<String>,
    /// `(subset name, material path, face indices)`.
    pub subset: Vec<(String, String, Vec<u32>)>,
}

const BINDING: &str = "prepend apiSchemas = [\"MaterialBindingAPI\"]";

impl UsdaWriter {
    /// A mesh prim (see [`UsdMesh`]).
    pub fn mesh(&mut self, mesh: &UsdMesh<'_>) {
        self.push_scope_with(mesh.name, "Mesh", &[BINDING]);
        let [lo, hi] = extent(mesh.point);
        self.line(&format!("float3[] extent = [{}, {}]", three(lo), three(hi)));
        self.line(&format!("int[] faceVertexCounts = [{}]", list(mesh.count.iter())));
        self.line(&format!("int[] faceVertexIndices = [{}]", list(mesh.index.iter())));
        self.line(&format!("point3f[] points = [{}]", list(mesh.point.iter().map(|p| three(*p)))));
        if !mesh.normal.is_empty() {
            self.line(&format!("normal3f[] normals = [{}] (", list(mesh.normal.iter().map(|n| three(*n)))));
            match mesh.normal_by_point {
                true => self.line("    interpolation = \"vertex\""),
                false => self.line("    interpolation = \"faceVarying\""),
            }
            self.line(")");
        }
        if !mesh.uv.is_empty() && mesh.uv.len() == mesh.point.len() {
            self.line(&format!("texCoord2f[] primvars:st = [{}] (", list(mesh.uv.iter().map(|t| format!("({}, {})", t[0], t[1])))));
            self.line("    interpolation = \"vertex\"");
            self.line(")");
        }
        match mesh.colours.len() == mesh.point.len() && !mesh.colours.is_empty() {
            true => {
                self.line(&format!("color3f[] primvars:displayColor = [{}] (", list(mesh.colours.iter().map(|c| three(*c)))));
                self.line("    interpolation = \"vertex\"");
            }
            false => {
                self.line(&format!("color3f[] primvars:displayColor = [{}] (", three(mesh.colour)));
                self.line("    interpolation = \"constant\"");
            }
        }
        self.line(")");
        self.line("uniform token subdivisionScheme = \"none\"");
        if let Some(look) = &mesh.look {
            self.line(&format!("rel material:binding = <{look}>"));
        }
        if !mesh.subset.is_empty() {
            self.line("uniform token subsetFamily:materialBind:familyType = \"partition\"");
        }
        for (name, look, face) in &mesh.subset {
            self.push_scope_with(name, "GeomSubset", &[BINDING]);
            self.line("uniform token elementType = \"face\"");
            self.line("uniform token familyName = \"materialBind\"");
            self.line(&format!("int[] indices = [{}]", list(face.iter())));
            self.line(&format!("rel material:binding = <{look}>"));
            self.pop_scope();
        }
        self.pop_scope();
    }

    /// A material prim named `name` under `scope` (a prim path), its
    /// surface a `UsdPreviewSurface` with `look`'s colour and finish.
    pub fn look(&mut self, scope: &str, name: &str, look: &PreviewSurfaceMaterial) {
        let shader = usd_word("shader");
        self.push_scope(name, "Material");
        self.line(&format!("token outputs:surface.connect = <{scope}/{name}/{shader}.outputs:surface>"));
        self.push_scope(shader, "Shader");
        self.line("uniform token info:id = \"UsdPreviewSurface\"");
        self.line(&format!("color3f inputs:diffuseColor = {}", three(look.diffuse_color)));
        self.line(&format!("float inputs:roughness = {}", look.roughness));
        self.line(&format!("float inputs:metallic = {}", look.metallic));
        if look.opacity < 1.0 {
            self.line(&format!("float inputs:opacity = {}", look.opacity));
        }
        if look.emissive_color.iter().any(|c| *c > 0.0) {
            self.line(&format!("color3f inputs:emissiveColor = {}", three(look.emissive_color.map(|c| c * look.emissive_intensity))));
        }
        self.line("token outputs:surface");
        self.pop_scope();
        self.pop_scope();
    }

    /// A transform as `xformOp:transform` (USD's row-vector layout: the
    /// fourth row is the translation) and its op order.
    pub fn transform(&mut self, rows: &[[f32; 4]; 4]) {
        let row = rows.map(|r| format!("({}, {}, {}, {})", r[0], r[1], r[2], r[3]));
        self.line(&format!("matrix4d xformOp:transform = ( {}, {}, {}, {} )", row[0], row[1], row[2], row[3]));
        self.line("uniform token[] xformOpOrder = [\"xformOp:transform\"]");
    }
}

/// The smallest and largest corner of the points' box (zero when empty).
pub fn extent(point: &[[f32; 3]]) -> [[f32; 3]; 2] {
    match point.is_empty() {
        true => [[0.0; 3]; 2],
        false => point.iter().fold([[f32::MAX; 3], [f32::MIN; 3]], |[lo, hi], p| {
            [[0, 1, 2].map(|i| lo[i].min(p[i])), [0, 1, 2].map(|i| hi[i].max(p[i]))]
        }),
    }
}

fn three(v: [f32; 3]) -> String {
    format!("({}, {}, {})", v[0], v[1], v[2])
}

fn list<T: std::fmt::Display>(items: impl Iterator<Item = T>) -> String {
    let mut out = String::new();
    for (i, item) in items.enumerate() {
        let sep = match i {
            0 => "",
            _ => ", ",
        };
        let _ = write!(out, "{sep}{item}");
    }
    out
}
