use super::*;
use crate::math::Vec3;
use crate::components::entities::game_entity::{Entity, EntityType, RenderData, RenderLayer};
// CameraSystem removed; use turbo camera API directly
// use crate::constants::*;

/// Handles all game rendering
#[turbo::serialize]
pub struct RenderSystem {
    camera_pos: (f32, f32),
    render_queue: Vec<RenderCommand>,
    background_layers: Vec<BackgroundLayer>,
    view_mode: RenderViewMode,
    transition_alpha: f32,
    last_player_world_pos: Option<Vec3>,
}

impl RenderSystem {
    pub fn new() -> Self {
        Self {
            camera_pos: (0.0, 0.0),
            render_queue: Vec::new(),
            background_layers: Vec::new(),
            view_mode: RenderViewMode::TopDown,
            transition_alpha: 0.0,
            last_player_world_pos: None,
        }
    }
    
    /// Set camera target from world position; compute screen-plane y based on view mode
    pub fn set_camera_target(&mut self, world: Vec3) {
        let cam_y = match self.view_mode {
            RenderViewMode::TopDown => world.y,
            RenderViewMode::SideScroll => -world.z,
        };
        self.camera_pos = (world.x, cam_y);
        camera::set_xy(self.camera_pos.0, self.camera_pos.1);
    }
    
    /// Update camera
    pub fn update_camera(&mut self, delta_time: f32) {
        // No smoothing; camera already set via set_camera_target
        if self.transition_alpha > 0.0 {
            self.transition_alpha = (self.transition_alpha - delta_time * 2.0).max(0.0);
        }
    }
    
    /// Add entity to render queue
    pub fn add_entity(&mut self, entity: &Entity) {
        let mut render_data = entity.get_render_data();
        // Project world position into current view
        let world_pos = entity.get_world_position();
        render_data.screen_position = match self.view_mode {
            RenderViewMode::TopDown => Some((world_pos.x, world_pos.y)),
            RenderViewMode::SideScroll => Some((world_pos.x, -world_pos.z)),
        };
        if render_data.visible {
            let command = RenderCommand::Entity {
                data: render_data.clone(),
                entity_type: entity.get_entity_type(),
            };
            self.render_queue.push(command);
        }
    }
    
    /// Add background layer
    pub fn add_background_layer(&mut self, layer: BackgroundLayer) {
        self.background_layers.push(layer);
    }
    
    /// Render everything
    pub fn render(&mut self) {
        let camera_pos = (self.camera_pos.0, self.camera_pos.1);
        let (screen_w, screen_h) = resolution();
        
        // Cache player world position (if present) for distance-based effects
        self.last_player_world_pos = None;
        for command in &self.render_queue {
            if let RenderCommand::Entity { data, entity_type } = command {
                if let EntityType::Player = entity_type {
                    self.last_player_world_pos = Some(data.world_position.clone());
                    break;
                }
            }
        }

        log!("Last player pos: {:?}", self.last_player_world_pos);
        
        // Clear screen
        self.clear_screen();
        
        // Sort render queue by layer
        self.render_queue.sort_by(|a, b| {
            let layer_a = match a {
                RenderCommand::Entity { data, .. } => data.layer,
                RenderCommand::Background { layer, .. } => *layer,
                RenderCommand::UI { layer, .. } => *layer,
            };
            let layer_b = match b {
                RenderCommand::Entity { data, .. } => data.layer,
                RenderCommand::Background { layer, .. } => *layer,
                RenderCommand::UI { layer, .. } => *layer,
            };
            // Force player and raft to render on top
            let bias = |layer: RenderLayer, cmd: &RenderCommand| -> i32 {
                match cmd {
                    RenderCommand::Entity { entity_type, .. } => match entity_type {
                        EntityType::Player => 2,
                        EntityType::Raft => 1,
                        _ => 0,
                    },
                    _ => 0,
                }
            };
            let ord = layer_a.cmp(&layer_b);
            if ord == std::cmp::Ordering::Equal {
                let ba = bias(layer_a, a);
                let bb = bias(layer_b, b);
                ba.cmp(&bb)
            } else { ord }
        });
        
        // Render background layers
        self.render_background_layers(camera_pos, screen_w, screen_h);
        
        // Render entities
        self.render_entities(camera_pos, screen_w, screen_h);
        
        // Fade overlay
        if self.transition_alpha > 0.0 {
            let alpha = (self.transition_alpha * 255.0) as u32;
            let color = (0x00 << 24) | (0x00 << 16) | (0x00 << 8) | alpha;
            rect!(x = 0.0, y = 0.0, w = screen_w as f32, h = screen_h as f32, color = color, fixed = true);
        }
        
        // Clear render queue
        self.render_queue.clear();
    }

