use crate::math::Vec2;
// use crate::constants::PARTICLE_LIFETIME_TICKS;
use crate::constants::GRAVITY;

#[turbo::serialize]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: u32,
}

impl Particle {
    pub fn new(pos: Vec2, vel: Vec2) -> Self {
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
