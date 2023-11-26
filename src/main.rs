mod interface;
mod cube_scene_renderer;

fn main() {
    interface::scene_render::run::<cube_scene_renderer::CubeSceneRenderer>("cube");
}