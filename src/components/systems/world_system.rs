use crate::math::Vec2 as V2;
use crate::models::terrain::TerrainChunk;
use crate::constants::*;
use std::collections::HashMap;

/// Handles world generation, chunk management, and terrain updates
#[turbo::serialize]
pub struct WorldSystem {
    chunks: HashMap<(i32, i32), TerrainChunk>,
    chunk_size: usize,
    render_distance: i32,
    world_seed: u32,
}

impl WorldSystem {
    pub fn new(seed: u32) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size: CHUNK_SIZE,
            render_distance: RENDER_DISTANCE,
            world_seed: seed,
        }
    }
    
    /// Update world around player position
    pub fn update(&mut self, player_pos: &V2) {
        self.generate_chunks_around_player(player_pos);
        self.cleanup_distant_chunks(player_pos);
    }
    
    /// Generate chunks around player position
    fn generate_chunks_around_player(&mut self, player_pos: &V2) {
        let chunk_x = (player_pos.x / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        let chunk_y = (player_pos.y / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        
        // Generate chunks in render distance
        for dy in -self.render_distance..=self.render_distance {
            for dx in -self.render_distance..=self.render_distance {
                let cx = chunk_x + dx;
                let cy = chunk_y + dy;
                
                if !self.chunks.contains_key(&(cx, cy)) {
                    let chunk = self.generate_chunk(cx, cy);
                    self.chunks.insert((cx, cy), chunk);
                }
            }
        }
    }
    
    /// Generate a new chunk at specified coordinates
    fn generate_chunk(&self, x: i32, y: i32) -> TerrainChunk {
        let mut blocks = Vec::new();
        
        for row in 0..self.chunk_size {
            for col in 0..self.chunk_size {
                let world_x = x * self.chunk_size as i32 + col as i32;
                let world_y = y * self.chunk_size as i32 + row as i32;
                
                let block_type = self.generate_block_type(world_x, world_y);
                let block = self.create_block(block_type);
                blocks.push(block);
            }
        }
        
        TerrainChunk::new(x, y)
    }
    
    /// Generate block type based on world coordinates
    fn generate_block_type(&self, world_x: i32, world_y: i32) -> crate::models::terrain::BlockType {
        // Use deterministic noise based on world seed
        let noise_x = world_x as f32 * 0.1;
        let noise_y = world_y as f32 * 0.1;
        let terrain_height = (noise_x.sin() * 10.0 + noise_y.cos() * 8.0) as i32;
        
        // Ocean floor level
        let floor_level = 80 + terrain_height;
        
        // Use deterministic random based on world coordinates
        let seed = (world_x as u32).wrapping_mul(73856093) ^ (world_y as u32).wrapping_mul(19349663) ^ self.world_seed;
        let block_random = ((seed % 1000) as f32) / 1000.0;
        
        if world_y > floor_level + 10 {
            crate::models::terrain::BlockType::Sand
        } else if world_y > floor_level && block_random < 0.3 {
            if block_random < 0.2 {
                crate::models::terrain::BlockType::Coral
            } else {
                crate::models::terrain::BlockType::Kelp
            }
        } else if world_y > floor_level - 10 && block_random < 0.5 && block_random > 0.3 {
            crate::models::terrain::BlockType::Coral
        } else if world_y > floor_level - 20 && block_random < 0.7 && block_random > 0.5 {
            crate::models::terrain::BlockType::Rock
        } else if world_y > floor_level - 30 && block_random > 0.98 {
            if block_random > 0.99 {
                crate::models::terrain::BlockType::TreasureChest
            } else {
                crate::models::terrain::BlockType::IronDeposit
            }
        } else if world_y > floor_level + 5 {
            crate::models::terrain::BlockType::Sand
        } else {
            crate::models::terrain::BlockType::Water
        }
    }
    
    /// Create a block with the specified type
    fn create_block(&self, block_type: crate::models::terrain::BlockType) -> crate::models::terrain::Block {
        let durability = match block_type {
            crate::models::terrain::BlockType::Sand => SAND_HP,
            crate::models::terrain::BlockType::Rock => STONE_HP,
            crate::models::terrain::BlockType::Coral => 30.0,
            crate::models::terrain::BlockType::Kelp => 15.0,
            crate::models::terrain::BlockType::TreasureChest => 200.0,
            crate::models::terrain::BlockType::IronDeposit => IRON_HP,
            crate::models::terrain::BlockType::PearlBed => 150.0,
            _ => WATER_HP,
        };
        
        crate::models::terrain::Block::new(block_type, durability)
    }
    
    /// Clean up chunks that are too far from player
    fn cleanup_distant_chunks(&mut self, player_pos: &V2) {
        let max_distance = (self.render_distance + 2) as f32 * self.chunk_size as f32 * PIXEL_SIZE;
        let player_chunk_x = (player_pos.x / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        let player_chunk_y = (player_pos.y / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        
        self.chunks.retain(|&(cx, cy), _| {
            let chunk_distance = ((cx - player_chunk_x).pow(2) + (cy - player_chunk_y).pow(2)) as f32;
            chunk_distance <= max_distance * max_distance
        });
    }
    
    /// Get chunk at specified coordinates
    pub fn get_chunk(&self, x: i32, y: i32) -> Option<&TerrainChunk> {
        self.chunks.get(&(x, y))
    }
    
    /// Get all chunks within render distance
    pub fn get_visible_chunks(&self) -> Vec<&TerrainChunk> {
        self.chunks.values().collect()
    }
    
    /// Modify block at world coordinates
    pub fn modify_block(&mut self, world_x: i32, world_y: i32, new_type: crate::models::terrain::BlockType) -> bool {
        let chunk_x = (world_x as f32 / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        let chunk_y = (world_y as f32 / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        
        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_y)) {
            let local_x = (world_x - chunk_x * self.chunk_size as i32) as usize;
            let local_y = (world_y - chunk_y * self.chunk_size as i32) as usize;
            
            if local_x < self.chunk_size && local_y < self.chunk_size {
                let index = local_y * self.chunk_size + local_x;
                if index < chunk.cells.len() {
                    // Convert BlockType to TerrainMaterial (simplified mapping)
                    let material = match new_type {
                        crate::models::terrain::BlockType::Sand => crate::models::terrain::TerrainMaterial::Sand,
                        crate::models::terrain::BlockType::Rock => crate::models::terrain::TerrainMaterial::Stone,
                        crate::models::terrain::BlockType::Coral => crate::models::terrain::TerrainMaterial::Stone, // Approximate
                        crate::models::terrain::BlockType::Kelp => crate::models::terrain::TerrainMaterial::Leaves, // Approximate
                        crate::models::terrain::BlockType::TreasureChest => crate::models::terrain::TerrainMaterial::Stone,
                        crate::models::terrain::BlockType::IronDeposit => crate::models::terrain::TerrainMaterial::Iron,
                        crate::models::terrain::BlockType::PearlBed => crate::models::terrain::TerrainMaterial::Stone,
                        _ => crate::models::terrain::TerrainMaterial::Water,
                    };
                    chunk.cells[index] = crate::models::terrain::TerrainCell::new(material);
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Get block at world coordinates
    pub fn get_block(&self, world_x: i32, world_y: i32) -> Option<&crate::models::terrain::Block> {
        let chunk_x = (world_x as f32 / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        let chunk_y = (world_y as f32 / (self.chunk_size as f32 * PIXEL_SIZE)).floor() as i32;
        
        if let Some(chunk) = self.chunks.get(&(chunk_x, chunk_y)) {
            let local_x = (world_x - chunk_x * self.chunk_size as i32) as usize;
            let local_y = (world_y - chunk_y * self.chunk_size as i32) as usize;
            
            if local_x < self.chunk_size && local_y < self.chunk_size {
                let index = local_y * self.chunk_size + local_x;
                if index < chunk.cells.len() {
                    // For now, return None since we're using TerrainCell instead of Block
                    // This method needs to be updated to work with the new terrain system
                    return None;
                }
            }
        }
        
        None
    }
    
    /// Set render distance
    pub fn set_render_distance(&mut self, distance: i32) {
        self.render_distance = distance;
    }
    
    /// Get world seed
    pub fn get_seed(&self) -> u32 {
        self.world_seed
    }
}
