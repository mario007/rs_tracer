use std::default::Default;

extern crate num_cpus;

pub struct SceneData {
    width: usize,
    height: usize,
    ntheads: usize
}

impl SceneData {
    pub fn image_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get_nthreads(&self) -> usize {
        self.ntheads
    }
}

impl Default for SceneData {
    fn default() -> Self {
        //Self { width: 200, height: 200, ntheads: num_cpus::get() }
        Self { width: 1024, height: 768, ntheads: 16 }
    }
}
