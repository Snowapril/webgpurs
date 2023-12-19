use crate::scene::scene_object;
use anyhow::Result;
use std::{fmt, path::Path};

pub fn load_scene_objects<P>(
    device: &wgpu::Device,
    obj_path: P,
) -> Result<Vec<scene_object::SceneObject>>
where
    P: AsRef<Path> + fmt::Debug,
{
    let (models, materials) = tobj::load_obj(&obj_path, &tobj::LoadOptions::default())?;

    let static_meshes = models
        .iter()
        .map(|model| load_model(model))
        .collect::<Result<Vec<scene_object::StaticMesh>>>()?
        .into_iter()
        .collect::<Vec<scene_object::StaticMesh>>();

    let materials = if let Ok(materials) = materials {
        materials
            .iter()
            .map(|material| load_material(material))
            .collect::<Result<Vec<scene_object::Material>>>()?
    } else {
        vec![]
    };

    static_meshes
        .into_iter()
        .map(|mesh| {
            let material = if let Some(material_id) = mesh.material_id {
                materials[material_id].clone()
            } else {
                scene_object::Material::default()
            };
            scene_object::SceneObject::create(device, &mesh, &material)
        })
        .collect::<Result<Vec<scene_object::SceneObject>>>()
}

fn load_model(model: &tobj::Model) -> Result<scene_object::StaticMesh> {
    let mut positions: Vec<glam::Vec3> = vec![];
    let mut normals: Vec<glam::Vec3> = vec![];
    let mut uvs: Vec<glam::Vec2> = vec![];

    let mesh = &model.mesh;
    // let mut next_face = 0;
    // for face in 0..mesh.face_arities.len() {
    //     let end = next_face + mesh.face_arities[face] as usize;

    //     let face_indices = &mesh.indices[next_face..end];

    //     if !mesh.texcoord_indices.is_empty() {
    //         let texcoord_face_indices = &mesh.texcoord_indices[next_face..end];
    //         for &texcoord_index in texcoord_face_indices {
    //         }
    //     }
    //     if !mesh.normal_indices.is_empty() {
    //         let normal_face_indices = &mesh.normal_indices[next_face..end];
    //         for &normal_index in normal_face_indices {
    //         }
    //     }
    //     next_face = end;
    // }

    for vtx in 0..mesh.positions.len() / 3 {
        positions.push(glam::Vec3::new(
            mesh.positions[3 * vtx],
            mesh.positions[3 * vtx + 1],
            mesh.positions[3 * vtx + 2],
        ))
    }

    let mut indices: Vec<u32> = vec![];
    let mut next_face = 0;
    for face in 0..mesh.face_arities.len() {
        let end = next_face + mesh.face_arities[face] as usize;

        let face_indices = &mesh.indices[next_face..end];
        indices.push(face_indices[0]);
        indices.push(face_indices[1]);
        indices.push(face_indices[2]);
        indices.push(face_indices[2]);
        indices.push(face_indices[1]);
        indices.push(face_indices[3]);

        next_face = end;
    }
    println!(" positions          = {:?}", positions);
    println!(" indices          = {:?}", indices);

    Ok(scene_object::StaticMesh {
        name: model.name.clone(),
        positions,
        normals,
        uvs,
        indices : mesh.indices.clone(),
        material_id: mesh.material_id,
    })
}

fn load_material(material: &tobj::Material) -> Result<scene_object::Material> {
    let ambient = material
        .ambient
        .map(|ambient| glam::Vec3::new(ambient[0], ambient[1], ambient[2]))
        .ok_or_else(|| anyhow::Error::msg("Essential material field 'ambient' missing"))?;

    let diffuse = material
        .diffuse
        .map(|diffuse| glam::Vec3::new(diffuse[0], diffuse[1], diffuse[2]))
        .ok_or_else(|| anyhow::Error::msg("Essential material field 'diffuse' missing"))?;

    let specular = material
        .specular
        .map(|specular| glam::Vec3::new(specular[0], specular[1], specular[2]))
        .ok_or_else(|| anyhow::Error::msg("Essential material field 'specular' missing"))?;

    let shininess = material
        .shininess
        .ok_or_else(|| anyhow::Error::msg("Essential material field 'shininess' missing"))?;

    Ok(scene_object::Material {
        name: material.name.clone(),
        ambient,
        diffuse,
        specular,
        shininess,
    })
}
