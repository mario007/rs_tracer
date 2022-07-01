use std::{error::Error, fs, collections::HashMap};
use crate::{scene::{SceneData, RenderingAlgorithm}, pixel_buffer::{TMOType, Color}, vec::f32x3, materials::MatteMaterial, shapes::{Sphere, Shape}, lights::PointLight};
use serde_json::Value;


pub fn parse_json_file(filename: &str) -> Result<SceneData, Box<dyn Error>> {
    let contents = fs::read_to_string(filename)?;
    let val:Value = serde_json::from_str(&contents)?;
    let mut scene_data = SceneData::default();
    let global = &val["global"];
    if !global.is_null() {
        parse_global(&mut scene_data, global)?;
    }
    let camera = &val["camera"];
    if !camera.is_null() {
        parse_camera(&mut scene_data, camera)?;
    }
    let mut mtrs: HashMap<String, usize> = HashMap::new();
    let materials = &val["materials"];
    if !materials.is_null() {
        let map = parse_materials(&mut scene_data, materials)?;
        mtrs.extend(map)
    }
    let shapes = &val["shapes"];
    if !shapes.is_null() {
        parse_shapes(&mut scene_data, shapes, &mtrs)?;
    }
    let lights = &val["lights"];
    if !lights.is_null() {
        parse_lights(&mut scene_data, lights)?;
    }

    Ok(scene_data)
}

fn parse_lights(scene_data: &mut SceneData, section: &Value) -> Result<(), Box<dyn Error>> {
    let lights = match section.as_array() {
        Some(lights) => lights,
        None => return Err("List of lights expected!".into())
    };
    for light in lights.iter() {
        parse_light(scene_data, light)?;
    }
    Ok(())
}

fn parse_light(scene_data: &mut SceneData, section: &Value) -> Result<(), Box<dyn Error>> {
    let typ = parse_string(&section["type"], "light->type")?;
    match typ.as_str() {
        "point" => parse_point_light(scene_data, section)?,
        _ => return Err(format!("Unknown light type {}", typ).into())
    };
    Ok(())
}

fn parse_point_light(scene_data: &mut SceneData, section: &Value) -> Result<(), Box<dyn Error>> {
    let intensity = parse_color(&section["intensity"], "light->intensity")?;
    let position = parse_f32x3(&section["position"], "light->position")?;
    let light = PointLight::new(intensity, position);
    scene_data.add_light(Box::new(light));
    Ok(())
}

fn parse_shapes(scene_data: &mut SceneData, section: &Value, map: &HashMap<String, usize>) -> Result<(), Box<dyn Error>> {
    let shapes = match section.as_array() {
        Some(shapes) => shapes,
        None => return Err("List of shapes expected!".into())
    };
    for shape in shapes.iter() {
        parse_shape(scene_data, shape, map)?;
    }
    Ok(())
}

fn parse_shape(scene_data: &mut SceneData, section: &Value, map: &HashMap<String, usize>) -> Result<(), Box<dyn Error>> {
    let typ = parse_string(&section["type"], "shape->type")?;
    match typ.as_str() {
        "sphere" => parse_sphere_shape(scene_data, section, map)?,
        _ => return Err(format!("Unknown shape type {}", typ).into())
    };
    Ok(())
}

fn parse_sphere_shape(scene_data: &mut SceneData, section: &Value, map: &HashMap<String, usize>) -> Result<(), Box<dyn Error>> {
    let mat_name = parse_string(&section["material"], "shape:material:name")?;
    let material_id = match map.get(&mat_name) {
        Some(material_id) => material_id,
        None => return Err(format!("Material {} doesn't exist", mat_name).into())
    };
    let postion = parse_f32x3(&section["position"], "shape->position")?;
    let radius = parse_f32(&section["radius"], "shape->radius")?;
    let sphere = Sphere::new(postion, radius);
    scene_data.add_shape(Shape::new(Box::new(sphere), *material_id));
    Ok(())
}


fn parse_materials(scene_data: &mut SceneData, section: &Value) -> Result<HashMap<String, usize>, Box<dyn Error>> {
    let mtrs = match section.as_array() {
        Some(mtrs) => mtrs,
        None => return Err("List of materials expected.".into())
    };
    let mut map = HashMap::new();
    for mat in mtrs.iter() {
        let name = parse_string(&mat["name"], "material->name")?;
        let material_id = parse_material(scene_data, mat, &name)?;
        if map.contains_key(&name) {
            return Err(format!("Material {} allread exist!", name).into())
        }
        map.insert(name, material_id);
    }
    Ok(map)
}

fn parse_material(scene_data: &mut SceneData, section: &Value, name: &str) -> Result<usize, Box<dyn Error>> {
    let typ = parse_string(&section["type"], "material->type")?;
    let material_id = match typ.as_str() {
        "matte" => parse_matte_material(scene_data, section, name)?,
        _ => return Err(format!("Unknown material type {}", typ).into())
    };
    Ok(material_id)
}

