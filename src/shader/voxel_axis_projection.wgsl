@group(0) @binding(0) var<uniform> model_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view_projection_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> normal_matrix: mat4x4<f32>;
@group(1) @binding(0) var<storage, read> positions: array<vec3<f32>>;
@group(1) @binding(1) var<storage, read> normals: array<vec3<f32>>;
@group(1) @binding(2) var<storage, read> texcoords: array<vec3<f32>>;
@group(1) @binding(3) var<storage, read_write> projected_position: array<vec3<f32>>;
@group(1) @binding(4) var<storage, read_write> projected_normals: array<vec3<f32>>;
@group(1) @binding(5) var<storage, read_write> projected_texcoords: array<vec3<f32>>;

@compute
@workgroup_size(1)
fn voxel_projection_cs(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let num_total_vertices = arraylength(&positions);
    let tid = global_invocation_id.x;
    if (index >= num_total_vertices / 3) {
        return;
    }

    let world_positions = model_matrix * positions[index * 3];
    let normal = normal_matrix * normals[index * 3];
    let 
}