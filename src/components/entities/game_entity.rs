use crate::math::Vec2 as V2;
use crate::math::Vec3;

#[derive(Copy, PartialEq, Eq, Ord, PartialOrd)]
#[turbo::serialize]
pub enum RenderLayer {
    Background,
    Terrain,
    Underwater,
    Entity,
    Player,
    UI,
    Foreground,
}

#[turbo::serialize]
pub struct RenderData {
    pub position: V2,
    pub size: f32,
    pub color: u32,
    pub visible: bool,
    pub layer: RenderLayer,
}

impl RenderData {
    pub fn new(position: V2, size: f32, color: u32) -> Self {
        Self { position, size, color, visible: true, layer: RenderLayer::Entity }
    }
    pub fn with_layer(mut self, layer: RenderLayer) -> Self {
        self.layer = layer;
        self
    }
}

#[derive(Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[turbo::serialize]
pub enum EntityType {
    Player,
    Raft,
    Fish,
    Monster,
    Shark,
    FloatingItem,
    Particle,
}

#[turbo::serialize]
pub enum Entity {
    Player(super::entity_factory::PlayerEntity),
    Raft(super::entity_factory::RaftEntity),
    Fish(super::entity_factory::FishEntity),
    Monster(super::entity_factory::MonsterEntity),
    FloatingItem(super::entity_factory::FloatingItemEntity),
    Particle(super::entity_factory::ParticleEntity),
}

impl Entity {
    pub fn get_id(&self) -> u32 {
        match self { 
            Entity::Player(e) => e.id, 
            Entity::Raft(e) => e.id,
            Entity::Fish(e) => e.id,
            Entity::Monster(e) => e.id,
            Entity::FloatingItem(e) => e.id,
            Entity::Particle(e) => e.id,
        }
    }
    pub fn get_entity_type(&self) -> EntityType {
        match self {
            Entity::Player(_) => EntityType::Player,
            Entity::Raft(_) => EntityType::Raft,
            Entity::Fish(_) => EntityType::Fish,
            Entity::Monster(_) => EntityType::Monster,
            Entity::FloatingItem(_) => EntityType::FloatingItem,
            Entity::Particle(_) => EntityType::Particle,
        }
    }
    pub fn get_position(&self) -> V2 {
        match self {
            Entity::Player(e) => e.player.pos.clone(),
            Entity::Raft(e) => e.raft.center.clone(),
            Entity::Fish(e) => e.position.clone(),
            Entity::Monster(e) => e.position.clone(),
            Entity::FloatingItem(e) => e.position.clone(),
            Entity::Particle(e) => e.position.clone(),
        }
    }
    pub fn set_position(&mut self, pos: V2) {
        match self {
            Entity::Player(e) => { e.player.pos = pos; }
            Entity::Raft(e) => { e.raft.center = pos; }
            Entity::Fish(e) => { e.position = pos; }
            Entity::Monster(e) => { e.position = pos; }
            Entity::FloatingItem(e) => { e.position = pos; }
            Entity::Particle(e) => { e.position = pos; }
        }
    }

    pub fn get_world_position(&self) -> Vec3 {
        match self {
            Entity::Player(e) => Vec3::new(e.player.pos.x, e.player.pos.y, if e.player.is_diving { e.player.depth as f32 } else { 0.0 }),
            Entity::Raft(e) => Vec3::new(e.raft.center.x, e.raft.center.y, 0.0),
            Entity::Fish(e) => Vec3::new(e.position.x, 0.0, e.position.y),
            Entity::Monster(e) => Vec3::new(e.position.x, 0.0, e.position.y),
            Entity::FloatingItem(e) => Vec3::new(e.position.x, e.position.y, 0.0),
            Entity::Particle(e) => Vec3::new(e.position.x, e.position.y, 0.0),
        }
    }
    
    pub fn get_velocity(&self) -> V2 {
        match self {
            Entity::Player(e) => e.player.vel.clone(),
            Entity::Raft(_e) => V2::zero(),
            Entity::Fish(e) => e.velocity.clone(),
            Entity::Monster(e) => e.velocity.clone(),
            Entity::FloatingItem(e) => e.velocity.clone(),
            Entity::Particle(e) => e.velocity.clone(),
        }
    }
    pub fn set_velocity(&mut self, vel: V2) {
        match self {
            Entity::Player(e) => { e.player.vel = vel; }
            Entity::Raft(_e) => {}
            Entity::Fish(e) => { e.velocity = vel; }
            Entity::Monster(e) => { e.velocity = vel; }
            Entity::FloatingItem(e) => { e.velocity = vel; }
            Entity::Particle(e) => { e.velocity = vel; }
        }
    }
    pub fn update(&mut self, delta_time: f32) {
        match self {
            Entity::Player(_e) => {},
            Entity::Raft(_e) => {},
            Entity::Fish(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.lifetime += delta_time;
                e.health.update(delta_time);
                e.stats.regenerate_stamina(delta_time);
            },
            Entity::Monster(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.health.update(delta_time);
                e.stats.regenerate_stamina(delta_time);
            },
            Entity::FloatingItem(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.lifetime += delta_time;
            },
            Entity::Particle(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.lifetime += delta_time;
                // gravity handled where needed; keep parity with previous
            },
        }
    }
    pub fn should_remove(&self) -> bool {
        match self {
            Entity::Fish(e) => !e.health.is_alive() || e.lifetime > 300.0,
            Entity::FloatingItem(e) => e.lifetime > 600.0,
            Entity::Particle(e) => e.lifetime > e.max_lifetime,
            _ => false,
        }
    }
    pub fn get_render_data(&self) -> RenderData {
        match self {
            Entity::Player(e) => {
                let mut data = e.render_data.clone();
                data.position = e.player.pos.clone();
                data
            }
            Entity::Raft(e) => {
                let mut data = e.render_data.clone();
                data.position = e.raft.center.clone();
                data
            }
            Entity::Fish(e) => {
                let mut data = e.render_data.clone();
                data.position = e.position.clone();
                data
            }
            Entity::Monster(e) => {
                let mut data = e.render_data.clone();
                data.position = e.position.clone();
                data
            }
            Entity::FloatingItem(e) => {
                let mut data = e.render_data.clone();
                data.position = e.position.clone();
                data
            }
            Entity::Particle(e) => {
                let mut data = e.render_data.clone();
                data.position = e.position.clone();
                data
            }
        }
    }
}

#[turbo::serialize]
pub struct HealthComponent {
    pub hp: f32,
    pub max_hp: f32,
}

impl HealthComponent {
    pub fn new(max_hp: f32) -> Self { Self { hp: max_hp, max_hp } }
    pub fn update(&mut self, _dt: f32) {}
    pub fn is_alive(&self) -> bool { self.hp > 0.0 }
}

#[turbo::serialize]
pub struct InventoryComponent {}

#[turbo::serialize]
pub struct StatsComponent {
    pub speed: f32,
    pub strength: f32,
    pub defense: f32,
    pub stamina: f32,
}

impl StatsComponent {
    pub fn new(speed: f32, strength: f32, defense: f32, stamina: f32) -> Self {
        Self { speed, strength, defense, stamina }
    }
    pub fn regenerate_stamina(&mut self, _dt: f32) {}
}


