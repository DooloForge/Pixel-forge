use crate::math::Vec3 as V3;
use crate::components::systems::*;
use crate::components::renderer::*;
use crate::components::input::*;
use crate::components::managers::*;
use crate::components::managers::scenes;
use crate::components::renderer::render_system::BackgroundLayer;
use crate::components::systems::spawn_system::SpawnType;
use crate::components::entities::{EntityManager, EntityStorage, EntityFactory};
use crate::models::player::Player;
use crate::models::raft::Raft;
use crate::models::ocean::Ocean;
use crate::models::particle::Particle;

/// Game state structure
#[derive(Default)]
#[turbo::serialize]
pub struct GameState {
    pub player: Option<Player>,
    pub raft: Option<Raft>,
    pub ocean: Option<Ocean>,
    pub particles: Vec<Particle>,
    pub player_entity_id: Option<u32>,
    pub raft_entity_id: Option<u32>,
    pub ui_mode: UiMode,
    pub game_mode: GameMode,
}

/// UI modes
#[derive(PartialEq, Default)]
#[turbo::serialize]
pub enum UiMode {
    #[default]
    Playing,
    Inventory,
    Crafting,
    Paused,
}

/// High-level gameplay mode switch
#[derive(Copy, PartialEq, Default)]
#[turbo::serialize]
pub enum GameMode {
    /// Raft/surface mode: top-down sailing and hooking floating materials
    #[default]
    Raft,
    /// Dive mode: underwater swimming and exploration
    Dive,
}

/// Scene types
#[derive(Copy, PartialEq)]
#[turbo::serialize]
pub enum SceneType {
    MainMenu,
    Playing,
    Inventory,
    Crafting,
    Paused,
}


/// Main game manager that coordinates all systems
#[turbo::serialize]
pub struct GameManager {
    // Systems
    pub(crate) physics_system: PhysicsSystem,
    pub(crate) spawn_system: SpawnSystem,
    pub(crate) world_system: WorldSystem,
    pub(crate) ai_system: AISystem,
    
    // Renderer
    pub(crate) render_system: RenderSystem,
    
    // Input
    pub(crate) input_system: InputSystem,
    
    // Managers
    pub(crate) scene_manager: SceneManager,
    pub(crate) resource_manager: ResourceManager,
    
    // Game state
    pub(crate) game_state: GameState,
    pub(crate) current_scene: SceneType,
    // Entities
    pub(crate) entity_manager: EntityManager,
    pub(crate) entity_storage: EntityStorage,
    pub(crate) entity_factory: EntityFactory,
    
    // Timing
    pub(crate) delta_time: f32,
    pub(crate) frame_count: u64,
}

impl GameManager {
    pub fn new() -> Self {
        let mut game_manager = Self {
            physics_system: PhysicsSystem::new(),
            spawn_system: SpawnSystem::new(),
            world_system: WorldSystem::new(12345), // Fixed seed for now
            ai_system: AISystem::new(),
            render_system: RenderSystem::new(),
            input_system: InputSystem::new(),
            scene_manager: SceneManager::new(),
            resource_manager: ResourceManager::new(),
            game_state: GameState { player_entity_id: None, raft_entity_id: None, ..GameState::default() },
            current_scene: SceneType::MainMenu,
            entity_manager: EntityManager::new(),
            entity_storage: EntityStorage::new(),
            entity_factory: EntityFactory::new(),
            delta_time: 1.0 / 60.0, // Assume 60 FPS
            frame_count: 0,
        };
        
        // Initialize systems
        game_manager.initialize_systems();
        
        game_manager
    }
    
    /// Initialize all systems
    fn initialize_systems(&mut self) {
        // Set up background layers
        self.render_system.add_background_layer(BackgroundLayer::SkyGradient);
        self.render_system.add_background_layer(BackgroundLayer::OceanGradient);
        self.render_system.add_background_layer(BackgroundLayer::WaterSurface);
        self.render_system.add_background_layer(BackgroundLayer::UnderwaterLighting);
        
        // Set up spawn system
        self.spawn_system.set_spawn_rate(SpawnType::FloatingItem, 300);
        self.spawn_system.set_spawn_rate(SpawnType::Fish, 180);
        self.spawn_system.set_spawn_rate(SpawnType::Bubble, 60);
        
        // Set up physics system
        self.physics_system.set_wind(V3::new(1.0, 0.0, 0.0), 0.5);
        // Add a broad, slow surface current so floats drift in raft mode
        self.physics_system.add_water_current(V3::new(0.0, 0.0, 0.0), V3::new(1.0, 0.0, 0.0), 0.6);
        // Mirror wind into spawner for directional edge spawns
        let wind = self.physics_system.get_wind();
        self.spawn_system.set_wind(wind);
    }
    
