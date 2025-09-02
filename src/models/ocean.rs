use crate::math::Vec2 as V2;

#[turbo::serialize]
pub struct Ocean {
    pub current_direction: V2,
    pub current_strength: f32,
}

impl Ocean {
    pub fn new() -> Self {
        Self {
            current_direction: V2::new(1.0, 0.0),
            current_strength: 0.25,
        }
    }
}

#[turbo::serialize]
#[derive(Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum FloatingItemType {
    Wood,
    Plastic,
    Rope,
    Metal,
    Coconut,
    Fish,
    Seaweed,
}

impl FloatingItemType {
    pub fn color(&self) -> u32 {
        match self {
            FloatingItemType::Wood => 0x8B4513FF,
            FloatingItemType::Plastic => 0x1E90FFFF,
            FloatingItemType::Rope => 0xC2B280FF,
            FloatingItemType::Metal => 0xB0B0B0FF,
            FloatingItemType::Coconut => 0x654321FF,
            FloatingItemType::Fish => 0x87CEFAFF,
            FloatingItemType::Seaweed => 0x228B22FF,
        }
    }
}


