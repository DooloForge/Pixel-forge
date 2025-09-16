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
    // Raft building materials
    Wood,
    Plastic,
    Rope,
    Metal,
    Nail,
    Cloth,
    Barrel,
    
    // Food items
    Coconut,
    Fish,
    Seaweed,
    
    // Special items
    Treasure,
    Bottle,
}

impl FloatingItemType {
    pub fn color(&self) -> u32 {
        match self {
            // Raft building materials
            FloatingItemType::Wood => 0x8B4513FF,      // Brown wood
            FloatingItemType::Plastic => 0x1E90FFFF,   // Blue plastic
            FloatingItemType::Rope => 0xC2B280FF,      // Tan rope
            FloatingItemType::Metal => 0xB0B0B0FF,     // Silver metal
            FloatingItemType::Nail => 0x696969FF,      // Dark gray nail
            FloatingItemType::Cloth => 0xFFB6C1FF,     // Pink cloth
            FloatingItemType::Barrel => 0x8B4513FF,    // Brown barrel
            
            // Food items
            FloatingItemType::Coconut => 0x654321FF,   // Brown coconut
            FloatingItemType::Fish => 0x87CEFAFF,      // Light blue fish
            FloatingItemType::Seaweed => 0x228B22FF,   // Green seaweed
            
            // Special items
            FloatingItemType::Treasure => 0xFFD700FF,  // Gold treasure
            FloatingItemType::Bottle => 0x87CEEBFF,    // Sky blue bottle
        }
    }
    
    pub fn size(&self) -> f32 {
        match self {
            // Raft building materials - larger items
            FloatingItemType::Wood => 12.0,
            FloatingItemType::Plastic => 10.0,
            FloatingItemType::Rope => 8.0,
            FloatingItemType::Metal => 9.0,
            FloatingItemType::Nail => 4.0,
            FloatingItemType::Cloth => 8.0,
            FloatingItemType::Barrel => 16.0,
            
            // Food items - medium size
            FloatingItemType::Coconut => 6.0,
            FloatingItemType::Fish => 7.0,
            FloatingItemType::Seaweed => 5.0,
            
            // Special items - various sizes
            FloatingItemType::Treasure => 8.0,
            FloatingItemType::Bottle => 6.0,
        }
    }
    
    pub fn rarity(&self) -> f32 {
        match self {
            // Common raft materials
            FloatingItemType::Wood => 0.3,
            FloatingItemType::Plastic => 0.25,
            FloatingItemType::Rope => 0.2,
            FloatingItemType::Metal => 0.15,
            FloatingItemType::Nail => 0.1,
            FloatingItemType::Cloth => 0.1,
            FloatingItemType::Barrel => 0.05,
            
            // Food items
            FloatingItemType::Coconut => 0.2,
            FloatingItemType::Fish => 0.15,
            FloatingItemType::Seaweed => 0.1,
            
            // Rare special items
            FloatingItemType::Treasure => 0.02,
            FloatingItemType::Bottle => 0.05,
        }
    }
}


