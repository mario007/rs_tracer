
pub struct PCGRng {
    state: u64,
    inc: u64
}

impl PCGRng {
    pub fn new(state: u64, inc: u64) -> PCGRng {
        PCGRng { state, inc }
    }

    pub fn rnd_u32(&mut self) -> u32 {
        let oldstate = self.state;
        // Advance internal state
        self.state = oldstate.wrapping_mul(6364136223846793005u64).wrapping_add(self.inc | 1);
        // Calculate output function (XSH RR), uses old state for max ILP
        let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;
        (xorshifted >> rot) | (xorshifted << ((-(rot as i32) as u32) & 31))
    }

    pub fn rnd_f32(&mut self) -> f32 {
        let val = f32::from_bits(0x33800000); // 0x1p-24f, 2^-24
        (self.rnd_u32() >> 8) as f32 * val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcg_test () {
        let mut rng = PCGRng::new(0xf123456789012345, 0);
        for _i in 0..20  {
            println!("rnd: {}", rng.rnd_f32());
        }
    }
}
