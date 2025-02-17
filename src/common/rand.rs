use bevy::ecs::system::Resource;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Resource)]
pub struct Rand {
    r: SmallRng,
}

#[allow(dead_code)]
impl Rand {
    pub fn seed(seed: u64) -> Self {
        Self {
            r: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn new() -> Self {
        Self {
            r: SmallRng::from_entropy(),
        }
    }

    pub fn pick<T>(&mut self, v: &[T]) -> T
    where
        T: Copy,
    {
        let idx = self.pick_idx(v);
        v[idx]
    }

    pub fn pick_idx<T>(&mut self, v: &[T]) -> usize {
        self.range_n(0, v.len() as i32) as usize
    }

    pub fn range_n(&mut self, min: i32, max: i32) -> i32 {
        (self.random() * (max as f32 - min as f32)) as i32 + min
    }

    pub fn random(&mut self) -> f32 {
        self.r.r#gen()
    }

    pub fn bool(&mut self, chance: f32) -> bool {
        self.random() < chance
    }
}
