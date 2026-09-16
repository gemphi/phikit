//! Shared node graph data model for phiber's Shader / Compositor / GeoNode
//! editors.
//!
//! Pure Rust — no gpui, no OS deps. Consumers reach these types via
//! `apple::services::node_graph::…`.
//!
//! Requirements: 22.1, 23.1, 24.1, 25.1

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ── Tree kind ────────────────────────────────────────────────────────────────

/// Which Blender node editor a graph belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BNodeTreeKind {
    Shader,
    Compositor,
    GeoNode,
}

// ── Sockets ──────────────────────────────────────────────────────────────────

/// Direction of a socket on a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BSocketDir {
    Input,
    Output,
}

/// Approximate value type carried on a socket.
///
/// Requirements 22–25 do not enumerate Blender's full DNA_NODE socket type
/// set; this is the conservative default covering the categories referenced
/// by the Shader / Compositor / GeoNode requirements. Follow-up work can
/// extend the enum without breaking the graph API — the socket type is
/// opaque to `add_edge` / `remove_edge` / `serialize`; only `BSocketDir`
/// participates in the directional check per Requirements 22.6, 22.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BSocketType {
    Value,
    Vector,
    Color,
    Shader,
    Geometry,
    Boolean,
    Integer,
    Float,
    String,
}

/// A named socket on a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BSocket {
    pub name: String,
    pub dir: BSocketDir,
    pub ty: BSocketType,
}

// ── Nodes ────────────────────────────────────────────────────────────────────

/// A node inside a `BNodeGraph`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BNode {
    pub id: u32,
    pub kind: BNodeKind,
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub inputs: Vec<BSocket>,
    #[serde(default)]
    pub outputs: Vec<BSocket>,
}

// ── Edges ────────────────────────────────────────────────────────────────────

/// Directed connection between two sockets.
///
/// `from` / `to` are `(node_id, socket_index)` pairs. `from` must reference
/// an output socket and `to` an input socket; validation lives in
/// `add_edge.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BEdge {
    pub from: (u32, u32),
    pub to: (u32, u32),
}

/// Error returned by `add_edge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BEdgeError {
    /// Both endpoints belong to the same node (Requirement 22.7).
    SelfEdge,
    /// Accepting the edge would create a cycle (Requirement 22.7).
    Cycle,
    /// Either endpoint id is absent from the graph.
    UnknownNode,
    /// Endpoints have incompatible directions (input↔input or output↔output).
    SocketDir,
}

/// Error returned by `deserialize`.
#[derive(Debug, Clone, PartialEq)]
pub enum BNodeParseError {
    Json(String),
}

impl core::fmt::Display for BNodeParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BNodeParseError::Json(m) => write!(f, "malformed node-graph JSON: {m}"),
        }
    }
}
impl std::error::Error for BNodeParseError {}

// ── Graph ────────────────────────────────────────────────────────────────────

/// A single node graph — Shader, Compositor, or GeoNode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BNodeGraph {
    pub kind: BNodeTreeKind,
    pub nodes: Vec<BNode>,
    pub edges: Vec<BEdge>,
    pub next_id: u32,
}

impl BNodeGraph {
    pub fn new(kind: BNodeTreeKind) -> Self {
        Self { kind, nodes: Vec::new(), edges: Vec::new(), next_id: 0 }
    }
}

// ── Node kinds ───────────────────────────────────────────────────────────────

