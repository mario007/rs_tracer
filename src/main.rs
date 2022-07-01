
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
pub mod onb;
pub mod materials;
pub mod lights;
pub mod json;

use std::{time::{Instant, Duration}, env};

use lights::PointLight;
use materials::MatteMaterial;
use pixel_buffer::Color;
use renderer::Renderer;
use scene::SceneData;
use shapes::{Sphere, Triangle, Shape};
use vec::f32x3;
use json::parse_json_file;

use crate::pixel_buffer::TMOType;

fn build_test_scene_1() -> SceneData {
    let mut scene_data = SceneData::default();

    let red = MatteMaterial::new(Color{red: 1.0, green: 0.0, blue: 0.0});
    let red_id = scene_data.add_material(Box::new(red));

    let sphere = Sphere::new(f32x3(0.0, 0.0, 2.0), 1.0);
    scene_data.add_shape(Shape::new(Box::new(sphere), red_id));

    let intensity = Color{red: 2.0, green: 2.0, blue: 2.0};
    let position = f32x3(0.0, 0.0, 0.0);
    let light = PointLight::new(intensity, position);
    scene_data.add_light(Box::new(light));

    scene_data.set_camera_pos(f32x3(0.0, 0.0, 0.0));
    scene_data.set_camera_look_at(f32x3(0.0, 0.0, 2.0));
    scene_data.set_camera_horizontal_fov(78.0);
    scene_data
}

fn build_cornell_1() -> SceneData {
    let mut scene_data = SceneData::default();
    let red = MatteMaterial::new(Color { red: 0.63, green: 0.06, blue: 0.04 });
    let red_id = scene_data.add_material(Box::new(red));
    let green = MatteMaterial::new(Color { red: 0.15, green: 0.48, blue: 0.09 });
    let green_id = scene_data.add_material(Box::new(green));
    let white = MatteMaterial::new(Color { red: 0.76, green: 0.75, blue: 0.5 });
    let white_id = scene_data.add_material(Box::new(white));

    let floor_1 = Triangle::new(f32x3(0.556, 0.0, 0.0), f32x3(0.0, 0.0, 0.0), f32x3(0.0, 0.0, 0.5592));
    let floor_2 = Triangle::new(f32x3(0.556, 0.0, 0.0), f32x3(0.0, 0.0, 0.5592), f32x3(0.556, 0.0, 0.5592));
    scene_data.add_shape(Shape::new(Box::new(floor_1), white_id));
    scene_data.add_shape(Shape::new(Box::new(floor_2), white_id));

    let ceiling_1 = Triangle::new(f32x3(0.556, 0.5488, 0.0), f32x3(0.566, 0.5488, 0.5592), f32x3(0.0, 0.5488, 0.5592));
    let ceiling_2 = Triangle::new(f32x3(0.556, 0.5488, 0.0), f32x3(0.0, 0.5488, 0.5592), f32x3(0.0, 0.5488, 0.0));
    scene_data.add_shape(Shape::new(Box::new(ceiling_1), white_id));
    scene_data.add_shape(Shape::new(Box::new(ceiling_2), white_id));

    let back_1 = Triangle::new(f32x3(0.556, 0.0, 0.5592), f32x3(0.0, 0.0, 0.5592), f32x3(0.0, 0.5488, 0.5592));
    let back_2 = Triangle::new(f32x3(0.556, 0.0, 0.5592), f32x3(0.0, 0.5488, 0.5592), f32x3(0.556, 0.5488, 0.5592));
    scene_data.add_shape(Shape::new(Box::new(back_1), white_id));
    scene_data.add_shape(Shape::new(Box::new(back_2), white_id));

    let left_1 = Triangle::new(f32x3(0.556, 0.0, 0.0), f32x3(0.556, 0.0, 0.5592), f32x3(0.556, 0.5488, 0.5592));
    let left_2 = Triangle::new(f32x3(0.556, 0.0, 0.0), f32x3(0.556, 0.5488, 0.5592), f32x3(0.556, 0.5488, 0.0));
    scene_data.add_shape(Shape::new(Box::new(left_1), red_id));
    scene_data.add_shape(Shape::new(Box::new(left_2), red_id));

    let right_1 = Triangle::new(f32x3(0.0, 0.0, 0.5592), f32x3(0.0, 0.0, 0.0), f32x3(0.0, 0.5488, 0.0));
    let right_2 = Triangle::new(f32x3(0.0, 0.0, 0.5592), f32x3(0.0, 0.5488, 0.0), f32x3(0.0, 0.5488, 0.5592));
    scene_data.add_shape(Shape::new(Box::new(right_1), green_id));
    scene_data.add_shape(Shape::new(Box::new(right_2), green_id));
    
    let intensity = Color{red: 1.0, green: 1.0, blue: 1.0};
    let position = f32x3(0.278, 0.273, 0.278);
    let light = PointLight::new(intensity*0.02, position);
    scene_data.add_light(Box::new(light));

    scene_data.set_camera_pos(f32x3(0.278, 0.273, -0.8));
    scene_data.set_camera_look_at(f32x3(0.278, 0.273, -0.799));
    scene_data.set_camera_horizontal_fov(50.0);

    scene_data
}

fn main() {
    //let scene_data = build_test_scene_1();
    //let scene_data = build_cornell_1();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Missing scene description file");
        return;
    }

    let scene_data = match parse_json_file(&args[1]) {
        Err(err) => {
            eprintln!("Problem parsing input file {}: {}", &args[1], err);
            return;
        },
        Ok(scene) => scene
    };

    let mut ren = Renderer::new(scene_data);
    let start_time = Instant::now();
    loop {
        let is_finished = ren.render(Duration::from_millis(100));
        if is_finished { break; }
    }

    let render_time = Instant::now() - start_time;
    println!("Rendering time {}", render_time.as_millis());
    let _result = ren.save();
}
