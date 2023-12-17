use anyhow::Result;
use bytemuck::{bytes_of, Pod, Zeroable};
use glam::Vec3;
use std::cell::Cell;
use wgpu::util::DeviceExt;

// TODO(snowapril) : use shared primitive buffer pool
pub struct StaticMesh {
    pub(crate) name: String,
    pub(crate) positions: Vec<glam::Vec3>,
    pub(crate) normals: Vec<glam::Vec3>,
    pub(crate) uvs: Vec<glam::Vec2>,
    pub(crate) indices: Vec<u32>,
    pub(crate) material_id: Option<usize>,
}

// Does not support texture ye
#[derive(Clone)]
pub struct Material {
    pub(crate) name: String,
    pub(crate) ambient: glam::Vec3,
    pub(crate) diffuse: glam::Vec3,
    pub(crate) specular: glam::Vec3,
    pub(crate) shininess: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: String::from("Default material"),
            ambient: glam::Vec3::new(0.8, 0.8, 0.8),
            diffuse: glam::Vec3::new(0.8, 0.8, 0.8),
            specular: glam::Vec3::new(1.0, 1.0, 1.0),
            shininess: 0.5,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VertexPod {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialPod {
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
    shininess: f32,
    #[allow(dead_code)]
    _padding: [f32; 6], // TODO(snowapril) : how to aligning struct not like this in rust lang
}

fn create_vertex_pod(pos: glam::Vec3, normal: glam::Vec3, tex_coord: glam::Vec2) -> VertexPod {
    VertexPod {
        position: [pos.x, pos.y, pos.z],
        normal: [normal.x, normal.y, normal.z],
        tex_coord: [tex_coord.x, tex_coord.y],
    }
}

fn create_material_pod(material: &Material) -> MaterialPod {
    MaterialPod {
        ambient: [
            material.ambient[0],
            material.ambient[1],
            material.ambient[2],
        ],
        diffuse: [
            material.diffuse[0],
            material.diffuse[1],
            material.diffuse[2],
        ],
        specular: [
            material.specular[0],
            material.specular[1],
            material.specular[2],
        ],
        shininess: material.shininess,
        _padding: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    }
}

// TODO(snowapril) : share same material with other scene objects.
pub struct SceneObject {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub material: wgpu::Buffer,
}

impl SceneObject {
    pub fn create(device: &wgpu::Device, mesh: &StaticMesh, material: &Material) -> Result<Self> {
        let num_vertices = mesh.positions.len();
        let vertices = (0..num_vertices)
            .into_iter()
            // .map(|i| create_vertex_pod(mesh.positions[i], mesh.normals[i], mesh.uvs[i])) TODO(snowapril)
            .map(|i| {
                create_vertex_pod(
                    mesh.positions[i],
                    glam::Vec3::new(0.0, 1.0, 0.0),
                    glam::Vec2::new(0.0, 0.0),
                )
            })
            .collect::<Vec<VertexPod>>();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Vertex Buffer [ {} ]", mesh.name.as_str()).as_str()),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Index Buffer [ {} ]", mesh.name.as_str()).as_str()),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let material_pod = create_material_pod(material);
        let material = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("Material Buffer [ {} ]", mesh.name.as_str()).as_str()),
            contents: bytemuck::cast_slice(&[material_pod]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        Ok(Self {
            name: mesh.name.clone(),
            vertex_buffer,
            index_buffer,
            material,
        })
    }
}
