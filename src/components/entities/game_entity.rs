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
    pub screen_position: Option<(f32, f32)>,
    pub world_position: Vec3,
    pub size: f32,
    pub color: u32,
    pub visible: bool,
    pub layer: RenderLayer,
    pub player_is_moving: bool,
    pub player_last_movement: Vec3,
    pub player_on_raft: bool,
}

impl RenderData {
    pub fn new(world_position: Vec3, size: f32, color: u32) -> Self {
        Self { 
            screen_position: None, 
            world_position, 
            size, 
            color, 
            visible: true, 
            layer: RenderLayer::Entity,
            player_is_moving: false,
            player_last_movement: Vec3::zero(),
            player_on_raft: false,
        }
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
    Hook,
}

#[turbo::serialize]
pub enum Entity {
    Player(super::entity_factory::PlayerEntity),
    Raft(super::entity_factory::RaftEntity),
    Fish(super::entity_factory::FishEntity),
    Monster(super::entity_factory::MonsterEntity),
    FloatingItem(super::entity_factory::FloatingItemEntity),
    Particle(super::entity_factory::ParticleEntity),
    Hook(super::entity_factory::HookEntity),
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
            Entity::Hook(e) => e.id,
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
            Entity::Hook(_) => EntityType::Hook,
        }
    }
    pub fn get_world_position(&self) -> Vec3 {
        match self {
            Entity::Player(e) => e.player.pos.clone(),
            Entity::Raft(e) => e.raft.center.clone(),
            Entity::Fish(e) => e.position.clone(),
            Entity::Monster(e) => e.position.clone(),
            Entity::FloatingItem(e) => e.position.clone(),
            Entity::Particle(e) => e.position.clone(),
            Entity::Hook(e) => e.hook.position.clone(),
        }
    }
    pub fn set_world_position(&mut self, pos: Vec3) {
        match self {
            Entity::Player(e) => { 
                e.player.pos = pos;
                e.render_data.world_position = pos;
            }
            Entity::Raft(e) => { 
                e.raft.center = pos;
                e.render_data.world_position = pos;
            }
            Entity::Fish(e) => { 
                e.position = pos;
                e.render_data.world_position = pos;
            }
            Entity::Monster(e) => { 
                e.position = pos;
                e.render_data.world_position = pos;
            }
            Entity::FloatingItem(e) => { 
                e.position = pos;
                e.render_data.world_position = pos;
            }
            Entity::Particle(e) => { 
                e.position = pos;
                e.render_data.world_position = pos;
            }
            Entity::Hook(e) => { 
                e.hook.position = pos;
                e.render_data.world_position = pos;
            }
        }
    }

    pub fn get_render_data(&self) -> RenderData {
        match self {
            Entity::Player(e) => e.render_data.clone(),
            Entity::Raft(e) => e.render_data.clone(),
            Entity::Fish(e) => e.render_data.clone(),
            Entity::Monster(e) => e.render_data.clone(),
            Entity::FloatingItem(e) => e.render_data.clone(),
            Entity::Particle(e) => e.render_data.clone(),
            Entity::Hook(e) => e.render_data.clone(),
        }
    }
    
    pub fn update_render_data(&mut self, render_data: RenderData) {
        match self {
            Entity::Player(e) => { e.render_data = render_data; }
            Entity::Raft(e) => { e.render_data = render_data; }
            Entity::Fish(e) => { e.render_data = render_data; }
            Entity::Monster(e) => { e.render_data = render_data; }
            Entity::FloatingItem(e) => { e.render_data = render_data; }
            Entity::Particle(e) => { e.render_data = render_data; }
            Entity::Hook(e) => { e.render_data = render_data; }
        }
    }

    pub fn get_velocity(&self) -> Vec3 {
        match self {
            Entity::Player(e) => e.player.vel.clone(),
            Entity::Raft(_e) => Vec3::zero(),
            Entity::Fish(e) => e.velocity.clone(),
            Entity::Monster(e) => e.velocity.clone(),
            Entity::FloatingItem(e) => e.velocity.clone(),
            Entity::Particle(e) => e.velocity.clone(),
            Entity::Hook(e) => e.hook.velocity.clone(),
        }
    }
    pub fn set_velocity(&mut self, vel: Vec3) {
        match self {
            Entity::Player(e) => { e.player.vel = vel; }
            Entity::Raft(_e) => {}
            Entity::Fish(e) => { e.velocity = vel; }
            Entity::Monster(e) => { e.velocity = vel; }
            Entity::FloatingItem(e) => { e.velocity = vel; }
            Entity::Particle(e) => { e.velocity = vel; }
            Entity::Hook(e) => { e.hook.velocity = vel; }
        }
    }
    pub fn update(&mut self, delta_time: f32) {
        match self {
            Entity::Player(e) => {
                // only update this for raft rendering distancing effect
                e.render_data.world_position = e.player.pos.clone();
            },
            Entity::Raft(_e) => {},
            Entity::Fish(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.lifetime += delta_time;
                e.health.update(delta_time);
                e.stats.regenerate_stamina(delta_time);
                // Despawn after flowing a certain distance from origin
                if e.position.distance_to(&e.spawn_origin) > 1200.0 {
                    e.health.hp = 0.0; // mark for removal via is_alive/lifetime checks
                }
            },
            Entity::Monster(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.health.update(delta_time);
                e.stats.regenerate_stamina(delta_time);
            },
            Entity::FloatingItem(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.lifetime += delta_time;
                if e.position.distance_to(&e.spawn_origin) > 1600.0 {
                    e.lifetime = 10000.0; // exceed removal threshold
                }
            },
            Entity::Particle(e) => {
                e.position = e.position.add(e.velocity.scale(delta_time));
                e.lifetime += delta_time;
                // gravity handled where needed; keep parity with previous
            },
            Entity::Hook(e) => {
                // Hook update is handled in the hook system, not here
                // Just update render position
                e.render_data.world_position = e.hook.position.clone();
            },
        }
    }
    pub fn should_remove(&self) -> bool {
        match self {
            Entity::Fish(e) => !e.health.is_alive() || e.lifetime > 300.0,
            Entity::FloatingItem(e) => e.lifetime > 600.0,
            Entity::Particle(e) => e.lifetime > e.max_lifetime,
            Entity::Hook(e) => !e.hook.is_active(), // Remove when hook is retracted
            _ => false,
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


