use std::error::Error;
use std::path::Path;
use std::sync::{Arc, mpsc};
use std::thread;
use std::mem::drop;
use std::time::{Duration, Instant};

use crate::pcg::PCGRng;
use crate::pixel_buffer::{Color, PixelBuffer, PixelData, TMOType};
use crate::ray::Ray;
use crate::scene::SceneData;
use crate::img_sampling::{Tile, ImageSampler};
use crate::render::render_sample;


#[derive(Debug, Clone, Copy)]
pub struct PixelSample {
    x: usize,
    y: usize,
    color: Color
}


fn render_tile(tile: &Tile, scene_data: &SceneData, rng: &mut PCGRng) -> Vec<PixelSample> {
    let capacity = (tile.endx - tile.startx) * (tile.endy - tile.starty);
    let mut samples = Vec::with_capacity(capacity);

    let mut img_sampler = ImageSampler::new(*tile);
    while let Some(sample) = img_sampler.next(rng) {
        let ray = scene_data.generate_ray(sample.x, sample.y, sample.xp, sample.yp);
        let color = render_sample(&ray, scene_data, rng);
        samples.push(PixelSample { x: sample.x, y: sample.y, color });
    }

    samples
}

pub struct TileData {
    samples: Vec<PixelSample>
}

fn create_tiles(width: usize, height: usize, tile_size: usize) -> Vec<Tile> {
    let mut tiles = Vec::new();
    for y in (0..height).step_by(tile_size) {
        for x in (0..width).step_by(tile_size) {
            let endx = (x + tile_size).min(width);
            let endy = (y + tile_size).min(height);
            tiles.push(Tile{startx: x, starty: y, endx, endy});
        }
    }
    tiles
}

pub struct Renderer {
    scene_data: Arc<SceneData>,

    renderig_in_progress: bool,
    tiles: Arc<Vec<Tile>>,
    threads: Vec<thread::JoinHandle<()>>,
    receiver: Option<mpsc::Receiver<TileData>>,
    pixel_buffer: PixelBuffer,
    n_tiles_processed: usize
}

impl Renderer {

    pub fn new(sc_data: SceneData) -> Renderer {
        let (width, height) = sc_data.image_size();
        Renderer {
            scene_data: Arc::new(sc_data),
            renderig_in_progress: false,
            tiles: Arc::new(create_tiles(width, height, 16)),
            threads: Vec::new(),
            receiver: None,
            pixel_buffer: PixelBuffer::new(width, height),
            n_tiles_processed: 0
        }
    }

    fn create_threads(&mut self) {
        let n_actual_threads = self.tiles.len().min(self.scene_data.get_nthreads());
        let (tx, reciver): (mpsc::SyncSender<TileData>, mpsc::Receiver<TileData>) = mpsc::sync_channel(0);

        self.receiver = Some(reciver);

        for thread_id in 0..n_actual_threads {
            let sender = tx.clone();
            let tiles = Arc::clone(&self.tiles);
            let sc_data = Arc::clone(&self.scene_data);

            let handle = thread::spawn (move || {
                let mut rng = PCGRng::new(0xf123456789012345, 1000 * thread_id as u64);
                for _n in 0..sc_data.get_samples_per_pixel() {
                    for tile in tiles.iter().skip(thread_id).step_by(n_actual_threads) {
                        let samples = render_tile(tile, &sc_data, &mut rng);
                        sender.send(TileData {samples});
                    }
                }
            });
            self.threads.push(handle);
        }
        drop(tx);
    }

    pub fn render(&mut self, timeout: Duration) -> bool {
        if self.n_tiles_processed == self.tiles.len() * self.scene_data.get_samples_per_pixel() {
            return true;
        }

        if self.renderig_in_progress == false {
            self.create_threads();
            self.renderig_in_progress = true;
        }

        let start_time = Instant::now();

        if let Some(rx) = &self.receiver {
            loop {
                let data = rx.recv().unwrap();

                let (_width, height) = self.scene_data.image_size();
                for sample in data.samples {
                    let pdata = PixelData{color: sample.color, weight: 1.0};
                    self.pixel_buffer.add_pixel(sample.x, height - sample.y - 1, &pdata);
                }

                self.n_tiles_processed += 1;
                if self.n_tiles_processed == self.tiles.len() * self.scene_data.get_samples_per_pixel() {
                    self.renderig_in_progress = false;
                    while let Some(cur_thread) = self.threads.pop() {
                        cur_thread.join().unwrap();
                    }
                    return true;
                }

                let render_time = Instant::now() - start_time;
                if render_time > timeout {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, tmo_type: &TMOType) -> Result<(), Box<dyn Error>> {
        self.pixel_buffer.save(path, tmo_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel_buffer::TMOType;

    #[test]
    fn render_tiles () {
        let mut ren = Renderer::new(SceneData::default());
        let start_time = Instant::now();
        loop {
            let is_finished = ren.render(Duration::from_millis(10));
            if is_finished { break; }
        }
        let render_time = Instant::now() - start_time;
        println!("Render time {}", render_time.as_millis());
        ren.pixel_buffer.save("test.jpg", &TMOType::Linear);
    }
}
