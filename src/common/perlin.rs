use noise::{Clamp, Fbm, MultiFractal, NoiseFn};

#[allow(dead_code)]
pub struct Perlin {
    nz: Clamp<f64, Fbm<noise::Perlin>, 2>,
    seed: u32,
}

#[allow(dead_code)]
impl Perlin {
    pub fn new(seed: u32, frequency: f64, octaves: usize, lacunarity: f64) -> Self {
        let fbm = noise::Fbm::<noise::Perlin>::new(seed)
            .set_frequency(frequency)
            .set_octaves(octaves)
            .set_lacunarity(lacunarity);

        let nz = Clamp::new(fbm).set_bounds(0., 1.);

        Self {
            nz,
            seed,
        }
    }

    // get 2d noise value, clamped to [0, 1)
    pub fn get(&mut self, x: f32, y: f32) -> f64 {
        self.nz.get([x as f64, y as f64])
    }
}