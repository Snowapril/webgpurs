use crate::point_cloud::e57_reader;
use anyhow::Result;

pub(crate) struct Batch {
    offset: u32,
    num_points: u32,
}

pub(crate) struct PointCloud {
    batches: Vec<Batch>,
    point_xyz_list: Vec<glam::Vec3>,
}

fn organize_batch(_xyz_list: &Vec<glam::Vec3>) -> Result<Vec<Batch>> {
    Ok(Vec::<Batch>::new())
}

impl From<&String> for PointCloud {
    fn from(e57_path: &String) -> Self {
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
