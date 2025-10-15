use crate::components::entities::game_entity::{Entity, RenderData, RenderLayer, HealthComponent, StatsComponent};
use crate::math::Vec3 as V3;
use crate::models::player::Player;
use crate::models::raft::Raft;
use crate::models::ocean::FloatingItemType;
use crate::constants::*;
// use super::*;

/// Factory for creating different types of game entities
#[turbo::serialize]
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
    pub fn create_player(&mut self, position: V3) -> Entity {
        let player = Player::new(position);
        Entity::Player(PlayerEntity::new(self.next_entity_id(), player))
    }
    
    /// Create a player entity from existing player data
    pub fn create_player_from_existing(&mut self, player: Player) -> Entity {
        Entity::Player(PlayerEntity::new(self.next_entity_id(), player))
    }
    
    /// Create a raft entity
    pub fn create_raft(&mut self, position: V3) -> Entity {
        let raft = Raft::new(position);
        Entity::Raft(RaftEntity::new(self.next_entity_id(), raft))
    }
    
    /// Create a fish entity
    pub fn create_fish(&mut self, position: V3, fish_type: FishType) -> Entity {
        Entity::Fish(FishEntity::new(self.next_entity_id(), position, fish_type))
    }
    
    /// Create a floating item entity
    pub fn create_floating_item(&mut self, position: V3, item_type: FloatingItemType) -> Entity {
        Entity::FloatingItem(FloatingItemEntity::new(self.next_entity_id(), position, item_type))
    }
    
    /// Create a particle entity
    pub fn create_particle(&mut self, position: V3, velocity: V3) -> Entity {
        Entity::Particle(ParticleEntity::new(self.next_entity_id(), position, velocity))
    }
    
    /// Create a monster entity
    pub fn create_monster(&mut self, position: V3, monster_type: MonsterType) -> Entity {
        Entity::Monster(MonsterEntity::new(self.next_entity_id(), position, monster_type))
    }
    
    /// Create a hook entity
    pub fn create_hook(&mut self, owner_id: u32) -> Entity {
        Entity::Hook(HookEntity::new(self.next_entity_id(), owner_id))
    }
    
    /// Get next entity ID
    fn next_entity_id(&mut self) -> u32 {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }
}

/// Fish types
#[turbo::serialize]
pub enum FishType {
    SmallFish,
    TropicalFish,
    DeepSeaFish,
    Shark,
}

/// Monster types
#[turbo::serialize]
pub enum MonsterType {
    SeaMonster,
    Kraken,
    GiantSquid,
}

/// Player entity wrapper
#[turbo::serialize]
pub struct PlayerEntity {
    pub id: u32,
    pub player: Player,
    pub render_data: RenderData,
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

// GameEntity trait removed; behavior handled via Entity enum



impl crate::components::systems::ai_system::AIEntity for PlayerEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Players don't use AI
    }
    fn get_position(&self) -> V3 { self.player.pos.clone() }
    fn set_position(&mut self, pos: V3) { self.player.pos = pos; }
    fn get_velocity(&self) -> V3 { self.player.vel.clone() }
    fn set_velocity(&mut self, vel: V3) { self.player.vel = vel; }
}

/// Raft entity wrapper
#[turbo::serialize]
pub struct RaftEntity {
    pub id: u32,
    pub raft: Raft,
    pub render_data: RenderData,
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

// GameEntity trait removed; behavior handled via Entity enum



impl crate::components::systems::ai_system::AIEntity for RaftEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Rafts don't use AI
    }
    fn get_position(&self) -> V3 { self.raft.center.clone() }
    fn set_position(&mut self, pos: V3) { self.raft.center = pos; }
    fn get_velocity(&self) -> V3 { V3::zero() }
    fn set_velocity(&mut self, _vel: V3) { }
}

/// Fish entity
#[turbo::serialize]
pub struct FishEntity {
    pub id: u32,
    pub position: V3,
    pub velocity: V3,
    pub spawn_origin: V3,
    pub fish_type: FishType,
    pub health: HealthComponent,
    pub stats: StatsComponent,
    pub render_data: RenderData,
    pub lifetime: f32,
}

impl FishEntity {
    pub fn new(id: u32, position: V3, fish_type: FishType) -> Self {
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
            velocity: V3::zero(),
            spawn_origin: position.clone(),
            fish_type,
            health: HealthComponent::new(50.0),
            stats: StatsComponent::new(speed, 10.0, 5.0, 100.0),
            render_data,
            lifetime: 0.0,
        }
    }
}