    pub fn set_render_mode(&mut self, mode: RenderViewMode) {
        self.view_mode = mode;
    }

    pub fn trigger_transition_fade(&mut self) {
        self.transition_alpha = 1.0;
    }
    
    /// Clear the screen
    fn clear_screen(&self) {
        // Clear with deep underwater blue
        clear(0x001122FF);
    }
    
    /// Render background layers
    fn render_background_layers(&self, camera_pos: (f32, f32), screen_w: u32, screen_h: u32) {
        // In TopDown mode, draw a full-screen ocean background
        if let RenderViewMode::TopDown = self.view_mode {
            self.render_ocean_fullscreen(screen_w, screen_h);
            return;
        }
        // SideScroll and others: layered backgrounds
        for layer in &self.background_layers {
            match layer {
                BackgroundLayer::SkyGradient => self.render_sky_gradient(camera_pos, screen_w, screen_h),
                BackgroundLayer::OceanGradient => self.render_ocean_gradient(camera_pos, screen_w, screen_h),
                BackgroundLayer::WaterSurface => self.render_water_surface(camera_pos, screen_w, screen_h),
                BackgroundLayer::UnderwaterLighting => self.render_underwater_lighting(screen_w, screen_h),
            }
        }
    }
    
    /// Render sky gradient
    fn render_sky_gradient(&self, camera_pos: (f32, f32), screen_w: u32, screen_h: u32) {
        for y in 0..screen_h {
            let screen_y = y as f32;
            let world_y = camera_pos.1 + (screen_y - screen_h as f32 * 0.5);
            
            if world_y < 0.0 {
                // Above sea level - sky that gets darker when viewed from depth
                let view_depth_factor = (camera_pos.1 / 200.0).clamp(0.0, 0.8);
                let sky_brightness = 1.0 - view_depth_factor;
                let sky_r = (0x87 as f32 * sky_brightness) as u32;
                let sky_g = (0xCE as f32 * sky_brightness) as u32;
                let sky_b = (0xEB as f32 * sky_brightness) as u32;
                let sky_color = (sky_r << 24) | (sky_g << 16) | (sky_b << 8) | 0xFF;
                
                rect!(
                    x = 0.0,
                    y = screen_y,
                    w = screen_w as f32,
                    h = 1.0,
                    color = sky_color,
                    fixed = true
                );
            }
        }
    }
    
    /// Render ocean gradient
    fn render_ocean_gradient(&self, camera_pos: (f32, f32), screen_w: u32, screen_h: u32) {
        for y in 0..screen_h {
            let screen_y = y as f32;
            let world_y = camera_pos.1 + (screen_y - screen_h as f32 * 0.5);
            
            if world_y >= 0.0 {
                // Below sea level - underwater that gets darker with depth
                let depth_factor = (world_y / 400.0).clamp(0.0, 1.0);
                let ocean_brightness = 1.0 - (depth_factor * 0.9);
                let ocean_r = (0x41 as f32 * ocean_brightness) as u32;
                let ocean_g = (0x69 as f32 * ocean_brightness) as u32;
                let ocean_b = (0xE1 as f32 * ocean_brightness) as u32;
                let ocean_color = (ocean_r << 24) | (ocean_g << 16) | (ocean_b << 8) | 0xFF;
                
                rect!(
                    x = 0.0,
                    y = screen_y,
                    w = screen_w as f32,
                    h = 1.0,
                    color = ocean_color,
                    fixed = true
                );
            }
        }
    }
    
    /// Render water surface
    fn render_water_surface(&self, camera_pos: (f32, f32), screen_w: u32, screen_h: u32) {
        let water_surface_screen_y = -camera_pos.1 + screen_h as f32 * 0.5;
        
        if water_surface_screen_y >= -10.0 && water_surface_screen_y <= screen_h as f32 + 10.0 {
            for x in 0..screen_w as i32 {
                let world_x = (x as f32 - screen_w as f32 * 0.5) + camera_pos.0;
                let wave = (world_x * 0.02).sin() * 3.0;
                let surface_y = water_surface_screen_y + wave;
                
                // Bright surface line visible from both above and below
                rect!(
                    x = x as f32,
                    y = surface_y,
                    w = 1.0,
                    h = 3.0,
                    color = 0x66BBFFFF, // Bright blue surface
                    fixed = true
                );
            }
        }
    }
    
