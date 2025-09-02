#[turbo::serialize]
pub enum TerrainMaterial {
    Water,
    Sand,
    Stone,
    Leaves,
    Iron,
}

#[turbo::serialize]
pub struct TerrainCell {
    pub material: TerrainMaterial,
}

impl TerrainCell {
    pub fn new(material: TerrainMaterial) -> Self { Self { material } }
}

#[turbo::serialize]
pub struct TerrainChunk {
    pub x: i32,
    pub y: i32,
    pub cells: Vec<TerrainCell>,
}

impl TerrainChunk {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y, cells: vec![TerrainCell::new(TerrainMaterial::Water); 32*32] }
    }
}

#[turbo::serialize]
pub enum BlockType {
    Water,
    Sand,
    Rock,
    Coral,
    Kelp,
    TreasureChest,
    IronDeposit,
    PearlBed,
}

#[turbo::serialize]
pub struct Block {
    pub block_type: BlockType,
    pub durability: f32,
}

impl Block {
    pub fn new(block_type: BlockType, durability: f32) -> Self { Self { block_type, durability } }
}


