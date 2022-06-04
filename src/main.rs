
pub mod pixel_buffer;
pub mod traits;
pub mod scene;
pub mod renderer;
pub mod pcg;
pub mod vec;
pub mod ray;
pub mod camera;
pub mod img_sampling;
pub mod shapes;
pub mod render;

use std::time::{Instant, Duration};

use renderer::Renderer;
use scene::SceneData;
use shapes::Sphere;
use vec::f32x3;

use crate::pixel_buffer::TMOType;

fn build_test_scene_1() -> SceneData {
    let mut scene_data = SceneData::default();
    let sphere = Sphere::new(f32x3(0.0, 0.0, 3.0), 1.0);
    scene_data.add_shape(Box::new(sphere));

    scene_data.set_camera_pos(f32x3(0.0, 0.0, 0.0));
    scene_data.set_camera_look_at(f32x3(0.0, 0.0, 3.0));
    scene_data.set_camera_horizontal_fov(60.0);
    scene_data
}

fn main() {
    let scene_data = build_test_scene_1();
    let mut ren = Renderer::new(scene_data);
    let start_time = Instant::now();
    loop {
        let is_finished = ren.render(Duration::from_millis(100));
        if is_finished { break; }
    }

    let render_time = Instant::now() - start_time;
    println!("Rendering time {}", render_time.as_millis());
    ren.save("test.jpg", &TMOType::Linear);
}