// GameEntity trait removed; behavior handled via Entity enum



impl crate::components::systems::ai_system::AIEntity for FishEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish
    }
    fn get_position(&self) -> V3 { self.position.clone() }
    fn set_position(&mut self, pos: V3) { self.position = pos; }
    fn get_velocity(&self) -> V3 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V3) { self.velocity = vel; }
}

/// Floating item entity
#[turbo::serialize]
pub struct FloatingItemEntity {
    pub id: u32,
    pub position: V3,
    pub velocity: V3,
    pub spawn_origin: V3,
    pub item_type: FloatingItemType,
    pub render_data: RenderData,
    pub lifetime: f32,
}

impl FloatingItemEntity {
    pub fn new(id: u32, position: V3, item_type: FloatingItemType) -> Self {
        let size = item_type.size();
        let render_data = RenderData::new(position.clone(), size, item_type.color())
            .with_layer(RenderLayer::Entity);
        
        Self {
            id,
            position,
            velocity: V3::zero(),
            spawn_origin: position.clone(),
            item_type,
            render_data,
            lifetime: 0.0,
        }
    }
}

// GameEntity trait removed; behavior handled via Entity enum



impl crate::components::systems::ai_system::AIEntity for FloatingItemEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Items don't use AI
    }
    fn get_position(&self) -> V3 { self.position.clone() }
    fn set_position(&mut self, pos: V3) { self.position = pos; }
    fn get_velocity(&self) -> V3 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V3) { self.velocity = vel; }
}

/// Particle entity
#[turbo::serialize]
pub struct ParticleEntity {
    pub id: u32,
    pub position: V3,
    pub velocity: V3,
    pub render_data: RenderData,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

impl ParticleEntity {
    pub fn new(id: u32, position: V3, velocity: V3) -> Self {
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

// GameEntity trait removed; behavior handled via Entity enum



impl crate::components::systems::ai_system::AIEntity for ParticleEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Fish // Particles don't use AI
    }
    fn get_position(&self) -> V3 { self.position.clone() }
    fn set_position(&mut self, pos: V3) { self.position = pos; }
    fn get_velocity(&self) -> V3 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V3) { self.velocity = vel; }
}

/// Monster entity
#[turbo::serialize]
pub struct MonsterEntity {
    pub id: u32,
    pub position: V3,
    pub velocity: V3,
    pub monster_type: MonsterType,
    pub health: HealthComponent,
    pub stats: StatsComponent,
    pub render_data: RenderData,
}

/// Hook entity
#[turbo::serialize]
pub struct HookEntity {
    pub id: u32,
    pub hook: crate::models::hook::Hook,
    pub render_data: RenderData,
    pub player_pos: V3, // Store player position for line rendering
}

impl MonsterEntity {
    pub fn new(id: u32, position: V3, monster_type: MonsterType) -> Self {
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
            velocity: V3::zero(),
            monster_type,
            health: HealthComponent::new(200.0),
            stats: StatsComponent::new(1.5, 25.0, 15.0, 150.0),
            render_data,
        }
    }
}

impl HookEntity {
    pub fn new(id: u32, owner_id: u32) -> Self {
        let hook = crate::models::hook::Hook::new(owner_id);
        // Start with hook position (will be updated when launched)
        let render_data = RenderData::new(hook.position.clone(), 12.0, 0x8B4513FF) // Brown hook
            .with_layer(RenderLayer::Entity);
        
        Self {
            id,
            hook,
            render_data,
            player_pos: V3::zero(), // Will be updated when hook is launched
        }
    }
}

// GameEntity trait removed; behavior handled via Entity enum



impl crate::components::systems::ai_system::AIEntity for MonsterEntity {
    fn get_id(&self) -> u32 { self.id }
    fn get_entity_type(&self) -> crate::components::systems::ai_system::EntityType { 
        crate::components::systems::ai_system::EntityType::Monster
    }
    fn get_position(&self) -> V3 { self.position.clone() }
    fn set_position(&mut self, pos: V3) { self.position = pos; }
    fn get_velocity(&self) -> V3 { self.velocity.clone() }
    fn set_velocity(&mut self, vel: V3) { self.velocity = vel; }
}
