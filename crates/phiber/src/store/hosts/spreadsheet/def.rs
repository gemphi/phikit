//! Spreadsheet host definitions matching Blender 5.2 `SpaceSpreadsheet` in `DNA_space_types.h`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GeometryComponentType {
    Mesh = 0,
    PointCloud = 1,
    Curves = 2,
    Volume = 3,
    Instances = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AttributeDomain {
    Point = 0,
    Edge = 1,
    Face = 2,
    Corner = 3,
    Spline = 4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpreadsheetConfig {
    pub object_name: Option<String>,
    pub component: GeometryComponentType,
    pub domain: AttributeDomain,
}

impl Default for SpreadsheetConfig {
    fn default() -> Self {
        Self {
            object_name: None,
            component: GeometryComponentType::Mesh,
            domain: AttributeDomain::Point,
        }
    }
}
