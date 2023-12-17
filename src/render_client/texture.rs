use anyhow::Result;

pub struct Texture {
    texture: wgpu::Texture,
    views: Vec<wgpu::TextureView>,
}

impl Texture {
    pub fn new(device: &wgpu::Device, desc : wgpu::TextureDescriptor) -> Self {
        let texture = device.create_texture(&desc);
        Self {
            texture,
            views: vec![]
        }
    }

    pub fn create_view(&mut self, desc: wgpu::TextureViewDescriptor) -> Result<usize> {
        let texture_view = self.texture.create_view(&desc);
        self.views.push(texture_view);
        
        Ok(self.views.len() - 1)
    }

    pub fn get_view(&self, view_index : usize) -> Option<&wgpu::TextureView> {
        Some(&self.views[view_index])
    }
}