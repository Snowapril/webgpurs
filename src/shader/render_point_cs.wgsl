@group(0) @binding(0) var<uniform> world_view_proj: mat4x4<f32>;

//struct FrameBuffer {
//    high_pos : array<u32>,
//    medium_pos : array<u32>,
//    low_pos : array<u32>,
//    color : vec4<f32>,
//};
@group(0) @binding(1) var<storage, read_write> frame_buffer: array<u32>;
@group(1) @binding(0) var<storage, read> positions: array<vec3<f32>>;

@compute
@workgroup_size(1)
fn render_point_cs(@builtin(global_invocation_id) global_id: vec3<u32>) {
    
    let point_index : u32 = global_id.x;

    let local_pos : vec3<f32> = positions[global_id.x];
    let world_pos : vec4<f32> = world_view_proj * vec4(local_pos.x, local_pos.y, local_pos.z, 1.0);
    let pixel_id = 0u; // to_pixel_id(pos);
    let depth : u32 = 0u; // cast_float_to_i64(pos.w);
    //let new_point : u32 = (depth << 32) | point_index;
    //let old_point : u32 = frame_buffer[pixel_id];
    //if (new_point < old_point) {
    //    // atomicMin(frame_buffers[pixel_id], new_point);
    //    frame_buffer[pixel_id] = min(frame_buffer[pixel_id], new_point);
    //}
}
