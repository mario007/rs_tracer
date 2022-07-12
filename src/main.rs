
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

use renderer::Renderer;
use json::parse_json_file;


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
