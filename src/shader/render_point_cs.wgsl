@group(0) @binding(0) var<uniform> world_view_proj: mat4x4<f32>;

struct FrameBuffer {
  depth: u32, // 32 bits | high 8bits of color
  color: u32, // low 24bits -> color rgb. 8bits per channel  
};

@group(0) @binding(1) var<storage, read_write> frame_buffer: <arrayFrameBuffer>;

@group(1) @binding(0) var<storage, read> point_cloud_positions: array<vec3<f32>>;
@group(1) @binding(1) var<storage, read> point_cloud_colors: array<vec3<f32>>;

fn cast_float_to_i32(value : f32) -> i32 {
    (value * 65536.0) as i32
}

@compute
@workgroup_size(1, 1, 1)
fn render_point_cs(@builtin(local_invocation_id) lid: vec3<u32>, @builtin(workgroup_id) wid: vec3<u32>) {
    
    let point_index : u32 = global_id.x;

    let local_pos : vec3<f32> = point_cloud_positions[global_id.x];
    let world_pos : vec4<f32> = world_view_proj * vec4(local_pos.x, local_pos.y, local_pos.z, 1.0);
    let pixel_id = 0u; // to_pixel_id(pos);
    let depth : u32 = cast_float_to_i32(pos.w);
    let new_point = (depth << 32) | point_index;
    //let old_point : u32 = frame_buffer[pixel_id];
    //if (new_point < old_point) {
    //    // atomicMin(frame_buffers[pixel_id], new_point);
    //    frame_buffer[pixel_id] = min(frame_buffer[pixel_id], new_point);
    //}
}
