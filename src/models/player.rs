use crate::math::Vec2 as V2;

#[turbo::serialize]
pub struct Player {
    pub pos: V2,
    pub vel: V2,
    pub grounded: bool,
    pub facing: f32,
}

impl Player {
    pub fn new(pos: V2) -> Self { Self { pos, vel: V2::zero(), grounded: false, facing: 0.0 } }
}
