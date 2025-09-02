use crate::components::entities::game_entity::{GameEntity, EntityType, RenderData, RenderLayer, HealthComponent, InventoryComponent, StatsComponent};
use crate::math::Vec2 as V2;
use crate::models::player::Player;
use crate::models::raft::Raft;
use crate::models::ocean::FloatingItemType;
use crate::constants::*;
use super::*;

/// Factory for creating different types of game entities
pub struct EntityFactory {
    next_entity_id: u32,
}

impl EntityFactory {
    pub fn new() -> Self {
        Self {
            next_entity_id: 1,
        }
    }
    
    /// Create a player entity
    pub fn create_player(&mut self, position: V2) -> Box<dyn GameEntity> {
        let player = Player::new(position);
        Box::new(PlayerEntity::new(self.next_entity_id(), player))
    }
    
    /// Create a raft entity
    pub fn create_raft(&mut self, position: V2) -> Box<dyn GameEntity> {
        let raft = Raft::new(position);
        Box::new(RaftEntity::new(self.next_entity_id(), raft))
    }
    
    /// Create a fish entity
    pub fn create_fish(&mut self, position: V2, fish_type: FishType) -> Box<dyn GameEntity> {
        Box::new(FishEntity::new(self.next_entity_id(), position, fish_type))
    }
    
    /// Create a floating item entity
    pub fn create_floating_item(&mut self, position: V2, item_type: FloatingItemType) -> Box<dyn GameEntity> {
        Box::new(FloatingItemEntity::new(self.next_entity_id(), position, item_type))
    }
    
    /// Create a particle entity
    pub fn create_particle(&mut self, position: V2, velocity: V2) -> Box<dyn GameEntity> {
        Box::new(ParticleEntity::new(self.next_entity_id(), position, velocity))
    }
    
    /// Create a monster entity
    pub fn create_monster(&mut self, position: V2, monster_type: MonsterType) -> Box<dyn GameEntity> {
        Box::new(MonsterEntity::new(self.next_entity_id(), position, monster_type))
    }
    
    /// Get next entity ID
    fn next_entity_id(&mut self) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }
}

/// Fish types
#[derive(Clone, Copy, PartialEq)]
pub enum FishType {
    SmallFish,
    TropicalFish,
    DeepSeaFish,
    Shark,
}

/// Monster types
#[derive(Clone, Copy, PartialEq)]
pub enum MonsterType {
    SeaMonster,
    Kraken,
    GiantSquid,
}

/// Player entity wrapper
pub struct PlayerEntity {
    id: u32,
    player: Player,
    render_data: RenderData,
}

impl PlayerEntity {
    pub fn new(id: u32, player: Player) -> Self {
        let render_data = RenderData::new(
            player.pos.clone(),
            8.0, // PLAYER_RADIUS
            if player.on_raft { PLAYER_ON_RAFT_COLOR } else { PLAYER_SWIMMING_COLOR },
        ).with_layer(RenderLayer::Player);
        
        Self {
            id,
            player,
            render_data,
        }
    }
}

impl GameEntity for PlayerEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> EntityType { EntityType::Player }
    fn get_position(&self) -> V2 { self.player.pos.clone() }
    fn set_position(&mut self, pos: V2) { self.player.pos = pos; }
    fn get_velocity(&self) -> V2 { self.player.vel.clone() }
    fn set_velocity(&mut self, vel: V2) { self.player.vel = vel; }
    fn update(&mut self, _delta_time: f32) { /* Player updates handled elsewhere */ }
    fn should_remove(&self) -> bool { false }
    fn get_render_data(&self) -> RenderData { self.render_data.clone() }
}



impl crate::components::systems::ai_system::AIEntity for PlayerEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Players don't use AI
    }
    fn get_position(&self) -> V2 { self.player.pos.clone() }
    fn set_position(&mut self, pos: V2) { self.player.pos = pos; }
    fn get_velocity(&self) -> V2 { self.player.vel.clone() }
    fn set_velocity(&mut self, vel: V2) { self.player.vel = vel; }
}

