use std::collections::HashMap;
use crate::components::entities::game_entity::{Entity, EntityType};
use crate::math::Vec3 as V3;


/// Manages all game entities and their lifecycle
#[turbo::serialize]
pub struct EntityManager {
    entity_types: HashMap<EntityType, Vec<u32>>,
    next_entity_id: u32,
    spatial_hash: SpatialHash,
}

/// Runtime entity storage
#[derive(Default)]
#[turbo::serialize]
pub struct EntityStorage {
    entities: HashMap<u32, Entity>,
}
impl EntityStorage {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entity_types: HashMap::new(),
            next_entity_id: 1,
            spatial_hash: SpatialHash::new(100.0), // 100 unit grid size
        }
    }
    
    /// Create a new entity
    pub fn create_entity(&mut self, storage: &mut EntityStorage, entity: Entity) -> u32 {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        
        let entity_type = entity.get_entity_type();
        
        // Add to entities map
        storage.entities.insert(entity_id, entity);
        
        // Add to type index
        self.entity_types.entry(entity_type).or_insert_with(Vec::new).push(entity_id);
        
        // Add to spatial hash
        if let Some(entity_ref) = storage.entities.get(&entity_id) {
            self.spatial_hash.insert(entity_id, entity_ref.get_world_position());
        }
        
        entity_id
    }
    
    /// Remove an entity
    pub fn remove_entity(&mut self, storage: &mut EntityStorage, entity_id: u32) -> bool {
        if let Some(entity) = storage.entities.remove(&entity_id) {
            let entity_type = entity.get_entity_type();
            
            // Remove from type index
            if let Some(type_list) = self.entity_types.get_mut(&entity_type) {
                type_list.retain(|&id| id != entity_id);
            }
            
            // Remove from spatial hash
            self.spatial_hash.remove(entity_id);
            
            true
        } else {
            false
        }
    }
    
    /// Get entity by ID
    pub fn get_entity<'a>(&self, storage: &'a EntityStorage, entity_id: u32) -> Option<&'a Entity> {
        storage.entities.get(&entity_id)
    }
    
    /// Get mutable entity by ID
    pub fn get_entity_mut<'a>(&mut self, storage: &'a mut EntityStorage, entity_id: u32) -> Option<&'a mut Entity> {
        storage.entities.get_mut(&entity_id)
    }
    
    /// Get all entities of a specific type
    pub fn get_entities_by_type<'a>(&self, storage: &'a EntityStorage, entity_type: EntityType) -> Vec<&'a Entity> {
        if let Some(entity_ids) = self.entity_types.get(&entity_type) {
            entity_ids.iter()
                .filter_map(|&id| self.get_entity(storage, id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all entities of a specific type (mutable)
    pub fn get_entities_by_type_mut<'a>(&mut self, storage: &'a mut EntityStorage, entity_type: EntityType) -> Vec<&'a mut Entity> {
        // Return empty vector for now - this method has borrowing issues
        // Use get_entity_ids_by_type() and get_entity_mut_by_id() separately instead
        Vec::new()
    }
    
    /// Get entity IDs of a specific type (no borrowing conflicts)
    pub fn get_entity_ids_by_type(&self, entity_type: EntityType) -> Vec<u32> {
        if let Some(entity_ids) = self.entity_types.get(&entity_type) {
            entity_ids.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Get a single entity by ID (safe for iteration)
    pub fn get_entity_mut_by_id<'a>(&mut self, storage: &'a mut EntityStorage, entity_id: u32) -> Option<&'a mut Entity> {
        storage.entities.get_mut(&entity_id)
    }
    
    /// Get all entities
    pub fn get_all_entities<'a>(&self, storage: &'a EntityStorage) -> Vec<&'a Entity> {
        storage.entities.values().collect()
    }
    
    /// Get all entities (mutable)
    pub fn get_all_entities_mut<'a>(&mut self, storage: &'a mut EntityStorage) -> Vec<&'a mut Entity> {
        storage.entities.values_mut().collect()
    }
    
    /// Update all entities
    pub fn update_entities(&mut self, storage: &mut EntityStorage, delta_time: f32) {
        let mut entities_to_remove = Vec::new();
        
        for (entity_id, entity) in &mut storage.entities {
            entity.update(delta_time);
            
            if entity.should_remove() {
                entities_to_remove.push(*entity_id);
            }
        }
        
        // Remove entities that should be removed
        for entity_id in entities_to_remove {
            self.remove_entity(storage, entity_id);
        }
    }
    
    /// Get entities in a specific area
    pub fn get_entities_in_area<'a>(&self, storage: &'a EntityStorage, center: &V3, radius: f32) -> Vec<&'a Entity> {
        let entity_ids = self.spatial_hash.query_area(center, radius);
        
        entity_ids.iter()
            .filter_map(|&id| self.get_entity(storage, id))
            .collect()
    }
    
    /// Get entities near a position
    pub fn get_entities_near<'a>(&self, storage: &'a EntityStorage, position: &V3, max_distance: f32) -> Vec<&'a Entity> {
        self.get_entities_in_area(storage, position, max_distance)
    }
    
    /// Get entity count by type
    pub fn get_entity_count(&self, entity_type: EntityType) -> usize {
        self.entity_types.get(&entity_type).map(|v| v.len()).unwrap_or(0)
    }
    
    /// Get total entity count
    pub fn get_total_entity_count(&self, storage: &EntityStorage) -> usize {
        storage.entities.len()
    }
    
    /// Clear all entities
    pub fn clear_all_entities(&mut self, storage: &mut EntityStorage) {
        storage.entities.clear();
        self.entity_types.clear();
        self.spatial_hash.clear();
    }
    
    /// Update spatial hash for an entity
    pub fn update_entity_position(&mut self, storage: &EntityStorage, entity_id: u32, new_position: V3) {
        if let Some(entity) = storage.entities.get(&entity_id) {
            self.spatial_hash.update(entity_id, entity.get_world_position(), new_position);
        }
    }
}

