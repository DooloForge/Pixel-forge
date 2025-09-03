use crate::math::Vec3;
// use crate::constants::PARTICLE_LIFETIME_TICKS;
use crate::constants::GRAVITY;

#[turbo::serialize]
pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub life: u32,
}

impl Particle {
    pub fn new(pos: Vec3, vel: Vec3) -> Self {
        Self { pos, vel, life: 30 }
    }
    pub fn update(&mut self) -> bool {
        self.vel.y += GRAVITY * 0.2;
        self.vel = self.vel.mul(0.97);
        self.pos = self.pos.add(self.vel.clone());
        self.life = self.life.saturating_sub(1);
        self.life > 0
    }
}
