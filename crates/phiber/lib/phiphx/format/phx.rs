//! PHX format writer.

use std::io::Write;

use crate::phx::point_cloud::PointCloud;

/// Write a point cloud in ASCII PHX format.
pub fn write_phx<W: Write>(cloud: &PointCloud, writer: &mut W) -> std::io::Result<()> {
    writeln!(writer, "phx")?;
    writeln!(writer, "format ascii 1.0")?;
    writeln!(writer, "element vertex {}", cloud.len())?;
    writeln!(writer, "property float x")?;
    writeln!(writer, "property float y")?;
    writeln!(writer, "property float z")?;
    writeln!(writer, "end_header")?;

    for point in &cloud.points {
        writeln!(
            writer,
            "{} {} {}",
            point.position.x, point.position.y, point.position.z
        )?;
    }

    Ok(())
}
