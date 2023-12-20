struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0) var<uniform> mvp: mat4x4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.position = mvp * vec4(position, 1.0);
    return result;
}

struct Material {
    ambient : vec3<f32>,
    diffuse : vec3<f32>,
    specular : vec3<f32>,
    shininess : f32,
}

@group(1) @binding(0) var<uniform> material : Material;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(vec3<f32>(vertex.position.w, vertex.position.w, vertex.position.w), 1.0);
}