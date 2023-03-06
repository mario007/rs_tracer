
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
pub mod bbox;
pub mod bvh;

use std::{time::{Instant, Duration}, env};

use minifb::{Window, WindowOptions, Key};
use renderer::Renderer;
use json::parse_json_file;
use scene::SceneData;

fn run_in_console(scene_data: SceneData) {
    let prepare_time = Instant::now();
    let mut ren = Renderer::new(scene_data);
    let prepare_time = Instant::now() - prepare_time;
    println!("Prepare time {}", prepare_time.as_millis());
    let start_time = Instant::now();
    loop {
        let is_finished = ren.render(Duration::from_millis(500));
        if is_finished { break; }
    }

    let render_time = Instant::now() - start_time;
    println!("Rendering time {}", render_time.as_millis());
    let _result = ren.save();
}

fn run_in_window(scene_data: SceneData) {
    let (width, height) = scene_data.image_size();
    let mut window = Window::new(
        "Tracer - ESC to exit",
        width,
        height,
        WindowOptions::default()
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut ren = Renderer::new(scene_data);
    let start_time = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let is_finished = ren.render(Duration::from_millis(500));
        let buffer = ren.to_rgb_vector();
        let _r = window.update_with_buffer(&buffer, width, height);
        if is_finished { break; }
    }
    let render_time = Instant::now() - start_time;
    println!("Rendering time {}", render_time.as_millis());
    let _result = ren.save();
}

fn main() {
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

    let mut console = false;
    for a in args {
        if a == "--console" {
            console = true;
        }
    }

    if console {
        run_in_console(scene_data);
    } else {
        run_in_window(scene_data);
    }
    
}
