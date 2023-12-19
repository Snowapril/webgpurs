use ahash::{AHasher, RandomState};
use std::collections::HashMap;

pub struct BlackBoard {
    pub textures: HashMap<&'static str, wgpu::Texture, RandomState>,
}
