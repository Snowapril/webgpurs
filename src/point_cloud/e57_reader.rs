use anyhow::{Context, Result};
use e57::{CartesianCoordinate, E57Reader};

pub fn read_e57(e57_path: &'static str) -> Result<Vec<glam::Vec3>> {
    // Open E57 input file for reading
    let mut file = E57Reader::from_file(e57_path).context("Failed to open E57 file")?;

    let mut out_xyz_array: Vec<glam::Vec3> = Vec::new();

    // Loop over all point clouds in the E57 file
    let pointclouds = file.pointclouds();
    for pointcloud in pointclouds {
        let mut iter = file
            .pointcloud_simple(&pointcloud)
            .context("Unable to get point cloud iterator")?;

        // Set point iterator options
        iter.spherical_to_cartesian(true);
        iter.cartesian_to_spherical(false);
        iter.intensity_to_color(true);
        iter.apply_pose(true);

        // Iterate over all points in point cloud
        for p in iter {
            let p = p.context("Unable to read next point")?;

            // Write XYZ data to output file
            if let CartesianCoordinate::Valid { x, y, z } = p.cartesian {
                out_xyz_array.push(glam::Vec3::new(x as f32, y as f32, z as f32));
            } else {
                continue;
            }

            // If available, write RGB color or intensity color values
            // if let Some(color) = p.color {
            //     writer
            //         .write_fmt(format_args!(
            //             " {} {} {}",
            //             (color.red * 255.) as u8,
            //             (color.green * 255.) as u8,
            //             (color.blue * 255.) as u8
            //         ))
            //         .context("Failed to write RGB color")?;
            // }
        }
    }

    Ok(out_xyz_array)
}