/// Raft entity wrapper
pub struct RaftEntity {
    id: u32,
    raft: Raft,
    render_data: RenderData,
}

impl RaftEntity {
    pub fn new(id: u32, raft: Raft) -> Self {
        let render_data = RenderData::new(
            raft.center.clone(),
            32.0, // RAFT_TILE_SIZE
            RAFT_WOOD_FLOOR_COLOR,
        ).with_layer(RenderLayer::Entity);
        
        Self {
            id,
            raft,
            render_data,
        }
    }
}

impl GameEntity for RaftEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> EntityType { EntityType::Raft }
    fn get_position(&self) -> V2 { self.raft.center.clone() }
    fn set_position(&mut self, pos: V2) { self.raft.center = pos; }
    fn get_velocity(&self) -> V2 { V2::zero() } // Raft doesn't have velocity
    fn set_velocity(&mut self, _vel: V2) { /* Raft doesn't have velocity */ }
    fn update(&mut self, _delta_time: f32) { /* Raft updates handled elsewhere */ }
    fn should_remove(&self) -> bool { false }
    fn get_render_data(&self) -> RenderData { self.render_data.clone() }
}



impl crate::components::systems::ai_system::AIEntity for RaftEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Rafts don't use AI
    }
    fn get_position(&self) -> V2 { self.raft.center.clone() }
    fn set_position(&mut self, pos: V2) { self.raft.center = pos; }
    fn get_velocity(&self) -> V2 { V2::zero() }
    fn set_velocity(&mut self, _vel: V2) { }
}

/// Fish entity
pub struct FishEntity {
    id: u32,
    position: V2,
    velocity: V2,
    fish_type: FishType,
    health: HealthComponent,
    stats: StatsComponent,
    render_data: RenderData,
    lifetime: f32,
}

impl FishEntity {
    pub fn new(id: u32, position: V2, fish_type: FishType) -> Self {
        let (size, color) = match fish_type {
            FishType::SmallFish => (4.0, 0xFFB6C1FF),
            FishType::TropicalFish => (6.0, 0xFFFF00FF),
            FishType::DeepSeaFish => (8.0, 0x4169E1FF),
            FishType::Shark => (16.0, 0x696969FF),
        };
        
        let speed = match fish_type {
            FishType::SmallFish => 1.0,
            FishType::TropicalFish => 1.5,
            FishType::DeepSeaFish => 0.8,
            FishType::Shark => 2.5,
        };
        
        let render_data = RenderData::new(position.clone(), size, color)
            .with_layer(RenderLayer::Entity);
        
        Self {
            id,
            position,
            velocity: V2::zero(),
            fish_type,
            health: HealthComponent::new(50.0),
            stats: StatsComponent::new(speed, 10.0, 5.0, 100.0),
            render_data,
            lifetime: 0.0,
        }
    }
}

impl GameEntity for FishEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> EntityType { EntityType::Fish }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
    fn update(&mut self, delta_time: f32) { 
        self.position = self.position.add(self.velocity.scale(delta_time));
        self.lifetime += delta_time;
        self.health.update(delta_time);
        self.stats.regenerate_stamina(delta_time);
    }
    fn should_remove(&self) -> bool { 
        !self.health.is_alive() || self.lifetime > 300.0 // 5 minutes lifetime
    }
    fn get_render_data(&self) -> RenderData { 
        let mut data = self.render_data.clone();
        data.position = self.position.clone();
        data
    }
}



impl crate::components::systems::ai_system::AIEntity for FishEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish
    }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
}

/// Floating item entity
pub struct FloatingItemEntity {
    id: u32,
    position: V2,
    velocity: V2,
    item_type: FloatingItemType,
    render_data: RenderData,
    lifetime: f32,
}

impl FloatingItemEntity {
    pub fn new(id: u32, position: V2, item_type: FloatingItemType) -> Self {
        let render_data = RenderData::new(position.clone(), 8.0, item_type.color())
            .with_layer(RenderLayer::Entity);
        
        Self {
            id,
            position,
            velocity: V2::zero(),
            item_type,
            render_data,
            lifetime: 0.0,
        }
    }
}