    /// Render underwater lighting effect
    fn render_underwater_lighting(&self, screen_w: u32, screen_h: u32) {
        // Create a subtle vignette effect for underwater ambiance
        for y in 0..screen_h {
            for x in 0..screen_w {
                let dx = (x as f32 - screen_w as f32 * 0.5) / screen_w as f32;
                let dy = (y as f32 - screen_h as f32 * 0.5) / screen_h as f32;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance > 0.6 {
                    let alpha = ((distance - 0.6) * 2.0 * 128.0) as u32;
                    let tint_color = 0x00112200 | (alpha.min(128) << 24);
                    rect!(x = x as f32, y = y as f32, w = 1.0, h = 1.0, color = tint_color, fixed = true);
                }
            }
        }
    }
    
    /// Render entities
    fn render_entities(&self, camera_pos: (f32, f32), screen_w: u32, screen_h: u32) {
        for command in &self.render_queue {
            if let RenderCommand::Entity { data, entity_type } = command {
                self.render_entity(data, entity_type, camera_pos, screen_w, screen_h);
            }
        }
    }
    
    /// Render a single entity
    fn render_entity(&self, data: &RenderData, entity_type: &EntityType, camera_pos: (f32, f32), screen_w: u32, screen_h: u32) {
        if let Some(screen_position) = data.screen_position {
            let screen_x = screen_position.0 - camera_pos.0 + screen_w as f32 * 0.5;
            let screen_y = screen_position.1 - camera_pos.1 + screen_h as f32 * 0.5;
        

            // Check if entity is on screen
            if screen_x > -data.size && screen_x < screen_w as f32 + data.size &&
            screen_y > -data.size && screen_y < screen_h as f32 + data.size {
                match entity_type {
                    EntityType::Player => {
                        self.render_player(screen_x, screen_y, data);
                    },
                    EntityType::Raft => {
                        self.render_raft(screen_x, screen_y, data);
                    },
                    EntityType::Fish => {
                        self.render_fish(screen_x, screen_y, data);
                    },
                    EntityType::Monster => {
                        self.render_monster(screen_x, screen_y, data);
                    },
                    EntityType::Shark => {
                        self.render_shark(screen_x, screen_y, data);
                    },
                    EntityType::FloatingItem => {
                        self.render_floating_item(screen_x, screen_y, data);
                    },
                    EntityType::Particle => {
                        self.render_particle(screen_x, screen_y, data);
                    },
                    _ => {
                        // Default rendering for other entity types
                        circ!(d = data.size, position = (screen_x, screen_y), color = data.color, fixed = true);
                    }
                }
            }
        }
    }
    
    /// Render player
    fn render_player(&self, x: f32, y: f32, data: &RenderData) {
        let player_color = data.color;
        
        // Main body (10x12 pixels)
        rect!(
            x = x - 5.0,
            y = y - 6.0,
            w = 10.0,
            h = 12.0,
            color = player_color,
            fixed = true
        );
        
        // Head (6x6 pixels)
        rect!(
            x = x - 3.0,
            y = y - 12.0,
            w = 6.0,
            h = 6.0,
            color = player_color,
            fixed = true
        );
        
        // Arms (2x6 pixels each)
        rect!(
            x = x - 7.0,
            y = y - 4.0,
            w = 2.0,
            h = 6.0,
            color = player_color,
            fixed = true
        );
        rect!(
            x = x + 5.0,
            y = y - 4.0,
            w = 2.0,
            h = 6.0,
            color = player_color,
            fixed = true
        );
        
        // Legs (3x6 pixels each)
        rect!(
            x = x - 3.0,
            y = y + 6.0,
            w = 3.0,
            h = 6.0,
            color = player_color,
            fixed = true
        );
        rect!(
            x = x + 0.0,
            y = y + 6.0,
            w = 3.0,
            h = 6.0,
            color = player_color,
            fixed = true
        );
    }
    
    /// Render fish
    fn render_fish(&self, x: f32, y: f32, data: &RenderData) {
        circ!(d = data.size, position = (x, y), color = data.color, fixed = true);
    }
    
    /// Render monster
    fn render_monster(&self, x: f32, y: f32, data: &RenderData) {
        rect!(x = x - data.size * 0.5, y = y - data.size * 0.5, w = data.size, h = data.size, color = data.color, fixed = true);
    }
    
