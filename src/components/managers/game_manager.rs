use crate::math::Vec2 as V2;
use crate::components::systems::*;
use crate::components::renderer::*;
use crate::components::input::*;
use crate::components::managers::*;
use crate::components::managers::scenes;
use crate::components::renderer::render_system::BackgroundLayer;
use crate::components::systems::spawn_system::SpawnType;
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
    pub ui_mode: UiMode,
    pub game_mode: GameMode,
    // Store top-down state to restore after surfacing
    pub last_surface_pos: V2,
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
            game_state: GameState {..GameState::default() },
            current_scene: SceneType::MainMenu,
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
        self.physics_system.set_wind(V2::new(1.0, 0.0), 0.5);
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
            let player = Player::new(V2::new(0.0, 0.0));
            self.game_state.player = Some(player);
        }
        
        // Create raft if not exists
        if self.game_state.raft.is_none() {
            let raft = Raft::new(V2::new(0.0, 0.0));
            self.game_state.raft = Some(raft);
        }
        
        // Create ocean if not exists
        if self.game_state.ocean.is_none() {
            let ocean = Ocean::new();
            self.game_state.ocean = Some(ocean);
        }
        
        // Initialize camera to center on player
        if let Some(player) = &self.game_state.player {
            self.render_system.set_camera_target(player.pos);
            self.render_system.update_camera(0.0); // Immediate update
        }
    }
    
    // Scene-specific update functions are now in managers::scenes::* modules
    
    /// Update AI for all entities
    pub(crate) fn update_ai(&mut self) {
        // TODO: Get all AI entities and update them
    }
    
    /// Update spawning (internal version that takes extracted values)
    pub(crate) fn update_spawning_internal(&mut self, player_pos: &V2) {
        // Get current entity counts
        let mut current_counts = std::collections::HashMap::new();
        current_counts.insert(SpawnType::FloatingItem, 0); // TODO: Get actual count
        current_counts.insert(SpawnType::Fish, 0); // TODO: Get actual count
        current_counts.insert(SpawnType::Bubble, 0); // TODO: Get actual count
        
        // Update spawn system
        self.spawn_system.update(player_pos, &current_counts);
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
        
        // Render the UI
        ui_renderer.render();
    }
}

/// Apply player input directly (no self borrowing)
pub(crate) fn apply_player_input(player: &mut Player, input_state: &crate::components::input::input_system::InputState, movement: &V2) {
    // Tool switching
    if input_state.switch_tool {
        player.switch_tool();
    }
    
    // Movement: raft vs dive
    if player.on_raft {
        // Raft mode: slower on-raft movement; separate sailing inputs can be applied to raft
        let move_speed = 1.0;
        player.pos.x += movement.x * move_speed;
        player.pos.y += movement.y * move_speed;
    } else {
        // Top-down swim outside raft: no clamp
        let move_speed = 2.0;
        player.pos.y += movement.y * move_speed;
        player.pos.x += movement.x * move_speed;
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
pub(crate) fn apply_physics_update(player: &mut Player, water_current: &V2, delta_time: f32) {
    if !player.on_raft {
        player.vel = player.vel.add(water_current.scale(delta_time));
        player.pos = player.pos.add(player.vel.clone());
    }
}