impl GameEntity for FloatingItemEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> EntityType { EntityType::FloatingItem }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
    fn update(&mut self, delta_time: f32) { 
        self.position = self.position.add(self.velocity.scale(delta_time));
        self.lifetime += delta_time;
    }
    fn should_remove(&self) -> bool { 
        self.lifetime > 600.0 // 10 minutes lifetime
    }
    fn get_render_data(&self) -> RenderData { 
        let mut data = self.render_data.clone();
        data.position = self.position.clone();
        data
    }
}



impl crate::components::systems::ai_system::AIEntity for FloatingItemEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Items don't use AI
    }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
}

/// Particle entity
pub struct ParticleEntity {
    id: u32,
    position: V2,
    velocity: V2,
    render_data: RenderData,
    lifetime: f32,
    max_lifetime: f32,
}

impl ParticleEntity {
    pub fn new(id: u32, position: V2, velocity: V2) -> Self {
        let render_data = RenderData::new(position.clone(), 2.0, PARTICLE_COLOR)
            .with_layer(RenderLayer::Entity);
        
        Self {
            id,
            position,
            velocity,
            render_data,
            lifetime: 0.0,
            max_lifetime: 2.0, // 2 seconds
        }
    }
}

impl GameEntity for ParticleEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> EntityType { EntityType::Particle }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
    fn update(&mut self, delta_time: f32) { 
        self.position = self.position.add(self.velocity.scale(delta_time));
        self.lifetime += delta_time;
        
        // Apply gravity
        self.velocity.y += GRAVITY * delta_time;
    }
    fn should_remove(&self) -> bool { 
        self.lifetime > self.max_lifetime
    }
    fn get_render_data(&self) -> RenderData { 
        let mut data = self.render_data.clone();
        data.position = self.position.clone();
        
        // Fade out based on lifetime
        let alpha = ((self.max_lifetime - self.lifetime) / self.max_lifetime) as u32;
        let color = (data.color & 0x00FFFFFF) | (alpha << 24);
        data.color = color;
        
        data
    }
}



impl crate::components::systems::ai_system::AIEntity for ParticleEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Particles don't use AI
    }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
}

/// Monster entity
pub struct MonsterEntity {
    id: u32,
    position: V2,
    velocity: V2,
    monster_type: MonsterType,
    health: HealthComponent,
    stats: StatsComponent,
    render_data: RenderData,
}

impl MonsterEntity {
    pub fn new(id: u32, position: V2, monster_type: MonsterType) -> Self {
        let (size, color) = match monster_type {
            MonsterType::SeaMonster => (20.0, 0x8B0000FF),
            MonsterType::Kraken => (30.0, 0x4B0082FF),
            MonsterType::GiantSquid => (25.0, 0x800080FF),
        };
        
        let render_data = RenderData::new(position.clone(), size, color)
            .with_layer(RenderLayer::Entity);
        
        Self {
            id,
            position,
            velocity: V2::zero(),
            monster_type,
            health: HealthComponent::new(200.0),
            stats: StatsComponent::new(1.5, 25.0, 15.0, 150.0),
            render_data,
        }
    }
}

impl GameEntity for MonsterEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> EntityType { EntityType::Monster }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
    fn update(&mut self, delta_time: f32) { 
        self.position = self.position.add(self.velocity.scale(delta_time));
        self.health.update(delta_time);
        self.stats.regenerate_stamina(delta_time);
    }
    fn should_remove(&self) -> bool { 
        !self.health.is_alive()
    }
    fn get_render_data(&self) -> RenderData { 
        let mut data = self.render_data.clone();
        data.position = self.position.clone();
        data
    }
}



impl crate::components::systems::ai_system::AIEntity for MonsterEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Monster
    }
    fn get_position(&self) -> V2 { self.position.clone() }
    fn set_position(&mut self, pos: V2) { self.position = pos; }
    fn get_velocity(&self) -> V2 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V2) { self.velocity = vel; }
}