    /// Render shark
    fn render_shark(&self, x: f32, y: f32, data: &RenderData) {
        // Shark body (elongated)
        rect!(x = x - data.size * 0.8, y = y - data.size * 0.3, w = data.size * 1.6, h = data.size * 0.6, color = data.color, fixed = true);
        
        // Shark fin
        rect!(x = x - data.size * 0.2, y = y - data.size * 0.8, w = data.size * 0.4, h = data.size * 0.4, color = data.color, fixed = true);
    }
    
    /// Render floating item
    fn render_floating_item(&self, x: f32, y: f32, data: &RenderData) {
        // Add bobbing animation
        let bobbing = (x * 0.05).sin() * 3.0;
        circ!(d = data.size, position = (x, y + bobbing), color = data.color, fixed = true);
    }
    
    /// Render particle
    fn render_particle(&self, x: f32, y: f32, data: &RenderData) {
        rect!(x = x - 1.0, y = y - 1.0, w = 2.0, h = 2.0, color = data.color, fixed = true);
    }
    
    /// Render raft
    fn render_raft(&self, x: f32, y: f32, data: &RenderData) {
        // Distance-based scaling in side-scrolling
        let raft_size = match self.view_mode {
            RenderViewMode::TopDown => data.size,
            RenderViewMode::SideScroll => {
                if let Some(player_pos) = &self.last_player_world_pos {
                    let dy_topdown = (player_pos.y - data.world_position.y).abs();
                    // Use live side-scroll horizontal separation for perspective
                    let dx_world = (data.world_position.x - player_pos.x).abs();
                    let max_v = 200.0; // vertical range before min scale
                    let max_h = 400.0; // horizontal range, weaker effect
                    let tv = (dy_topdown / max_v).clamp(0.0, 1.0);
                    let th = (dx_world / max_h).clamp(0.0, 1.0);
                    let min_scale = 0.5;
                    let base = 1.0 - tv * (1.0 - min_scale);
                    let scale = base * (1.0 - 0.15 * th);
                    data.size * scale
                } else {
                    data.size * 0.6
                }
            }
        };
        if let RenderViewMode::TopDown = self.view_mode {
            // Draw a square raft centered at (x, y)
            rect!(
                x = x - raft_size * 0.5,
                y = y - raft_size * 0.5,
                w = raft_size,
                h = raft_size,
                color = data.color,
                fixed = true
            );
            // Simple grid lines to imply planks
            for i in 1..4 {
                let t = i as f32 / 4.0;
                rect!(x = x - raft_size * 0.5, y = y - raft_size * 0.5 + raft_size * t, w = raft_size, h = 1.0, color = 0x8B4513FF, fixed = true);
                rect!(x = x - raft_size * 0.5 + raft_size * t, y = y - raft_size * 0.5, w = 1.0, h = raft_size, color = 0x8B4513FF, fixed = true);
            }
        } else {
            // Side/other modes: original elongated deck look
            rect!(
                x = x - raft_size * 0.5,
                y = y - raft_size * 0.3,
                w = raft_size,
                h = raft_size * 0.3,
                color = data.color,
                fixed = true
            );
            for i in 0..3 {
                rect!(
                    x = x - raft_size * 0.5 + i as f32 * (raft_size / 3.0),
                    y = y - raft_size * 0.3,
                    w = 2.0,
                    h = raft_size * 0.3,
                    color = 0x8B4513FF,
                    fixed = true
                );
            }
        }
    }

    fn render_ocean_fullscreen(&self, screen_w: u32, screen_h: u32) {
        // Simple vertical gradient fill for ocean in top-down view
        for y in 0..screen_h {
            let t = y as f32 / screen_h as f32;
            let r = (0x41 as f32 * (1.0 - 0.2 * t)) as u32;
            let g = (0x69 as f32 * (1.0 - 0.2 * t)) as u32;
            let b = (0xE1 as f32 * (1.0 - 0.4 * t)) as u32;
            let color = (r << 24) | (g << 16) | (b << 8) | 0xFF;
            rect!(x = 0.0, y = y as f32, w = screen_w as f32, h = 1.0, color = color, fixed = true);
        }
    }
}

/// Background layer types
#[turbo::serialize]
pub enum BackgroundLayer {
    SkyGradient,
    OceanGradient,
    WaterSurface,
    UnderwaterLighting,
}

/// Render commands for the render queue
#[turbo::serialize]
pub enum RenderCommand {
    Entity {
        data: RenderData,
        entity_type: EntityType,
    },
    Background {
        layer: RenderLayer,
        data: RenderData,
    },
    UI {
        layer: RenderLayer,
        data: RenderData,
    },
}

#[turbo::serialize]
pub enum RenderViewMode {
    TopDown,
    SideScroll,
}
