//! Shared per-line state-machine helpers used by the USDA parsers.
//!
//! Both `import_usda` (point cloud) and `import_usda_as_mesh` (full mesh)
//! drive the same line-oriented scan: detect which attribute a line opens,
//! accumulate array values while inside an attribute, and build triangle
//! indices from `faceVertexCounts`/`faceVertexIndices`. These helpers hold
//! that shared logic so the two entry points stay small and identical in
//! behaviour.

use super::UsdImporter;

impl UsdImporter {
    /// Update the attribute flags from a mesh-body line.
    ///
    /// When a recognised attribute declaration is seen the matching flag is
    /// set and all the others cleared, mirroring the original chained
    /// `if/else` exactly. Unrecognised lines leave the flags untouched.
    pub(crate) fn set_attr_flags(
        trimmed: &str,
        in_points_attr: &mut bool,
        in_normals_attr: &mut bool,
        in_colors_attr: &mut bool,
        in_counts_attr: &mut bool,
        in_indices_attr: &mut bool,
    ) {
        if trimmed.starts_with("point3f[] points") {
            *in_points_attr = true;
            *in_normals_attr = false;
            *in_colors_attr = false;
            *in_counts_attr = false;
            *in_indices_attr = false;
        } else if trimmed.starts_with("vector3f[] normals")
            || trimmed.starts_with("normal3f[] normals")
        {
            *in_normals_attr = true;
            *in_points_attr = false;
            *in_colors_attr = false;
            *in_counts_attr = false;
            *in_indices_attr = false;
        } else if trimmed.starts_with("color3f[] primvars:displayColor")
            || trimmed.starts_with("color4f[] primvars:displayColor")
        {
            *in_colors_attr = true;
            *in_points_attr = false;
            *in_normals_attr = false;
            *in_counts_attr = false;
            *in_indices_attr = false;
        } else if trimmed.starts_with("int[] faceVertexCounts") {
            *in_counts_attr = true;
            *in_points_attr = false;
            *in_normals_attr = false;
            *in_colors_attr = false;
            *in_indices_attr = false;
        } else if trimmed.starts_with("int[] faceVertexIndices") {
            *in_indices_attr = true;
            *in_points_attr = false;
            *in_normals_attr = false;
            *in_colors_attr = false;
            *in_counts_attr = false;
        }
    }

    /// Accumulate a vec3 array line while `flag` is set, clearing it at `]`.
    pub(crate) fn accumulate_vec3(flag: &mut bool, out: &mut Vec<[f32; 3]>, trimmed: &str) {
        if *flag {
            if let Some(parsed) = Self::parse_vec3_array(trimmed) {
                out.extend(parsed);
                if trimmed.contains(']') {
                    *flag = false;
                }
            }
        }
    }

    /// Accumulate a color array line while `flag` is set, clearing it at `]`.
    pub(crate) fn accumulate_color(flag: &mut bool, out: &mut Vec<[f32; 4]>, trimmed: &str) {
        if *flag {
            if let Some(parsed) = Self::parse_color_array(trimmed) {
                out.extend(parsed);
                if trimmed.contains(']') {
                    *flag = false;
                }
            }
        }
    }

    /// Accumulate an int array line while `flag` is set, clearing it at `]`.
    pub(crate) fn accumulate_int(flag: &mut bool, out: &mut Vec<u32>, trimmed: &str) {
        if *flag {
            if let Some(parsed) = Self::parse_int_array(trimmed) {
                out.extend(parsed);
                if trimmed.contains(']') {
                    *flag = false;
                }
            }
        }
    }
}
