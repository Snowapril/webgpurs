@group(0) @binding(0) var<uniform> model_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> view_projection_matrix: mat4x4<f32>;
@group(0) @binding(2) var<uniform> view_projection_matrix_inverse: mat4x4<f32>;
@group(0) @binding(3) var<uniform> normal_matrix: mat4x4<f32>;

@group(1) @binding(0) var<storage, read> positions: array<vec3<f32>>;
@group(1) @binding(1) var<storage, read> normals: array<vec3<f32>>;
@group(1) @binding(2) var<storage, read> texcoords: array<vec3<f32>>;
@group(1) @binding(3) var<storage, read_write> projected_position: array<vec3<f32>>;
@group(1) @binding(4) var<storage, read_write> projected_world_position: array<vec3<f32>>;
@group(1) @binding(5) var<storage, read_write> projected_normals: array<vec3<f32>>;
@group(1) @binding(6) var<storage, read_write> projected_texcoords: array<vec3<f32>>;
@group(1) @binding(7) var<storage, read_write> traignel_aabb: array<vec4<f32>>;

// select axis that generate biggest projection plane for each voxel faces
fn calculate_axis(positions : array<vec4<f32>, 3>) -> u32 {
    let p1 = positions[1].xyz - positions[0].xyz;
    let p2 = positions[2].xyz - positions[0].xyz;

    let face_normal = cross(p1, p2);

    let nx = abs(face_normal.x);
    let ny = abs(face_normal.y);
    let nz = abs(face_normal.z);

    if (nx > ny && nx > nz) {
        return 0u;
    }
    else if (ny > nx && ny > nz) {
        return 1u;
    }
    else {
        return 2u;
    }
}

fn axis_aligned_bounding_box(positions : array<vec4<f32>, 3>, half_pixel : vec2<f32>) -> vec4<f32> {
    let aa = vec2<f32>(min(positions[0].xy, min(positions[1].xy, positions[2].xy)));
    return vec4<f32>(
        aa - half_pixel,
        aa + half_pixel,
    );
}

struct VoxelConstants {
    volume_dim : u32,
    world_min_point : vec3<f32>,
    voxel_scale : f32,
};

var<push_constant> voxel_constants : VoxelConstants;

@compute
@workgroup_size(1)
fn voxel_projection_cs(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let num_total_vertices = arrayLength(&positions);
    let tid = global_id.x;
    if (tid >= (num_total_vertices / 3u)) {
        return;
    }

    let world_positions = array<vec4<f32>, 3>(
        model_matrix * vec4(positions[tid * 3u], 1.0), 
        model_matrix * vec4(positions[tid * 3u + 1u], 1.0), 
        model_matrix * vec4(positions[tid * 3u + 2u], 1.0) 
    );
    let normals = array<vec4<f32>, 3>( 
        normal_matrix * vec4(normals[tid * 3u], 1.0),
        normal_matrix * vec4(normals[tid * 3u + 1u], 1.0),
        normal_matrix * vec4(normals[tid * 3u + 2u], 1.0) 
    );

    let axis_index = calculate_axis(world_positions);

    let clip_space_positions = array<vec4<f32>, 3>(
        view_projection_matrix * world_positions[0],
        view_projection_matrix * world_positions[1],
        view_projection_matrix * world_positions[2],
    );

    let triangle_plane_normal = normalize(
        cross(clip_space_positions[1].xyz - clip_space_positions[0].xyz,
              clip_space_positions[2].xyz - clip_space_positions[0].xyz), 
    );
    let triangle_plane_eq = vec4<f32>(triangle_plane_normal, -dot(clip_space_positions[0].xyz, triangle_plane_normal));

    let half_pixel = vec2<f32>(1.0 / f32(voxel_constants.volume_dim));

    if (triangle_plane_normal.z == 0.0) {
        return;
    }

    traignel_aabb[tid] = axis_aligned_bounding_box(clip_space_positions, half_pixel);

    var planes = array<vec3<f32>, 3>(
        cross(clip_space_positions[0].xyw - clip_space_positions[2].xyw, clip_space_positions[2].xyw),
        cross(clip_space_positions[1].xyw - clip_space_positions[0].xyw, clip_space_positions[0].xyw),
        cross(clip_space_positions[2].xyw - clip_space_positions[1].xyw, clip_space_positions[1].xyw)
    );
	planes[0].z -= dot(half_pixel, abs(planes[0].xy));
	planes[1].z -= dot(half_pixel, abs(planes[1].xy));
	planes[2].z -= dot(half_pixel, abs(planes[2].xy));

    var intersection = array<vec3<f32>, 3>(
        cross(planes[0], planes[1]),
        cross(planes[1], planes[2]),
        cross(planes[2], planes[0])
    );
	intersection[0] /= intersection[0].z;
	intersection[1] /= intersection[1].z;
	intersection[2] /= intersection[2].z;

    var z = array<f32, 3>(
        -(intersection[0].x * triangle_plane_eq.x + intersection[0].y * triangle_plane_eq.y + triangle_plane_eq.w) / triangle_plane_eq.z,
        -(intersection[1].x * triangle_plane_eq.x + intersection[1].y * triangle_plane_eq.y + triangle_plane_eq.w) / triangle_plane_eq.z,
        -(intersection[2].x * triangle_plane_eq.x + intersection[2].y * triangle_plane_eq.y + triangle_plane_eq.w) / triangle_plane_eq.z
    );

    let dilated_positions = array<vec4<f32>, 3>(
        vec4<f32>(vec3<f32>(intersection[0].xy, z[0]), clip_space_positions[0].w),
        vec4<f32>(vec3<f32>(intersection[1].xy, z[1]), clip_space_positions[1].w),
        vec4<f32>(vec3<f32>(intersection[2].xy, z[2]), clip_space_positions[2].w),
    );

    for (var i = 0u; i < 3u; i++) {
        let voxel_pos = view_projection_matrix_inverse * dilated_positions[i];
        let transformed_voxel_pos = vec4<f32>(voxel_pos.xyz / voxel_pos.w - voxel_constants.world_min_point, voxel_pos.w) * voxel_constants.voxel_scale;
        projected_position[tid * 3u + i] = dilated_positions[i].xyz;
        projected_normals[tid * 3u + i] = normals[tid * 3u + i];
        projected_world_position[tid * 3u + i] = transformed_voxel_pos.xyz * voxel_constants.volume_dim;
    }
}