use crate::math::Vec3 as V3;
use crate::models::particle::Particle;
use turbo::random;

/// Handles spawning of various game entities
#[turbo::serialize]
pub struct SpawnSystem {
    spawn_timers: std::collections::HashMap<SpawnType, u32>,
    spawn_rates: std::collections::HashMap<SpawnType, u32>,
    max_entities: std::collections::HashMap<SpawnType, usize>,
    pending_spawns: Vec<(SpawnType, V3)>,
    wind: V3,
}

#[derive(Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
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
            pending_spawns: Vec::new(),
            wind: V3::zero(),
        }
    }
    
    /// Update cached wind vector used for directional spawns
    pub fn set_wind(&mut self, wind: V3) { self.wind = wind; }
    
    /// Update spawn timers and trigger spawns
    pub fn update(&mut self, player_pos: &V3, current_counts: &std::collections::HashMap<SpawnType, usize>) {
        let spawn_types = [SpawnType::FloatingItem, SpawnType::Fish, SpawnType::Bubble, SpawnType::Coral, SpawnType::Treasure];
        
        for spawn_type in spawn_types {
            let rate = *self.spawn_rates.get(&spawn_type).unwrap_or(&300);
            let max_count = *self.max_entities.get(&spawn_type).unwrap_or(&50);
            let current_count = *current_counts.get(&spawn_type).unwrap_or(&0);
            
            // Ensure timer exists; initialize to rate so first update can spawn immediately
            let init = match spawn_type { SpawnType::FloatingItem | SpawnType::Fish => rate, _ => 0 };
            let timer = self.spawn_timers.entry(spawn_type).or_insert(init);
            
            let should_spawn = *timer >= rate && current_count < max_count;
            if should_spawn { *timer = 0; } else { *timer += 1; }
            
            if should_spawn {
                self.trigger_spawn(&spawn_type, player_pos);
            }
        }
    }
    
    /// Trigger a specific spawn type
    fn trigger_spawn(&mut self, spawn_type: &SpawnType, player_pos: &V3) {
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
    fn spawn_floating_item(&mut self, player_pos: &V3) {
        // Always spawn at left edge so it flows left -> right across the view
        let (screen_w, screen_h) = turbo::resolution();
        let half_w = screen_w as f32 * 0.5;
        let margin = 40.0;
        let x = player_pos.x - half_w - margin;
        // Near the water surface (y ~ 0)
        let y = (-4.0 + random::f32() * 8.0).clamp(-10.0, 10.0);
        let final_pos = V3::new(x, y, 0.0);
        self.pending_spawns.push((SpawnType::FloatingItem, final_pos));
    }
    
    /// Spawn a fish near the player
    fn spawn_fish(&mut self, player_pos: &V3) {
        // Spawn underwater using new world pos: keep y (surface), set z to negative depth
        let (screen_w, _screen_h) = turbo::resolution();
        let half_w = screen_w as f32 * 0.5;
        let margin = 60.0;
        let left_side = random::f32() < 0.5;
        let x = if left_side { player_pos.x - half_w - margin } else { player_pos.x + half_w + margin };
        let y = player_pos.y;
        let z = -(20.0 + random::f32() * 120.0);
        let final_pos = V3::new(x, y, z);
        self.pending_spawns.push((SpawnType::Fish, final_pos));
    }
    
    /// Spawn a bubble particle
    fn spawn_bubble(&mut self, player_pos: &V3) {
        let offset = V3::new(
            (random::f32() - 0.5) * 20.0,
            (random::f32() - 0.5) * 20.0,
            0.0
        );
        let _spawn_pos = player_pos.add(offset);
        // TODO: enqueue or create bubble entity when bubble system exists
    }
    
    /// Spawn coral formation
    fn spawn_coral(&mut self, player_pos: &V3) {
        let angle = random::f32() * 6.28318;
        let distance = 150.0 + random::f32() * 300.0;
        let spawn_pos = V3::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + angle.sin() * distance,
            0.0
        );
        
        // Ensure coral spawns deep underwater
        let _final_pos = V3::new(spawn_pos.x, (50.0 + random::f32() * 200.0).max(80.0), 0.0);
        // TODO: enqueue coral when system exists
    }
    
    /// Spawn treasure
    fn spawn_treasure(&mut self, player_pos: &V3) {
        let angle = random::f32() * 6.28318;
        let distance = 200.0 + random::f32() * 400.0;
        let spawn_pos = V3::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + angle.sin() * distance,
            0.0
        );
        
        // Ensure treasure spawns very deep underwater
        let _final_pos = V3::new(spawn_pos.x, (100.0 + random::f32() * 300.0).max(150.0), 0.0);
        // TODO: enqueue treasure when system exists
    }

    /// Drain pending spawn requests
    pub fn drain_pending(&mut self) -> Vec<(SpawnType, V3)> {
        let mut out = Vec::new();
        std::mem::swap(&mut out, &mut self.pending_spawns);
        out
    }
    
    /// Spawn impact particles at a specific location
    pub fn spawn_impact_particles(&self, pos: &V3, count: usize) -> Vec<Particle> {
        let mut particles = Vec::new();
        
        for _ in 0..count {
            let angle = random::f32() * 6.28318;
            let speed = 0.5 + random::f32() * 2.0;
            let velocity = V3::new(angle.cos() * speed, angle.sin() * speed - 1.0, 0.0);
            
            particles.push(Particle::new(V3::new(pos.x, pos.y, 0.0), velocity));
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