/// Every node type surfaced by the Shader, Compositor, and Geometry Nodes
/// `Add_Menu`s.
///
/// Variants are prefixed by tree — `Sh` for Shader, `Cmp` for Compositor,
/// `Geo` for GeoNode — to disambiguate the many nodes Blender exposes with
/// the same display name across editors (e.g. `Math` in Shader vs GeoNode,
/// `RGB` in Shader Input vs Compositor Input). Prefix-per-tree matches
/// Blender's internal SDNA type names (`ShaderNodeMath`, `CompositorNodeMath`,
/// `GeometryNodeMath`).
///
/// Requirements 23.2–23.9, 24.2–24.5, 25.2–25.4 enumerate every entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BNodeKind {
    // ── Shader / Input (Requirement 23.2) ────────────────────────────────
    ShAmbientOcclusion, ShAttribute, ShBevel, ShCameraData, ShCurvesInfo,
    ShFresnel, ShGeometry, ShHairInfo, ShLayerWeight, ShLightPath,
    ShObjectInfo, ShParticleInfo, ShPointInfo, ShRgb, ShTangent,
    ShTextureCoordinate, ShUvMap, ShValue, ShVertexColor, ShVolumeInfo,
    ShWireframe,

    // ── Shader / Output (Requirement 23.3) ───────────────────────────────
    ShAovOutput, ShLightOutput, ShLineStyleOutput, ShMaterialOutput, ShWorldOutput,

    // ── Shader / Shader (Requirement 23.4) ───────────────────────────────
    ShAddShader, ShAnisotropicBsdf, ShBackground, ShDiffuseBsdf, ShEmission,
    ShGlassBsdf, ShGlossyBsdf, ShHairBsdf, ShHoldout, ShMetallicBsdf,
    ShMixShader, ShPrincipledBsdf, ShPrincipledHairBsdf, ShPrincipledVolume,
    ShRayPortalBsdf, ShRefractionBsdf, ShSheenBsdf, ShSpecularBsdf,
    ShSubsurfaceScattering, ShToonBsdf, ShTranslucentBsdf, ShTransparentBsdf,
    ShVolumeAbsorption, ShVolumeScatter,

    // ── Shader / Texture (Requirement 23.5) ──────────────────────────────
    ShBrickTexture, ShCheckerTexture, ShEnvironmentTexture, ShGaborTexture,
    ShGradientTexture, ShIesTexture, ShImageTexture, ShMagicTexture,
    ShMusgraveTexture, ShNoiseTexture, ShPointDensity, ShSkyTexture,
    ShVoronoiTexture, ShWaveTexture, ShWhiteNoiseTexture,

    // ── Shader / Color (Requirement 23.6) ────────────────────────────────
    ShBrightContrast, ShGamma, ShHueSaturationValue, ShInvert, ShLightFalloff,
    ShMixColor, ShRgbCurves,

    // ── Shader / Vector (Requirement 23.7) ───────────────────────────────
    ShBump, ShDisplacement, ShMapping, ShNormal, ShNormalMap, ShVectorCurves,
    ShVectorDisplacement, ShVectorRotate, ShVectorTransform,

    // ── Shader / Converter (Requirement 23.8) ────────────────────────────
    ShBlackbody, ShClamp, ShColorRamp, ShCombineColor, ShCombineXyz,
    ShFloatCurve, ShMapRange, ShMath, ShMix, ShRgbToBw, ShSeparateColor,
    ShSeparateXyz, ShShaderToRgb, ShVectorMath, ShWavelength,

    // ── Shader / Script (Requirement 23.9) ───────────────────────────────
    ShScript,

    // ── Compositor / Input (Requirement 24.2) ────────────────────────────
    CmpBokehImage, CmpImage, CmpMask, CmpMovieClip, CmpRenderLayers,
    CmpRgb, CmpSceneTime, CmpTimeCurve, CmpTrackPosition, CmpValue,

    // ── Compositor / Filter (Requirement 24.3) ───────────────────────────
    CmpAntiAliasing, CmpBlur, CmpBilateralBlur, CmpBokehBlur, CmpDefocus,
    CmpDenoise, CmpDespeckle, CmpDirectionalBlur, CmpDilateErode, CmpFilter,
    CmpGlare, CmpInpaint, CmpKuwahara, CmpPixelate, CmpPosterize,
    CmpSunBeams, CmpVectorBlur,

    // ── Compositor / Matte (Requirement 24.4) ────────────────────────────
    CmpBoxMask, CmpChannelKey, CmpChromaKey, CmpColorKey, CmpColorSpill,
    CmpCryptomatte, CmpCryptomatteLegacy, CmpDifferenceKey, CmpDistanceKey,
    CmpDoubleEdgeMask, CmpEllipseMask, CmpKeying, CmpKeyingScreen,
    CmpLuminanceKey,

    // ── Compositor / Distort (Requirement 24.5) ──────────────────────────
    CmpCornerPin, CmpCrop, CmpDisplace, CmpFlip, CmpLensDistortion, CmpMapUv,
    CmpMovieDistortion, CmpPlaneTrackDeform, CmpRotate, CmpScale,
    CmpStabilize2d, CmpTransform, CmpTranslate,

    // ── GeoNode / Mesh (Requirement 25.2) ────────────────────────────────
    GeoExtrudeMesh, GeoMergeByDistance, GeoSubdivideMesh, GeoSplitEdges,
    GeoDualMesh, GeoFlipFaces,

    // ── GeoNode / Curve (Requirement 25.3) ───────────────────────────────
    GeoCurveLength, GeoCurveToMesh, GeoCurveToPoints, GeoDeformCurvesOnSurface,
    GeoFillCurve, GeoFilletCurve, GeoResampleCurve, GeoReverseCurve,
    GeoSampleCurve, GeoSetCurveRadius, GeoSetCurveTilt, GeoSubdivideCurve,
    GeoTrimCurve,

    // ── GeoNode / Utilities (Requirement 25.4) ───────────────────────────
    GeoBooleanMath, GeoCompare, GeoFloatCurve, GeoFloatToInteger, GeoMapRange,
    GeoMath, GeoRandomValue, GeoSwitch,
}

impl BNodeKind {
    /// The tree this node kind belongs to. Every variant maps to exactly
    /// one tree per its prefix (Sh → Shader, Cmp → Compositor, Geo → GeoNode).
    pub fn tree(self) -> BNodeTreeKind {
        // Match on the enum discriminant via the debug name — cheap because
        // enum discriminants are compile-time constants; the compiler
        // collapses the string test to an integer comparison at higher
        // opt levels.
        let n = format!("{self:?}");
        if n.starts_with("Sh") { BNodeTreeKind::Shader }
        else if n.starts_with("Cmp") { BNodeTreeKind::Compositor }
        else { BNodeTreeKind::GeoNode }
    }
}