    /// Main update loop
    pub fn update(&mut self) {
        // Update input
        self.input_system.update();
        
        // Handle scene transitions
        self.handle_scene_transitions();
        
        // Update current scene (mutate game state only)
        match self.current_scene {
            SceneType::MainMenu => scenes::main_menu::update(self),
            SceneType::Playing => scenes::playing::update(self),
            SceneType::Inventory => scenes::inventory::update(self),
            SceneType::Crafting => scenes::crafting::update(self),
            SceneType::Paused => scenes::paused::update(self),
        }
        // Sync structs to entities
        if let Some(id) = self.game_state.player_entity_id {
            if let (Some(player), Some(entity)) = (self.game_state.player.as_ref(), self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, id)) {
                entity.set_world_position(player.pos.clone());
                entity.set_velocity(player.vel.clone());
            }
        }
        if let Some(id) = self.game_state.raft_entity_id {
            if let Some(raft) = self.game_state.raft.as_ref() {
                if let Some(entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, id) {
                    entity.set_world_position(raft.center.clone());
                }
            }
        }
        // Move raft world position with sea and optionally autopilot, and carry player if on raft
        let (player_on_raft, player_diving) = if let Some(p) = &self.game_state.player { (p.on_raft, p.is_diving) } else { (false, false) };
        if let Some(raft) = &mut self.game_state.raft {
            let cur = self.physics_system.get_water_current_at(&raft.center);
            let wind = self.physics_system.get_wind();
            // Slow tide-driven drift
            let drift = cur.scale(0.6).add(wind.scale(0.2));
            let delta = drift.scale(self.delta_time);
            raft.center = raft.center.add(delta);
            if player_on_raft {
                if let Some(p) = self.game_state.player.as_mut() {
                    p.pos = p.pos.add(delta);
                }
            }
        }
        // Apply simple environment to entities (water current drift for floats; gentle swim for fish)
        if let Some(player) = &self.game_state.player {
            // Floating items drift with water current + wind bias; despawn far away
            for id in self.entity_manager.get_entity_ids_by_type(crate::components::entities::game_entity::EntityType::FloatingItem) {
                if let Some(e) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, id) {
                    let pos = e.get_world_position();
                    let cur = self.physics_system.get_water_current_at(&pos);
                    let wind = self.physics_system.get_wind();
                    // Make floating items flow much faster from left to right
                    let base_flow = V3::new(6.0, 0.0, 0.0); // Much stronger left-to-right flow
                    let v = base_flow.add(cur.scale(0.2)).add(wind.scale(0.3));
                    e.set_velocity(v);
                }
            }
            // Fish drift with currents/wind
            for id in self.entity_manager.get_entity_ids_by_type(crate::components::entities::game_entity::EntityType::Fish) {
                if let Some(e) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, id) {
                    let pos = e.get_world_position();
                    let cur = self.physics_system.get_water_current_at(&pos);
                    let wind = self.physics_system.get_wind();
                    e.set_velocity(cur.add(wind.scale(0.2)));
                }
            }
            // Raft drifts slowly with surface current in Raft mode
            if self.game_state.game_mode == GameMode::Raft {
                if let Some(raft_id) = self.game_state.raft_entity_id {
                    if let Some(raft_entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, raft_id) {
                        let cur = self.physics_system.get_water_current_at(&raft_entity.get_world_position());
                        raft_entity.set_velocity(cur.scale(0.3));
                    }
                }
            }
            // Despawn floating items that drift too far from the raft/player
            let mut to_remove: Vec<u32> = Vec::new();
            let raft_pos_opt = self.game_state.raft.as_ref().map(|r| r.center.clone());
            for id in self.entity_manager.get_entity_ids_by_type(crate::components::entities::game_entity::EntityType::FloatingItem) {
                if let Some(e) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, id) {
                    let pos = e.get_world_position();
                    let mut too_far = pos.distance_to(&player.pos) > 800.0;
                    if let Some(raft_pos) = &raft_pos_opt {
                        if pos.distance_to(raft_pos) > 800.0 {
                            too_far = true;
                        }
                    }
                    if too_far { to_remove.push(id); }
                }
            }
            for id in to_remove { let _ = self.entity_manager.remove_entity(&mut self.entity_storage, id); }
        }
        
        // Update hook system
        let player_pos = self.game_state.player.as_ref().map(|p| p.pos.clone());
        if let Some(pos) = player_pos {
            self.update_hooks(&pos, self.delta_time);
        }
        
        // Update-render entities
        self.entity_manager.update_entities(&mut self.entity_storage, self.delta_time);
        for entity in self.entity_manager.get_all_entities(&self.entity_storage) {
            self.render_system.add_entity(entity);
        }
        // Render world then UI once per frame after scene update
        self.render_system.render();
        self.render_ui();
        
        // Update frame count
        self.frame_count += 1;
    }
    
    /// Handle scene transitions based on input
    fn handle_scene_transitions(&mut self) {
        let input_state = self.input_system.get_input_state();
        
        match self.current_scene {
            SceneType::MainMenu => {
                if input_state.use_tool {
                    self.current_scene = SceneType::Playing;
                    self.initialize_playing_scene();
                }
            },
            SceneType::Playing => {
                if input_state.open_inventory {
                    self.current_scene = SceneType::Inventory;
                } else if input_state.open_crafting {
                    self.current_scene = SceneType::Crafting;
                }
            },
            SceneType::Inventory => {
                if input_state.open_inventory {
                    self.current_scene = SceneType::Playing;
                }
            },
            SceneType::Crafting => {
                if input_state.open_crafting {
                    self.current_scene = SceneType::Playing;
                }
            },
            SceneType::Paused => {
                // Handle pause menu
            },
        }
    }
    
    /// Initialize playing scene
    fn initialize_playing_scene(&mut self) {
        // Create player if not exists
        if self.game_state.player.is_none() {
            let player = Player::new(V3::new(0.0, 0.0, 0.0));
            self.game_state.player = Some(player);
        }
        
        // Create raft if not exists
        if self.game_state.raft.is_none() {
            let raft = Raft::new(V3::new(0.0, 0.0, 0.0));
            self.game_state.raft = Some(raft);
        }
        
        // Create ocean if not exists
        if self.game_state.ocean.is_none() {
            let ocean = Ocean::new();
            self.game_state.ocean = Some(ocean);
        }
        
        // Create entities and initialize camera to center on player
        if let Some(player) = &self.game_state.player {
            if self.game_state.player_entity_id.is_none() {
                let e = self.entity_factory.create_player(player.pos.clone());
                let id = self.entity_manager.create_entity(&mut self.entity_storage, e);
                self.game_state.player_entity_id = Some(id);
            }
            self.render_system.set_camera_target(player.pos);
            self.render_system.update_camera(0.0); // Immediate update
        }
        if let Some(raft) = &self.game_state.raft {
            if self.game_state.raft_entity_id.is_none() {
                let e = self.entity_factory.create_raft(raft.center.clone());
                let id = self.entity_manager.create_entity(&mut self.entity_storage, e);
                self.game_state.raft_entity_id = Some(id);
            }
        }

        // No static seeding; items will spawn over time near the raft
    }
    
    // Scene-specific update functions are now in managers::scenes::* modules
    
    /// Update AI for all entities
    pub(crate) fn update_ai(&mut self) {
        // TODO: Get all AI entities and update them
    }
    
    /// Update spawning (internal version that takes extracted values)
    pub(crate) fn update_spawning_internal(&mut self, player_pos: &V3) {
        // Get current entity counts from entity manager
        let mut current_counts = std::collections::HashMap::new();
        let floats = self.entity_manager.get_entity_count(crate::components::entities::game_entity::EntityType::FloatingItem);
        let fish = self.entity_manager.get_entity_count(crate::components::entities::game_entity::EntityType::Fish);
        current_counts.insert(SpawnType::FloatingItem, floats);
        current_counts.insert(SpawnType::Fish, fish);
        current_counts.insert(SpawnType::Bubble, 0);
        
        // Update spawn system
        // Keep wind in sync
        self.spawn_system.set_wind(self.physics_system.get_wind());
        self.spawn_system.update(player_pos, &current_counts);
        // Consume pending spawns and create entities
        for (stype, pos) in self.spawn_system.drain_pending() {
            match stype {
                SpawnType::FloatingItem => {
                    let item_type = self.get_random_floating_item_type();
                    let item = self.entity_factory.create_floating_item(pos.clone(), item_type);
                    let _ = self.entity_manager.create_entity(&mut self.entity_storage, item);
                }
                SpawnType::Fish => {
                    let fish = self.entity_factory.create_fish(pos.clone(), crate::components::entities::entity_factory::FishType::SmallFish);
                    let _ = self.entity_manager.create_entity(&mut self.entity_storage, fish);
                }
                _ => {}
            }
        }

        // No event bus; handled via drain_pending above
    }
    
    /// Get a random floating item type based on rarity
    fn get_random_floating_item_type(&self) -> crate::models::ocean::FloatingItemType {
        use crate::models::ocean::FloatingItemType;
        use turbo::random;
        
        let rand = random::f32();
        let mut cumulative = 0.0;
        
        let item_types = [
            FloatingItemType::Wood,
            FloatingItemType::Plastic,
            FloatingItemType::Rope,
            FloatingItemType::Metal,
            FloatingItemType::Nail,
            FloatingItemType::Cloth,
            FloatingItemType::Barrel,
            FloatingItemType::Coconut,
            FloatingItemType::Fish,
            FloatingItemType::Seaweed,
            FloatingItemType::Treasure,
            FloatingItemType::Bottle,
        ];
        
        for item_type in item_types.iter() {
            cumulative += item_type.rarity();
            if rand <= cumulative {
                return *item_type;
            }
        }
        
        // Fallback to wood if something goes wrong
        FloatingItemType::Wood
    }
    
    /// Handle hook launching
    pub fn launch_hook(&mut self, player_pos: &V3, direction: crate::math::Vec2) {
        // Check if player already has an active hook
        let has_active_hook = self.entity_manager.get_entity_ids_by_type(crate::components::entities::game_entity::EntityType::Hook)
            .iter()
            .any(|&hook_id| {
                if let Some(entity) = self.entity_manager.get_entity(&self.entity_storage, hook_id) {
                    if let crate::components::entities::game_entity::Entity::Hook(hook_entity) = entity {
                        return hook_entity.hook.is_active();
                    }
                }
                false
            });
        
        if !has_active_hook {
            // Create new hook entity
            let hook = self.entity_factory.create_hook(0); // TODO: Use actual player ID
            let hook_id = self.entity_manager.create_entity(&mut self.entity_storage, hook);
            
            // Launch the hook
            if let Some(entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, hook_id) {
                if let crate::components::entities::game_entity::Entity::Hook(hook_entity) = entity {
                    hook_entity.hook.launch(*player_pos, direction);
                    hook_entity.player_pos = *player_pos; // Store player position for line rendering
                }
            }
        }
    }
    
    /// Update hook system
    pub fn update_hooks(&mut self, player_pos: &V3, delta_time: f32) {
        let mut hooks_to_remove = Vec::new();
        let mut collected_items = Vec::new();
        
        // First, collect all item positions to avoid borrowing conflicts
        let item_positions: Vec<(u32, V3)> = self.entity_manager.get_entity_ids_by_type(crate::components::entities::game_entity::EntityType::FloatingItem)
            .into_iter()
            .filter_map(|item_id| {
                if let Some(item_entity) = self.entity_manager.get_entity(&self.entity_storage, item_id) {
                    Some((item_id, item_entity.get_world_position()))
                } else {
                    None
                }
            })
            .collect();
        
        // Get all hook IDs first to avoid borrowing conflicts
        let hook_ids: Vec<u32> = self.entity_manager.get_entity_ids_by_type(crate::components::entities::game_entity::EntityType::Hook);
        
        for hook_id in hook_ids {
            // We'll compute any pinning we need to do outside the hook's mutable borrow
            let mut pin_request: Option<(Vec<u32>, V3)> = None;

            if let Some(entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, hook_id) {
                if let crate::components::entities::game_entity::Entity::Hook(hook_entity) = entity {
                    // Update hook physics
                    let hook_completed = hook_entity.hook.update(delta_time, *player_pos);
                    
                    if hook_completed {
                        // Hook has returned, collect attached items
                        let attached_items = hook_entity.hook.detach_all_items();
                        collected_items.extend(attached_items);
                        hooks_to_remove.push(hook_id);
                    } else {
                        // Check for item collisions during hook travel
                        let hook_tip_pos = hook_entity.hook.get_hook_tip_position();
                        
                        // Check collisions with each item using pre-collected positions
                        for (item_id, item_pos) in &item_positions {
                            let distance = hook_tip_pos.distance_to(item_pos);
                            
                            if distance <= 15.0 { // Hook collision range
                                hook_entity.hook.attach_item(*item_id);
                            }
                        }

                        // Clone attached items so we can move them after dropping the hook borrow
                        let attached_ids = hook_entity.hook.attached_items.clone();
                        pin_request = Some((attached_ids, hook_tip_pos));
                    }
                }
            }

            // If we have items attached to this hook, pin them to the hook tip visually
            if let Some((attached_ids, hook_tip_pos)) = pin_request {
                for (_i, item_id) in attached_ids.into_iter().enumerate() {
                    if let Some(item_entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, item_id) {
                        // Pin exactly at the hook tip to appear stuck to the head
                        let pin_pos = V3::new(hook_tip_pos.x, hook_tip_pos.y, hook_tip_pos.z);
                        item_entity.set_world_position(pin_pos);
                        item_entity.set_velocity(V3::zero());
                    }
                }
            }
        }
        
        // Remove completed hooks
        for hook_id in hooks_to_remove {
            let _ = self.entity_manager.remove_entity(&mut self.entity_storage, hook_id);
        }
        
        // Collect items that were attached to hooks
        for item_id in collected_items {
            if let Some(entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, item_id) {
                if let crate::components::entities::game_entity::Entity::FloatingItem(item_entity) = entity {
                    let item_type = item_entity.item_type;
                    
                    // Add to player inventory
                    if let Some(player) = &mut self.game_state.player {
                        if player.inventory.add_material(item_type, 1) {
                            // Successfully added to inventory, remove the entity
                            let _ = self.entity_manager.remove_entity(&mut self.entity_storage, item_id);
                        }
                    }
                }
            }
        }
    }
    
    /// Handle item collection mechanics (legacy method for manual collection)
    pub fn handle_item_collection(&mut self, player_pos: &V3, use_hook: bool) {
        if use_hook {
            // Use hook system instead
            return;
        }
        
        let collection_range = 20.0; // Manual collection range
        
        let mut items_to_collect = Vec::new();
        
        // Find nearby floating items
        for id in self.entity_manager.get_entity_ids_by_type(crate::components::entities::game_entity::EntityType::FloatingItem) {
            if let Some(entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, id) {
                let item_pos = entity.get_world_position();
                let distance = player_pos.distance_to(&item_pos);
                
                if distance <= collection_range {
                    items_to_collect.push(id);
                }
            }
        }
        
        // Collect the items
        for item_id in items_to_collect {
            if let Some(entity) = self.entity_manager.get_entity_mut_by_id(&mut self.entity_storage, item_id) {
                // Get the item type from the entity
                if let crate::components::entities::game_entity::Entity::FloatingItem(item_entity) = entity {
                    let item_type = item_entity.item_type;
                    
                    // Add to player inventory
                    if let Some(player) = &mut self.game_state.player {
                        if player.inventory.add_material(item_type, 1) {
                            // Successfully added to inventory, remove the entity
                            let _ = self.entity_manager.remove_entity(&mut self.entity_storage, item_id);
                        }
                    }
                }
            }
        }
    }
    
    /// Render UI/HUD elements
    pub fn render_ui(&mut self) {
        // Create UI renderer if needed
        let mut ui_renderer = crate::components::renderer::UIRenderer::new();
        
        // Set UI mode based on current scene
        match self.current_scene {
            SceneType::Playing => ui_renderer.set_ui_mode(crate::components::renderer::ui_renderer::UIMode::Playing),
            SceneType::Inventory => ui_renderer.set_ui_mode(crate::components::renderer::ui_renderer::UIMode::Inventory),
            SceneType::Crafting => ui_renderer.set_ui_mode(crate::components::renderer::ui_renderer::UIMode::Crafting),
            SceneType::Paused => ui_renderer.set_ui_mode(crate::components::renderer::ui_renderer::UIMode::Paused),
            _ => ui_renderer.set_ui_mode(crate::components::renderer::ui_renderer::UIMode::Playing),
        }

        // Feed HUD from authoritative GameState
        if let Some(player) = &self.game_state.player {
            let tool_name = match player.current_tool { 
                crate::models::player::Tool::Hook => "Hook",
                crate::models::player::Tool::Builder => "Builder",
                crate::models::player::Tool::Axe => "Axe",
                crate::models::player::Tool::Hammer => "Hammer",
            }.to_string();
            let status = if player.is_diving { "Diving" } else if player.on_raft { "On Raft" } else { "Swimming" }.to_string();
            let player_pos_str = Some(format!("Player: ({:.1}, {:.1}, {:.1})", player.pos.x, player.pos.y, player.pos.z));
            let raft_pos_str = self.game_state.raft.as_ref().map(|r| format!("Raft: ({:.1}, {:.1}, {:.1})", r.center.x, r.center.y, r.center.z));
            ui_renderer.set_hud_state(crate::components::renderer::ui_renderer::HudState {
                tool: tool_name,
                health: player.health,
                hunger: player.hunger,
                thirst: player.thirst,
                status,
                player_pos: player_pos_str,
                raft_pos: raft_pos_str,
            });
        }

        // Minimap: project nearby entities relative to player
        let mut points: Vec<crate::components::renderer::ui_renderer::MinimapPoint> = Vec::new();
        let center = (40.0, 40.0);
        let scale = 0.1; // world units to minimap pixels
        let minimap_range = crate::constants::MINIMAP_RANGE; // Only show entities within range of player
        if let Some(player) = &self.game_state.player {
            // Player at center
            points.push(crate::components::renderer::ui_renderer::MinimapPoint { x: center.0, y: center.1, size: 3.0, color: crate::constants::PLAYER_ON_RAFT_COLOR });
            for entity in self.entity_manager.get_all_entities(&self.entity_storage) {
                let ety = crate::components::entities::game_entity::Entity::get_entity_type(entity);
                let pos = crate::components::entities::game_entity::Entity::get_world_position(entity);
                
                // Calculate distance from player
                let distance = ((pos.x - player.pos.x).powi(2) + (pos.y - player.pos.y).powi(2)).sqrt();
                
                // Only show entities within minimap range
                if distance <= minimap_range {
                    let dx = (pos.x - player.pos.x) * scale;
                    let dy = (pos.y - player.pos.y) * scale;
                    let x = (center.0 + dx).clamp(4.0, 76.0);
                    let y = (center.1 + dy).clamp(4.0, 76.0);
                    let (size, color) = match ety {
                        crate::components::entities::game_entity::EntityType::FloatingItem => (2.0, 0xFFFF00FF),
                        crate::components::entities::game_entity::EntityType::Fish => (2.0, 0x00FFFFFF),
                        crate::components::entities::game_entity::EntityType::Raft => (3.0, crate::constants::RAFT_WOOD_FLOOR_COLOR),
                        crate::components::entities::game_entity::EntityType::Monster => (3.0, 0xFF4444FF),
                        crate::components::entities::game_entity::EntityType::Particle => (1.0, 0x888888FF),
                        _ => (1.0, 0xFFFFFFFF),
                    };
                    points.push(crate::components::renderer::ui_renderer::MinimapPoint { x, y, size, color });
                }
            }
        }
        ui_renderer.set_minimap_points(points);
        
        // Render the UI
        ui_renderer.render();
    }
}

