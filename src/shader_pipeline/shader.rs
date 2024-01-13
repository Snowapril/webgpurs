use anyhow::Result;
use std::borrow::Cow;

pub fn compile_glsl(
    shader_bytes: &str,
    shader_name: &'static str,
    entry_point: &'static str,
    stage: shaderc::ShaderKind,
) -> Result<Vec<u32>> {
    let mut compiler = shaderc::Compiler::new()
        .ok_or_else(|| anyhow::Error::msg("Failed to create shaderc compiler"))?;
    let mut options = shaderc::CompileOptions::new()
        .ok_or_else(|| anyhow::Error::msg("Failed to create shaderc Options"))?;
    Ok(compiler
        .compile_into_spirv(
            shader_bytes,
            stage,
            shader_name,
            entry_point,
            Some(&options),
        )?
        .as_binary()
        .into())
}

pub fn create_shader_module(
    device: &wgpu::Device,
    shader_bytes: &str,
    shader_name: &'static str,
    entry_point: &'static str,
    stage: shaderc::ShaderKind,
) -> Result<wgpu::ShaderModule> {
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(format!("{}_{}", shader_name, entry_point).as_str()),
        source: wgpu::ShaderSource::SpirV(Cow::Borrowed(
            compile_glsl(shader_bytes, shader_name, entry_point, stage)?.as_slice(),
        )),
    }))
}