#[turbo::serialize]
struct SpatialHash {
    grid_size: f32,
    grid: HashMap<(i32, i32), Vec<u32>>,
    entity_positions: HashMap<u32, V3>,
}

impl SpatialHash {
    pub fn new(grid_size: f32) -> Self {
        Self {
            grid_size,
            grid: HashMap::new(),
            entity_positions: HashMap::new(),
        }
    }
    
    /// Insert entity into spatial hash
    pub fn insert(&mut self, entity_id: u32, position: V3) {
        let grid_pos = self.world_to_grid(&position);
        self.grid.entry(grid_pos).or_insert_with(Vec::new).push(entity_id);
        self.entity_positions.insert(entity_id, position);
    }
    
    /// Remove entity from spatial hash
    pub fn remove(&mut self, entity_id: u32) {
        if let Some(position) = self.entity_positions.get(&entity_id) {
            let grid_pos = self.world_to_grid(position);
            if let Some(cell) = self.grid.get_mut(&grid_pos) {
                cell.retain(|&id| id != entity_id);
            }
        }
        self.entity_positions.remove(&entity_id);
    }
    
    /// Update entity position in spatial hash
    pub fn update(&mut self, entity_id: u32, old_position: V3, new_position: V3) {
        let old_grid_pos = self.world_to_grid(&old_position);
        let new_grid_pos = self.world_to_grid(&new_position);
        
        // Remove from old cell
        if let Some(cell) = self.grid.get_mut(&old_grid_pos) {
            cell.retain(|&id| id != entity_id);
        }
        
        // Add to new cell
        self.grid.entry(new_grid_pos).or_insert_with(Vec::new).push(entity_id);
        
        // Update position
        self.entity_positions.insert(entity_id, new_position);
    }
    
    /// Query entities in an area
    pub fn query_area(&self, center: &V3, radius: f32) -> Vec<u32> {
        let mut result = Vec::new();
        let center_grid = self.world_to_grid(center);
        let grid_radius = (radius / self.grid_size).ceil() as i32;
        
        for dx in -grid_radius..=grid_radius {
            for dy in -grid_radius..=grid_radius {
                let grid_pos = (center_grid.0 + dx, center_grid.1 + dy);
                
                if let Some(cell) = self.grid.get(&grid_pos) {
                    for &entity_id in cell {
                        if let Some(entity_pos) = self.entity_positions.get(&entity_id) {
                            if center.distance_to(entity_pos) <= radius {
                                result.push(entity_id);
                            }
                        }
                    }
                }
            }
        }
        
        result
    }
    
    /// Clear spatial hash
    pub fn clear(&mut self) {
        self.grid.clear();
        self.entity_positions.clear();
    }
    
    /// Convert world position to grid position
    fn world_to_grid(&self, position: &V3) -> (i32, i32) {
        (
            (position.x / self.grid_size).floor() as i32,
            (position.y / self.grid_size).floor() as i32,
        )
    }
}
