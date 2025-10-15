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
    
    /// Convert world position to screen position using current camera (centered)
    fn world_to_screen(&self, world_pos: &Vec3) -> (f32, f32) {
        let (screen_w, screen_h) = resolution();
        (
            world_pos.x - self.camera_pos.0 + screen_w as f32 * 0.5,
            world_pos.y - self.camera_pos.1 + screen_h as f32 * 0.5,
        )
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
        let entity_type = entity.get_entity_type();
        
        // Hide entities based on view mode
        match entity_type {
            EntityType::Fish => {
                // Hide fish in top-down mode
                if self.view_mode == RenderViewMode::TopDown {
                    return;
                }
            },
            EntityType::FloatingItem => {
                // Hide floating items in side-scroll mode
                if self.view_mode == RenderViewMode::SideScroll {
                    return;
                }
            },
            _ => {} // Other entities visible in both modes
        }
        
        // Project world position into current view
        let world_pos = entity.get_world_position();
        render_data.screen_position = match self.view_mode {
            RenderViewMode::TopDown => Some((world_pos.x, world_pos.y)),
            RenderViewMode::SideScroll => Some((world_pos.x, -world_pos.z)),
        };
        if render_data.visible {
            let command = RenderCommand::Entity {
                data: render_data.clone(),
                entity_type,
            };
            self.render_queue.push(command);
        }
    }
    
    /// Add player entity with movement data
    pub fn add_player_entity(&mut self, entity: &Entity, is_moving: bool, last_movement: &crate::math::Vec3) {
        let mut render_data = entity.get_render_data();
        let entity_type = entity.get_entity_type();
        
        // Store player movement data for rendering
        if let EntityType::Player = entity_type {
            // Store movement data in render data for player sprite selection
            render_data.player_is_moving = is_moving;
            render_data.player_last_movement = *last_movement;
        }
        
        // Project world position into current view
        let world_pos = entity.get_world_position();
        render_data.screen_position = match self.view_mode {
            RenderViewMode::TopDown => Some((world_pos.x, world_pos.y)),
            RenderViewMode::SideScroll => Some((world_pos.x, -world_pos.z)),
        };
        if render_data.visible {
            let command = RenderCommand::Entity {
                data: render_data.clone(),
                entity_type,
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
        
        // Clear screen
        self.clear_screen();
        
        // Sort render queue by layer, ensuring player renders on top
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
            
            // Get entity type for priority calculation
            let entity_priority_a = match a {
                RenderCommand::Entity { entity_type, .. } => match entity_type {
                    EntityType::Player => 100, // Highest priority
                    EntityType::Raft => 50,
                    _ => 0,
                },
                _ => 0,
            };
            let entity_priority_b = match b {
                RenderCommand::Entity { entity_type, .. } => match entity_type {
                    EntityType::Player => 100, // Highest priority
                    EntityType::Raft => 50,
                    _ => 0,
                },
                _ => 0,
            };
            
            // First sort by entity priority (player on top)
            if entity_priority_a != entity_priority_b {
                return entity_priority_a.cmp(&entity_priority_b);
            }
            
            // Then sort by layer
            layer_a.cmp(&layer_b)
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
            self.render_ocean_fullscreen(camera_pos, screen_w, screen_h);
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
                        self.render_player(data);
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
                    EntityType::Hook => {
                        self.render_hook(screen_x, screen_y, data);
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
    fn render_player(&self, data: &RenderData) {
        // Determine sprite based on movement, direction, and whether on raft
        let sprite_name = if data.player_is_moving {
            // Player is moving, determine direction and raft state
            let movement = &data.player_last_movement;
            if movement.y < -0.1 {
                if data.player_on_raft {
                    "run_up"
                } else {
                    "swim_move_up"
                }
            } else if movement.y > 0.1 {
                if data.player_on_raft {
                    "run_down"
                } else {
                    "swim_move_down"
                }
            } else if movement.x < -0.1 {
                if data.player_on_raft {
                    "run_left"
                } else {
                    "swim_move_left"
                }
            } else if movement.x > 0.1 {
                if data.player_on_raft {
                    "run_right"
                } else {
                    "swim_move_right"
                }
            } else {
                if data.player_on_raft {
                    "idle_down"
                } else {
                    "swim_idle_down"
                }
            }
        } else {
            // Player is idle, use last movement direction for idle sprite
            let movement = &data.player_last_movement;
            if movement.y < -0.1 {
                if data.player_on_raft {
                    "idle_up"
                } else {
                    "swim_idle_up"
                }
            } else if movement.y > 0.1 {
                if data.player_on_raft {
                    "idle_down"
                } else {
                    "swim_idle_down"
                }
            } else if movement.x < -0.1 {
                if data.player_on_raft {
                    "idle_left"
                } else {
                    "swim_idle_left"
                }
            } else if movement.x > 0.1 {
                if data.player_on_raft {
                    "idle_right"
                } else {
                    "swim_idle_right"
                }
            } else {
                if data.player_on_raft {
                    "idle_down"
                } else {
                    "swim_idle_down"
                }
            }
        };
        // Try to render player sprite using world coordinates
        sprite!(sprite_name, position = (data.world_position.x - 40.0, data.world_position.y - 40.0), size = (80.0, 80.0), origin = (40.0, 40.0));
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
        let final_y = y + bobbing;
        
        // Render different shapes based on size (which indicates item type)
        if data.size >= 12.0 {
            // Large items (Wood, Barrel) - render as rectangles
            rect!(
                x = x - data.size * 0.5,
                y = final_y - data.size * 0.3,
                w = data.size,
                h = data.size * 0.6,
                color = data.color,
                fixed = true
            );
        } else if data.size >= 8.0 {
            // Medium items (Plastic, Rope, Metal, Cloth) - render as squares
            rect!(
                x = x - data.size * 0.5,
                y = final_y - data.size * 0.5,
                w = data.size,
                h = data.size,
                color = data.color,
                fixed = true
            );
        } else {
            // Small items (Nail, Coconut, Fish, etc.) - render as circles
            circ!(d = data.size, position = (x, final_y), color = data.color, fixed = true);
        }
        
        // Add a subtle outline for better visibility
        let outline_color = (data.color & 0xFFFFFF00) | 0x80; // Same color with 50% alpha
        if data.size >= 8.0 {
            rect!(
                x = x - data.size * 0.5 - 1.0,
                y = final_y - data.size * 0.5 - 1.0,
                w = data.size + 2.0,
                h = data.size + 2.0,
                color = outline_color,
                fixed = true
            );
        } else {
            circ!(d = data.size + 2.0, position = (x, final_y), color = outline_color, fixed = true);
        }
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

    fn render_ocean_fullscreen(&self, camera_pos: (f32, f32), screen_w: u32, screen_h: u32) {
        // Top-down ocean using a repeating, tile-aligned depth pattern (structured, non-random)
        // Draw per world tile to minimize draw calls and avoid stutter
        let tile: f32 = 32.0;
        let pattern_size: i32 = 8; // 8x8 cells repeat
        let screen_w_f = screen_w as f32;
        let screen_h_f = screen_h as f32;

        // Base ocean color (steel blue-ish)
        let base_r = 0x41 as f32;
        let base_g = 0x69 as f32;
        let base_b = 0xE1 as f32;

        // Discrete shade multipliers (dark -> light)
        let shades: [f32; 3] = [0.72, 0.82, 0.92];

        // Hand-crafted 8x8 pattern of indices into shades[]
        let pattern: [[u8; 8]; 8] = [
            [1,1,1,1,2,2,2,1],
            [1,0,0,1,2,2,1,1],
            [1,0,0,1,1,1,1,1],
            [1,1,1,1,1,1,0,0],
            [2,2,1,1,1,1,0,0],
            [2,2,1,1,1,1,1,1],
            [2,1,1,1,1,1,1,2],
            [1,1,1,2,2,2,1,1],
        ];

        // Compute visible world tile range
        let world_left = camera_pos.0 - screen_w_f * 0.5;
        let world_top  = camera_pos.1 - screen_h_f * 0.5;
        let min_gx = (world_left / tile).floor() as i32 - 1;
        let min_gy = (world_top  / tile).floor() as i32 - 1;
        let max_gx = ((world_left + screen_w_f) / tile).ceil() as i32 + 1;
        let max_gy = ((world_top  + screen_h_f) / tile).ceil() as i32 + 1;

        // Collect wave positions to draw after filling tiles, so they are not overdrawn
        let mut wave_positions: Vec<(f32, f32)> = Vec::new();

        for gy in min_gy..=max_gy {
            for gx in min_gx..=max_gx {
                // Pattern index
                let mx = ((gx % pattern_size) + pattern_size) % pattern_size;
                let my = ((gy % pattern_size) + pattern_size) % pattern_size;
                let idx = pattern[my as usize][mx as usize] as usize;
                let mut shade = shades[idx];

                // World tile center for a tiny ripple once per tile
                let cx = gx as f32 * tile + tile * 0.5;
                let cy = gy as f32 * tile + tile * 0.5;
                let ripple = ((cx * 0.02).sin() * (cy * 0.017).cos()) * 0.012;
                shade = (shade + ripple).clamp(0.6, 1.0);

                // Convert world tile to screen rect
                let screen_x = (gx as f32 * tile - camera_pos.0) + screen_w_f * 0.5;
                let screen_y = (gy as f32 * tile - camera_pos.1) + screen_h_f * 0.5;

                let r = (base_r * shade) as u32;
                let g = (base_g * shade) as u32;
                let b = (base_b * shade) as u32;
            let color = (r << 24) | (g << 16) | (b << 8) | 0xFF;

                rect!(x = screen_x, y = screen_y, w = tile, h = tile, color = color, fixed = true);

                // Queue wave sprite world positions for a second pass
                if idx == 2 && ((gx + gy) & 1) == 0 {
                    let world_cx = gx as f32 * tile + tile * 0.5;
                    let world_cy = gy as f32 * tile + tile * 0.5;
                    wave_positions.push((world_cx, world_cy));
                }
            }
        }

        // Second pass: draw waves on top so they are not truncated by later tile fills
        for (wx, wy) in wave_positions.into_iter() {
            sprite!("waves", position = (wx, wy), size = (20.0, 20.0), origin = (10.0, 10.0));
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

#[derive(PartialEq)]
#[turbo::serialize]
pub enum RenderViewMode {
    TopDown,
    SideScroll,
}

impl RenderSystem {
    /// Render hook with rectangular body, hook tip, and line to player
    fn render_hook(&self, x: f32, y: f32, _data: &RenderData) {
        // Compute player's screen position from cached world position and camera
        let (screen_w, screen_h) = resolution();
        let (cam_x, cam_y) = self.camera_pos;

        if let Some(player_world) = &self.last_player_world_pos {
            let player_screen_x = (player_world.x - cam_x) + screen_w as f32 * 0.5;
            let player_screen_y = match self.view_mode {
                RenderViewMode::TopDown => (player_world.y - cam_y) + screen_h as f32 * 0.5,
                RenderViewMode::SideScroll => (-player_world.z - cam_y) + screen_h as f32 * 0.5,
            };

            // Draw thin line from hook to player using small rect segments
            let dx = player_screen_x - x;
            let dy = player_screen_y - y;
            let distance = (dx * dx + dy * dy).sqrt();
            let steps = (distance / 2.0) as i32; // segment every 2 pixels

            if steps > 0 {
                let step_x = dx / steps as f32;
                let step_y = dy / steps as f32;

                for i in 0..steps {
                    let line_x = x + step_x * i as f32;
                    let line_y = y + step_y * i as f32;

                    rect!(
                        x = line_x - 0.5,
                        y = line_y - 0.5,
                        w = 1.0,
                        h = 1.0,
                        color = 0x8B4513FF,
                        fixed = true
                    );
                }
            }
        }
        
        // Render hook body as a rectangle - make it very visible
        rect!(
            x = x - 6.0,
            y = y - 3.0,
            w = 12.0,
            h = 6.0,
            color = 0x8B4513FF, // Brown hook body
            fixed = true
        );
        
        // Add a bright white center to make it stand out
        rect!(
            x = x - 4.0,
            y = y - 2.0,
            w = 8.0,
            h = 4.0,
            color = 0xFFFFFFFF, // White center
            fixed = true
        );
        
        // Add a dark brown outline
        rect!(
            x = x - 7.0,
            y = y - 4.0,
            w = 14.0,
            h = 8.0,
            color = 0x654321FF, // Dark brown outline
            fixed = true
        );
        
        // Render the hook point as a small rectangle extending from the body
        rect!(
            x = x + 4.0,
            y = y - 1.0,
            w = 4.0,
            h = 2.0,
            color = 0x8B4513FF, // Brown hook point
            fixed = true
        );
        
        // Add a small circle at the very end to represent the hook tip
        circ!(d = 3.0, position = (x + 7.0, y), color = 0x8B4513FF, fixed = true);
    }
}
