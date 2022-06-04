use crate::{ray::Ray, scene::SceneData, pcg::PCGRng, pixel_buffer::Color};


pub fn render_sample(ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {

    let result = scene_data.intersect(ray, 1e30);
    if result.is_some() {
        return Color {red: 0.0, green: 0.0, blue: 1.0};
    }
    Color {red: 1.0, green: 0.0, blue: 0.0}
}

