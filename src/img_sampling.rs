use crate::pcg::PCGRng;

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub startx: usize,
    pub starty: usize,
    pub endx: usize,
    pub endy: usize
}

#[derive(Debug, Clone, Copy)]
pub struct ImageSample {
    pub x: usize,
    pub y: usize,
    pub xp: f32,
    pub yp: f32
}

pub struct ImageSampler {
    tile: Tile,
    curx: usize,
    cury: usize
}

impl ImageSampler {
    pub fn new(tile: Tile) -> Self {
        ImageSampler { tile, curx: tile.startx, cury: tile.starty }
    }

    pub fn next(&mut self, rng: &mut PCGRng) -> Option<ImageSample> {
        if self.cury == self.tile.endy {
            return None
        }

        let img_sample = ImageSample {
            x: self.curx,
            y: self.cury,
            xp: rng.rnd_f32(),
            yp: rng.rnd_f32()
        };

        self.curx += 1;
        if self.curx == self.tile.endx {
            self.curx = self.tile.startx;
            self.cury += 1;
        }
        Some(img_sample)
    }
}
