use crate::math::Vec3 as V3;

#[turbo::serialize]
pub enum RaftTileType {
    Wood,
}

#[turbo::serialize]
pub struct Raft {
    pub center: V3,
    pub size_tiles: (i32, i32),
}

impl Raft {
    pub fn new(center: V3) -> Self {
        Self { center, size_tiles: (4, 3) }
    }

    pub fn is_on_raft(&self, pos: &V3) -> bool {
        let half_w = self.size_tiles.0 as f32 * 8.0;
        let half_h = self.size_tiles.1 as f32 * 8.0;
        pos.x >= self.center.x - half_w && pos.x <= self.center.x + half_w &&
        pos.y >= self.center.y - half_h && pos.y <= self.center.y + half_h
    }
}


