@group(0)
@binding(0)
var<storage, read_write> g_counter_buffer: array<u32>; // this is used as both input and output for convenience

@compute
@workgroup_size(1)
fn reset_counter_buffer(@builtin(global_invocation_id) global_id: vec3<u32>) {
    g_counter_buffer[global_id.x] = 0;
}

@compute
@workgroup_size(1)
fn process_visibility_cs(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // TODO(snowapril)
    g_counter_buffer[global_id.x] = 0;
}
