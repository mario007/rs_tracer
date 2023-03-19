use std::error::Error;
use std::path::Path;
use std::sync::{Arc, mpsc};
use std::thread;
use std::mem::drop;
use std::time::{Duration, Instant};

use crate::pcg::PCGRng;
use crate::pixel_buffer::{Color, PixelBuffer, PixelData, TMOType};
use crate::ray::Ray;
use crate::scene::{SceneData, RenderingAlgorithm};
use crate::img_sampling::{Tile, ImageSampler};
use crate::render::{ambient_occlusion, direct_lighting, path_tracer};


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
        let color = match scene_data.rendering_algorithm {
            RenderingAlgorithm::AmbientOcclusion => ambient_occlusion(&ray, scene_data, rng),
            RenderingAlgorithm::DirectLighting => direct_lighting(&ray, scene_data, rng),
            RenderingAlgorithm::PathTracer => path_tracer(&ray, scene_data, rng)
        };
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

    pub fn new(mut sc_data: SceneData) -> Renderer {
        let (width, height) = sc_data.image_size();
        sc_data.prepare();
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
                    for tile in tiles.iter().skip(thread_id).step_by(n_actual_threads) {
                        for _n in 0..sc_data.get_samples_per_pixel() {
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

    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        self.pixel_buffer.save(self.scene_data.get_output_file(), self.scene_data.get_tmo_type())
    }

    pub fn to_rgb_vector(&self) -> Vec<u32> {
        self.pixel_buffer.to_rgb_vector(self.scene_data.get_tmo_type())
    }

}

pub struct TileData2 {
    samples: Vec<PixelSample>,
    thread_id: usize
}

enum Job {
    Tile(Tile),
    Close
}

pub struct Renderer2 {
    scene_data: Arc<SceneData>,

    renderig_in_progress: bool,
    tiles: Vec<Tile>,
    threads: Vec<thread::JoinHandle<()>>,
    receiver: mpsc::Receiver<TileData2>,
    pixel_buffer: PixelBuffer,
    n_tiles_processed: usize,
    senders: Vec<mpsc::Sender<Job>>,

}

impl Renderer2 {

    pub fn new(mut sc_data: SceneData) -> Renderer2 {
        let (width, height) = sc_data.image_size();
        sc_data.prepare();
        // ah, when reciever is in Option than he borrow self and I can't use write_samples method
        let (_tx, reciver): (mpsc::Sender<TileData2>, mpsc::Receiver<TileData2>) = mpsc::channel();
        Renderer2 {
            scene_data: Arc::new(sc_data),
            renderig_in_progress: false,
            tiles: create_tiles(width, height, 16),
            threads: Vec::new(),
            receiver: reciver,
            pixel_buffer: PixelBuffer::new(width, height),
            n_tiles_processed: 0,
            senders: Vec::new()
        }
    }

    fn create_threads(&mut self) {
        let n_actual_threads = self.tiles.len().min(self.scene_data.get_nthreads());
        let (tx, reciver): (mpsc::Sender<TileData2>, mpsc::Receiver<TileData2>) = mpsc::channel();

        self.receiver = reciver;

        for thread_id in 0..n_actual_threads {
            let sender = tx.clone();
            let sc_data = Arc::clone(&self.scene_data);
            let (tx_job, rec_job): (mpsc::Sender<Job>, mpsc::Receiver<Job>) = mpsc::channel();

            let handle = thread::spawn (move || {
                let mut rng = PCGRng::new(0xf123456789012345, 1000 * thread_id as u64);
                loop {
                    let job = rec_job.recv().unwrap();
                    let tile = match job {
                        Job::Tile(tile) => tile,
                        Job::Close => break
                    };
                    let samples = render_tile(&tile, &sc_data, &mut rng);
                    sender.send(TileData2 {samples, thread_id}).unwrap();
                }
            });
            self.threads.push(handle);
            self.senders.push(tx_job);
        }
        drop(tx);
    }

    fn shutdown_threads(&mut self) {
        for sender in self.senders.iter() {
            let _result = sender.send(Job::Close);
        }

        while let Some(cur_thread) = self.threads.pop() {
            cur_thread.join().unwrap();
        }
        self.senders.clear();
    }

    pub fn render(&mut self, timeout: Duration) -> bool {
        if self.n_tiles_processed == self.tiles.len() * self.scene_data.get_samples_per_pixel() {
            return true;
        }

        if self.renderig_in_progress == false {
            self.create_threads();
            self.renderig_in_progress = true;
        }

        // state variables
        let start_time = Instant::now();
        let mut current_tile = self.n_tiles_processed % self.tiles.len();
        let mut current_spp = self.n_tiles_processed / self.tiles.len();
        let mut thread_id = 0;
        let mut n_tiles_in_progress = 0;

        loop {
            let _res = self.senders[thread_id].send(Job::Tile(self.tiles[current_tile]));
            current_tile += 1;
            thread_id += 1;
            n_tiles_in_progress += 1;
            self.n_tiles_processed += 1;
            if thread_id == self.threads.len() {
                break;
            }
            if current_tile == self.tiles.len() {
                current_tile = 0;
                current_spp += 1;
                if current_spp == self.scene_data.get_samples_per_pixel() {
                    break;
                }
            }
        }

        loop {
            let data = self.receiver.recv().unwrap();
            n_tiles_in_progress -= 1;
            let render_time = Instant::now() - start_time;
            if current_spp == self.scene_data.get_samples_per_pixel() || render_time > timeout {
                self.write_samples(&data);
                break;
            }

            if current_tile == self.tiles.len() {
                current_tile = 0;
                current_spp += 1;
                if current_spp == self.scene_data.get_samples_per_pixel() {
                    self.write_samples(&data);
                    break;
                }
            }

            let _res = self.senders[data.thread_id].send(Job::Tile(self.tiles[current_tile]));
            current_tile += 1;
            n_tiles_in_progress += 1;
            self.n_tiles_processed += 1;

            self.write_samples(&data);
        }

        for _i in 0..n_tiles_in_progress {
            let data = self.receiver.recv().unwrap();
            self.write_samples(&data);
        }
        if self.n_tiles_processed == self.tiles.len() * self.scene_data.get_samples_per_pixel() {
            self.renderig_in_progress = false;
            self.shutdown_threads();
            return true;
        }
        return false;
    }

    fn write_samples(&mut self, data: &TileData2) {
        let (_width, height) = self.scene_data.image_size();
        for sample in data.samples.iter() {
            let pdata = PixelData{color: sample.color, weight: 1.0};
            self.pixel_buffer.add_pixel(sample.x, height - sample.y - 1, &pdata);
        }
    }
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        self.pixel_buffer.save(self.scene_data.get_output_file(), self.scene_data.get_tmo_type())
    }

    pub fn to_rgb_vector(&self) -> Vec<u32> {
        self.pixel_buffer.to_rgb_vector(self.scene_data.get_tmo_type())
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
