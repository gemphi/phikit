//! What a `.blend` file holds, read kind by kind: objects, meshes, curves,
//! text, scenes and assets. The container (header, SDNA, blocks) is in
//! `file`; Blender's own evaluation (modifiers, rigs, animation, physics,
//! shading) has folders of its own beside this one.
pub mod asset;
pub mod curve;
pub mod idprop;
pub mod mesh;
pub mod object;
pub mod scene;
pub mod text;