fn parse_matte_material(scene_data: &mut SceneData, section: &Value, name: &str) -> Result<usize, Box<dyn Error>> {
    let color = parse_color(&section["diffuse"], &format!("material:{}:diffuse", name))?;
    let material_id = scene_data.add_material(Box::new(MatteMaterial::new(color)));
    Ok(material_id)
}

fn parse_global(scene_data: &mut SceneData, section: &Value) -> Result<(), Box<dyn Error>> {
    if !section["resolution"].is_null() {
        let (width, height) = parse_resolution(&section["resolution"])?;
        scene_data.set_image_size(width, height);
    }
    if !section["spp"].is_null() {
        let spp = parse_usize(&section["spp"], "spp")?;
        scene_data.set_samples_per_pixel(spp);
    }
    if !section["rendering"].is_null() {
        let alg = parse_string(&section["rendering"], "rendering")?;
        match alg.as_str() {
            "ambient" => scene_data.set_rendering_algorithm(RenderingAlgorithm::AmbientOcclusion),
            "direct_lighting" => scene_data.set_rendering_algorithm(RenderingAlgorithm::DirectLighting),
            _ => return Err(format!("Unknown rendering algorithm: {}", alg).into())
        }
    }
    if !section["tonemap"].is_null() {
        let tmo = parse_string(&section["tonemap"], "tonemap")?;
        match tmo.as_str() {
            "linear" => scene_data.set_tmo_type(TMOType::Linear),
            "gamma" => scene_data.set_tmo_type(TMOType::Gamma),
            "reinhard" => scene_data.set_tmo_type(TMOType::Reinhard),
            _ => return Err(format!("Unknown tone mapping operator: {}", tmo).into())
        }
    }
    if !section["output"].is_null() {
        let output = parse_string(&section["output"], "output")?;
        scene_data.set_output_file(output);
    }
    if !section["nthreads"].is_null() {
        let nthreads = parse_usize(&section["nthreads"], "nthreads")?;
        scene_data.set_nthreads(nthreads);
    }

    Ok(())
}

fn parse_camera(scene_data: &mut SceneData, section: &Value) -> Result<(), Box<dyn Error>> {
    if !section["eye"].is_null() {
        let eye = parse_f32x3(&section["eye"], "camera->eye")?;
        scene_data.set_camera_pos(eye);
    }
    if !section["lookat"].is_null() {
        let look_at = parse_f32x3(&section["lookat"], "camera->lookat")?;
        scene_data.set_camera_look_at(look_at);
    }
    if !section["hfov"].is_null() {
        let hfov = parse_f32(&section["hfov"], "camera->hfov")?;
        scene_data.set_camera_horizontal_fov(hfov);
    }
    if !section["vp_distance"].is_null() {
        let dist = parse_f32(&section["vp_distance"], "camera->vp_distance")?;
        scene_data.set_camera_view_plane_distance(dist);
    }
    Ok(())
}

fn parse_resolution(section: &Value) -> Result<(usize, usize), Box<dyn Error>> {
    let width = parse_usize(&section[0], "resolution")?;
    let height = parse_usize(&section[1], "resolution")?;
    Ok((width, height))
}

fn parse_f32x3(section: &Value, field_name: &str) -> Result<f32x3, Box<dyn Error>> {
    let val1 = parse_f32(&section[0], field_name)?;
    let val2 = parse_f32(&section[1], field_name)?;
    let val3 = parse_f32(&section[2], field_name)?;
    if !&section[3].is_null() {
        return Err(format!("Field: {} - Exactly 3 values expected!", field_name).into())
    }
    Ok(f32x3(val1, val2, val3))
}

fn parse_color(section: &Value, field_name: &str) -> Result<Color, Box<dyn Error>> {
    let red = parse_f32(&section[0], field_name)?;
    let green = parse_f32(&section[1], field_name)?;
    let blue = parse_f32(&section[2], field_name)?;
    if !&section[3].is_null() {
        return Err(format!("Field: {} - Exactly 3 values expected!", field_name).into())
    }
    Ok(Color{red, green, blue})

}

fn parse_usize(section: &Value, field_name: &str) -> Result<usize, Box<dyn Error>> {
    let val = match section.as_u64() {
        Some(val) => val as usize,
        None => return Err(format!("Field: {}", field_name).into())
    };
    Ok(val)
}

fn parse_string(section: &Value, field_name: &str) -> Result<String, Box<dyn Error>> {
    let val = match section.as_str() {
        Some(val) => val,
        None => return Err(format!("Field: {}", field_name).into())
    };
    Ok(val.to_string())
}

fn parse_f32(section: &Value, field_name: &str) -> Result<f32, Box<dyn Error>> {
    let val = match section.as_f64() {
        Some(val) => val as f32,
        None => return Err(format!("Field: {}", field_name).into())
    };
    Ok(val)
}
