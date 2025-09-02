use crate::math::Vec2 as V2;

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

pub trait GameEntity {
    fn get_id(&self) -> u32;
    fn get_entity_type(&self) -> EntityType;
    fn get_position(&self) -> V2;
    fn set_position(&mut self, pos: V2);
    fn get_velocity(&self) -> V2;
    fn set_velocity(&mut self, vel: V2);
    fn update(&mut self, delta_time: f32);
    fn should_remove(&self) -> bool;
    fn get_render_data(&self) -> RenderData;
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


