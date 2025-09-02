use crate::math::Vec2;

const BULLET_SPEED: f32 = 10.0;

#[turbo::serialize]
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub ttl: u32,
}

impl Bullet {
    pub fn new(pos: Vec2, dir: f32) -> Self {
        Self {
            pos,
            vel: Vec2::new(dir.cos() * BULLET_SPEED, dir.sin() * BULLET_SPEED),
            ttl: 180,
        }
    }
    pub fn update(&mut self) -> bool {
        self.pos = self.pos.add(self.vel.clone());
        self.ttl = self.ttl.saturating_sub(1);
        self.ttl > 0
    }
}