/// Apply player input directly (no self borrowing)
pub(crate) fn apply_player_input(player: &mut Player, input_state: &crate::components::input::input_system::InputState, movement: &V3) {
    // Tool switching
    if input_state.switch_tool {
        player.switch_tool();
    }
    
    // Movement: raft vs swim vs dive
    if player.on_raft {
        // Raft mode: slower on-raft movement; separate sailing inputs can be applied to raft
        let move_speed = 1.0;
        player.pos.x += movement.x * move_speed;
        player.pos.y += movement.y * move_speed;
    } else if player.is_diving {
        // Dive mode: horizontal is x, vertical is depth (z). Do NOT change world y while diving
        let move_speed = 2.0;
        player.pos.x += movement.x * move_speed;
        player.pos.z += movement.y * -move_speed; // up input (negative y) should reduce depth (towards 0)
    } else {
        // Top-down swim outside raft: move in x/y plane
        let move_speed = 2.0;
        player.pos.x += movement.x * move_speed;
        player.pos.y += movement.y * move_speed;
    }
    
    // on_raft is determined by the caller (uses top-down position when in Dive)
    
    if input_state.eat_food {
        if player.inventory.remove_material(crate::models::ocean::FloatingItemType::Coconut, 1) {
            player.eat_food(crate::models::ocean::FloatingItemType::Coconut);
        }
    }
    
    player.update_cooldowns();
}

/// Apply physics update directly (no self borrowing)
pub(crate) fn apply_physics_update(player: &mut Player, water_current: &V3, delta_time: f32) {
    if !player.on_raft {
        // Swimmer is fixed against tide: no passive drift from water current
        player.vel = V3::zero();
        // Position changes only via input handling
    }
}
