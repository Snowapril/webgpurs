use crate::point_cloud::e57_reader;
use anyhow::Result;

struct Batch {
    offset: u32,
    num_points: u32,
}

struct PointCloud {
    batches: Vec<Batch>,
    point_xyz_list: Vec<glam::Vec3>,
}

fn organize_batch(_xyz_list: &Vec<glam::Vec3>) -> Result<Vec<Batch>> {
    Ok(Vec::<Batch>::new())
}

impl From<&'static str> for PointCloud {
    fn from(e57_path: &'static str) -> Self {
        let point_xyz_list = e57_reader::read_e57(e57_path).unwrap_or_else(|_| {
            log::error!("Failed to load e57 file from {}", e57_path);
            Vec::<glam::Vec3>::new()
        });

        let batches = organize_batch(&point_xyz_list).unwrap_or_else(|_| {
            log::error!("Failed to load e57 file from {}", e57_path);
            Vec::<Batch>::new()
        });

        Self {
            batches,
            point_xyz_list,
        }
    }
}
