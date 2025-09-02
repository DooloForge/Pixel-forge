use crate::math::Vec2 as V2;
use crate::models::ocean::FloatingItemType;
use crate::models::particle::Particle;
use crate::constants::*;
use turbo::random;

/// Handles spawning of various game entities
#[turbo::serialize]
pub struct SpawnSystem {
    spawn_timers: std::collections::HashMap<SpawnType, u32>,
    spawn_rates: std::collections::HashMap<SpawnType, u32>,
    max_entities: std::collections::HashMap<SpawnType, usize>,
}

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
#[turbo::serialize]
pub enum SpawnType {
    FloatingItem,
    Fish,
    Bubble,
    Particle,
    Coral,
    Treasure,
}

impl SpawnSystem {
    pub fn new() -> Self {
        let mut spawn_rates = std::collections::HashMap::new();
        spawn_rates.insert(SpawnType::FloatingItem, 300); // Every 5 seconds
        spawn_rates.insert(SpawnType::Fish, 180);          // Every 3 seconds
        spawn_rates.insert(SpawnType::Bubble, 60);         // Every second
        spawn_rates.insert(SpawnType::Particle, 10);       // Every 1/6 second
        spawn_rates.insert(SpawnType::Coral, 600);         // Every 10 seconds
        spawn_rates.insert(SpawnType::Treasure, 1200);     // Every 20 seconds
        
        let mut max_entities = std::collections::HashMap::new();
        max_entities.insert(SpawnType::FloatingItem, 50);
        max_entities.insert(SpawnType::Fish, 30);
        max_entities.insert(SpawnType::Bubble, 100);
        max_entities.insert(SpawnType::Particle, 200);
        max_entities.insert(SpawnType::Coral, 20);
        max_entities.insert(SpawnType::Treasure, 10);
        
        Self {
            spawn_timers: std::collections::HashMap::new(),
            spawn_rates,
            max_entities,
        }
    }
    
    /// Update spawn timers and trigger spawns
    pub fn update(&mut self, player_pos: &V2, current_counts: &std::collections::HashMap<SpawnType, usize>) {
        let spawn_types = [SpawnType::FloatingItem, SpawnType::Fish, SpawnType::Bubble, SpawnType::Coral, SpawnType::Treasure];
        
        for spawn_type in spawn_types {
            let rate = self.spawn_rates.get(&spawn_type).unwrap_or(&300);
            let max_count = self.max_entities.get(&spawn_type).unwrap_or(&50);
            let current_count = current_counts.get(&spawn_type).unwrap_or(&0);
            
            // Check if we should spawn
            let should_spawn = if let Some(timer) = self.spawn_timers.get(&spawn_type) {
                *timer >= *rate && *current_count < *max_count
            } else {
                false
            };
            
            // Update timer
            if let Some(timer) = self.spawn_timers.get_mut(&spawn_type) {
                if should_spawn {
                    *timer = 0;
                } else {
                    *timer += 1;
                }
            }
            
            // Trigger spawn if needed
            if should_spawn {
                self.trigger_spawn(&spawn_type, player_pos);
            }
        }
    }
    
    /// Trigger a specific spawn type
    fn trigger_spawn(&self, spawn_type: &SpawnType, player_pos: &V2) {
        match spawn_type {
            SpawnType::FloatingItem => self.spawn_floating_item(player_pos),
            SpawnType::Fish => self.spawn_fish(player_pos),
            SpawnType::Bubble => self.spawn_bubble(player_pos),
            SpawnType::Coral => self.spawn_coral(player_pos),
            SpawnType::Treasure => self.spawn_treasure(player_pos),
            _ => {}
        }
    }
    
    /// Spawn a floating item near the player
    fn spawn_floating_item(&self, player_pos: &V2) {
        let angle = random::f32() * 6.28318;
        let distance = 100.0 + random::f32() * 200.0;
        let spawn_pos = V2::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + angle.sin() * distance
        );
        
        // Ensure item spawns near water surface
        let final_pos = V2::new(spawn_pos.x, (-20.0 + random::f32() * 40.0).max(-50.0));
        
        // TODO: Add to floating items collection
        // This would be handled by the Ocean system
    }
    
    /// Spawn a fish near the player
    fn spawn_fish(&self, player_pos: &V2) {
        let angle = random::f32() * 6.28318;
        let distance = 80.0 + random::f32() * 150.0;
        let spawn_pos = V2::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + angle.sin() * distance
        );
        
        // Ensure fish spawns underwater
        let final_pos = V2::new(spawn_pos.x, (10.0 + random::f32() * 100.0).max(20.0));
        
        // TODO: Add to fish collection
        // This would be handled by the UnderwaterWorld system
    }
    
    /// Spawn a bubble particle
    fn spawn_bubble(&self, player_pos: &V2) {
        let offset = V2::new(
            (random::f32() - 0.5) * 20.0,
            (random::f32() - 0.5) * 20.0
        );
        let spawn_pos = player_pos.add(offset);
        
        // TODO: Add to bubble collection
        // This would be handled by the UnderwaterWorld system
    }
    
    /// Spawn coral formation
    fn spawn_coral(&self, player_pos: &V2) {
        let angle = random::f32() * 6.28318;
        let distance = 150.0 + random::f32() * 300.0;
        let spawn_pos = V2::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + angle.sin() * distance
        );
        
        // Ensure coral spawns deep underwater
        let final_pos = V2::new(spawn_pos.x, (50.0 + random::f32() * 200.0).max(80.0));
        
        // TODO: Add to coral collection
        // This would be handled by the UnderwaterWorld system
    }
    
    /// Spawn treasure
    fn spawn_treasure(&self, player_pos: &V2) {
        let angle = random::f32() * 6.28318;
        let distance = 200.0 + random::f32() * 400.0;
        let spawn_pos = V2::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + angle.sin() * distance
        );
        
        // Ensure treasure spawns very deep underwater
        let final_pos = V2::new(spawn_pos.x, (100.0 + random::f32() * 300.0).max(150.0));
        
        // TODO: Add to treasure collection
        // This would be handled by the UnderwaterWorld system
    }
    
    /// Spawn impact particles at a specific location
    pub fn spawn_impact_particles(&self, pos: &V2, count: usize) -> Vec<Particle> {
        let mut particles = Vec::new();
        
        for _ in 0..count {
            let angle = random::f32() * 6.28318;
            let speed = 0.5 + random::f32() * 2.0;
            let velocity = V2::new(angle.cos() * speed, angle.sin() * speed - 1.0);
            
            particles.push(Particle::new(pos.clone(), velocity));
        }
        
        particles
    }
    
    /// Set spawn rate for a specific type
    pub fn set_spawn_rate(&mut self, spawn_type: SpawnType, rate: u32) {
        self.spawn_rates.insert(spawn_type, rate);
    }
    
    /// Set maximum entities for a specific type
    pub fn set_max_entities(&mut self, spawn_type: SpawnType, max: usize) {
        self.max_entities.insert(spawn_type, max);
    }
}
