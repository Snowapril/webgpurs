#version 460

layout(set = 0, binding = 0) uniform CameraUniformBuffer
{
    mat4 model_matrix;
    mat4 view_projection_matrix;
    mat4 view_projection_matrix_inverse;
    mat4 normal_matrix;
};
layout(set = 1, binding = 0, std430) readonly buffer InputPosBuffer { vec3 positions[]; };
layout(set = 1, binding = 1, std430) readonly buffer InputNormalBuffer { vec3 normals[]; };
layout(set = 1, binding = 2, std430) readonly buffer InputUVBuffer { vec3 texcoords[]; };
layout(set = 1, binding = 3, std430) buffer OutPosBuffer { vec3 projected_position[]; };
layout(set = 1, binding = 4, std430) buffer OutWorldPosBuffer { vec3 projected_world_position[]; };
layout(set = 1, binding = 5, std430) buffer OutUVBuffer { vec3 projected_texcoords[]; };
layout(set = 1, binding = 6, std430) buffer OutAABB { vec4 traignel_aabb[]; };

layout( push_constant ) uniform VoxelConstants
{
    uint volume_dim;
    vec3 world_min_point;
    float voxel_scale;
};

// select axis that generate biggest projection plane for each voxel faces
uint calculate_axis(vec4 positions[3]) {
    vec3 p1 = positions[1].xyz - positions[0].xyz;
    vec3 p2 = positions[2].xyz - positions[0].xyz;

    vec3 face_normal = cross(p1, p2);

    float nx = abs(face_normal.x);
    float ny = abs(face_normal.y);
    float nz = abs(face_normal.z);

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

vec4 axis_aligned_bounding_box(vec4 positions[3], vec2 half_pixel) {
    vec2 aa = min(positions[0].xy, min(positions[1].xy, positions[2].xy));
    vec2 left_top = aa - half_pixel;
    vec2 right_bottom = aa + half_pixel;
    return vec4(left_top.x, left_top.y, right_bottom.x, right_bottom.y);
}

layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main() 
{
    uint num_total_vertices = positions.length();
    uint tid = gl_GlobalInvocationID.x;
    if (tid >= (num_total_vertices / 3u)) {
        return;
    }

    vec4 world_positions[3] = {
        model_matrix * vec4(positions[tid * 3u], 1.0), 
        model_matrix * vec4(positions[tid * 3u + 1u], 1.0), 
        model_matrix * vec4(positions[tid * 3u + 2u], 1.0) 
    };
    vec4 normals[3] = {
        normal_matrix * vec4(normals[tid * 3u], 1.0),
        normal_matrix * vec4(normals[tid * 3u + 1u], 1.0),
        normal_matrix * vec4(normals[tid * 3u + 2u], 1.0) 
    };

    uint axis_index = calculate_axis(world_positions);

    vec4 clip_space_positions[3] = {
        view_projection_matrix * world_positions[0],
        view_projection_matrix * world_positions[1],
        view_projection_matrix * world_positions[2],
    };

    vec3 triangle_plane_normal = normalize(
        cross(clip_space_positions[1].xyz - clip_space_positions[0].xyz,
              clip_space_positions[2].xyz - clip_space_positions[0].xyz)
    );
    vec4 triangle_plane_eq = vec4(triangle_plane_normal, -dot(clip_space_positions[0].xyz, triangle_plane_normal));

    vec2 half_pixel = vec2(1.0f / float(volume_dim));

    if (triangle_plane_normal.z == 0.0) {
        return;
    }

    traignel_aabb[tid] = axis_aligned_bounding_box(clip_space_positions, half_pixel);

    vec3 planes[3] = {
        cross(clip_space_positions[0].xyw - clip_space_positions[2].xyw, clip_space_positions[2].xyw),
        cross(clip_space_positions[1].xyw - clip_space_positions[0].xyw, clip_space_positions[0].xyw),
        cross(clip_space_positions[2].xyw - clip_space_positions[1].xyw, clip_space_positions[1].xyw)
    };
	planes[0].z -= dot(half_pixel, abs(planes[0].xy));
	planes[1].z -= dot(half_pixel, abs(planes[1].xy));
	planes[2].z -= dot(half_pixel, abs(planes[2].xy));

    vec3 intersection[3] = {
        cross(planes[0], planes[1]),
        cross(planes[1], planes[2]),
        cross(planes[2], planes[0])
    };
	intersection[0] /= intersection[0].z;
	intersection[1] /= intersection[1].z;
	intersection[2] /= intersection[2].z;

    float z[3] = {
        -(intersection[0].x * triangle_plane_eq.x + intersection[0].y * triangle_plane_eq.y + triangle_plane_eq.w) / triangle_plane_eq.z,
        -(intersection[1].x * triangle_plane_eq.x + intersection[1].y * triangle_plane_eq.y + triangle_plane_eq.w) / triangle_plane_eq.z,
        -(intersection[2].x * triangle_plane_eq.x + intersection[2].y * triangle_plane_eq.y + triangle_plane_eq.w) / triangle_plane_eq.z
    };

    vec4 dilated_positions[3] = {
        vec4(vec3(intersection[0].xy, z[0]), clip_space_positions[0].w),
        vec4(vec3(intersection[1].xy, z[1]), clip_space_positions[1].w),
        vec4(vec3(intersection[2].xy, z[2]), clip_space_positions[2].w)
    };

    // [manual unroll]
    for (int i = 0; i < 3; ++i) {
        vec4 voxel_pos = view_projection_matrix_inverse * dilated_positions[i];
        vec4 transformed_voxel_pos = vec4(voxel_pos.xyz / voxel_pos.w - world_min_point, voxel_pos.w) * voxel_scale;
        projected_position[tid * 3u + i] = dilated_positions[i].xyz;
        projected_world_position[tid * 3u + i] = transformed_voxel_pos.xyz * volume_dim;
    }

    // var voxel_positions = array<vec4<f32>, 3>(
    //     view_projection_matrix_inverse * dilated_positions[0],
    //     view_projection_matrix_inverse * dilated_positions[1],
    //     view_projection_matrix_inverse * dilated_positions[2]
    // );

    // var transformed_voxel_pos = array<vec4<f32>, 3>(
    //     vec4<f32>(voxel_positions[0].xyz / voxel_positions[0].w - world_min_point, voxel_positions[0].w) * voxel_scale,
    //     vec4<f32>(voxel_positions[1].xyz / voxel_positions[1].w - world_min_point, voxel_positions[1].w) * voxel_scale,
    //     vec4<f32>(voxel_positions[2].xyz / voxel_positions[2].w - world_min_point, voxel_positions[2].w) * voxel_scale,
    // );

    // projected_position[tid * 3u] = dilated_positions[0].xyz;
    // projected_world_position[tid * 3u] = transformed_voxel_pos[0].xyz * f32(volume_dim);

    // projected_position[tid * 3u + 1u] = dilated_positions[1].xyz;
    // projected_world_position[tid * 3u + 1u] = transformed_voxel_pos[1].xyz * f32(volume_dim);

    // projected_position[tid * 3u + 2u] = dilated_positions[2].xyz;
    // projected_world_position[tid * 3u + 2u] = transformed_voxel_pos[2].xyz * f32(volume_dim);
}