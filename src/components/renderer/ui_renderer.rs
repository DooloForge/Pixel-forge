use super::*;
use crate::math::Vec2 as V2;
use crate::constants::*;

/// Handles all UI rendering
#[turbo::serialize]
pub struct UIRenderer {
    ui_elements: Vec<UIElement>,
    current_ui_mode: UIMode,
    hud_state: Option<HudState>,
    minimap_points: Vec<MinimapPoint>,
}

impl UIRenderer {
    pub fn new() -> Self {
        Self {
            ui_elements: Vec::new(),
            current_ui_mode: UIMode::Playing,
            hud_state: None,
            minimap_points: Vec::new(),
        }
    }
    
    /// Set UI mode
    pub fn set_ui_mode(&mut self, mode: UIMode) {
        self.current_ui_mode = mode;
    }

    /// Set HUD dynamic values
    pub fn set_hud_state(&mut self, state: HudState) {
        self.hud_state = Some(state);
    }

    /// Set minimap points (world-space projected externally)
    pub fn set_minimap_points(&mut self, points: Vec<MinimapPoint>) {
        self.minimap_points = points;
    }
    
    /// Add UI element
    pub fn add_ui_element(&mut self, element: UIElement) {
        self.ui_elements.push(element);
    }
    
    /// Remove UI element
    pub fn remove_ui_element(&mut self, id: &str) {
        self.ui_elements.retain(|e| e.id != id);
    }
    
    /// Render all UI based on current mode
    pub fn render(&self) {
        match self.current_ui_mode {
            UIMode::Playing => self.render_hud(),
            UIMode::Inventory => self.render_inventory(),
            UIMode::Crafting => self.render_crafting(),
            UIMode::Paused => self.render_paused(),
        }
        
        // Render common UI elements
        self.render_common_ui();
    }
    
    /// Render HUD for playing mode
    fn render_hud(&self) {
        let (screen_w, _screen_h) = resolution();
        if let Some(hud) = &self.hud_state {
            // Tool info
            let t1 = format!("Tool: {}", hud.tool);
            text!(t1.as_str(), x = 10, y = 10, color = UI_TEXT_WHITE, fixed = true);
            // Survival stats
            let t2 = format!("Health: {}/100", hud.health as i32);
            let t3 = format!("Hunger: {}/100", hud.hunger as i32);
            let t4 = format!("Thirst: {}/100", hud.thirst as i32);
            text!(t2.as_str(), x = 10, y = 26, color = UI_TEXT_RED, fixed = true);
            text!(t3.as_str(), x = 10, y = 42, color = UI_TEXT_ORANGE, fixed = true);
            text!(t4.as_str(), x = 10, y = 58, color = UI_TEXT_BLUE, fixed = true);
            // Game status
            let t5 = format!("Status: {}", hud.status);
            text!(t5.as_str(), x = 10, y = 130, color = UI_TEXT_WHITE, fixed = true);
            // Positions (optional)
            if let Some(p) = &hud.player_pos {
                text!(p.as_str(), x = 10, y = 146, color = UI_TEXT_WHITE, fixed = true);
            }
            if let Some(r) = &hud.raft_pos {
                text!(r.as_str(), x = 10, y = 162, color = UI_TEXT_WHITE, fixed = true);
            }
        } else {
            // Fallback placeholders
            text!("Tool: Hook", x = 10, y = 10, color = UI_TEXT_WHITE, fixed = true);
            text!("Health: 100/100", x = 10, y = 26, color = UI_TEXT_RED, fixed = true);
            text!("Hunger: 100/100", x = 10, y = 42, color = UI_TEXT_ORANGE, fixed = true);
            text!("Thirst: 100/100", x = 10, y = 58, color = UI_TEXT_BLUE, fixed = true);
            text!("Status: --", x = 10, y = 130, color = UI_TEXT_WHITE, fixed = true);
        }
        
        // Controls
        text!("WASD: Move, E: Switch Tool, F: Eat", x = 10, y = 90, color = UI_TEXT_WHITE, fixed = true);
        text!("I: Inventory, C: Crafting", x = 10, y = 106, color = UI_TEXT_WHITE, fixed = true);
        
        // Minimap
        self.render_minimap(screen_w);
    }
    
    /// Render inventory UI
    fn render_inventory(&self) {
        let (w, h) = resolution();
        let panel_w = 300.0;
        let panel_h = 400.0;
        let panel_x = (w as f32 - panel_w) * 0.5;
        let panel_y = (h as f32 - panel_h) * 0.5;
        
        // Background
        rect!(x = panel_x, y = panel_y, w = panel_w, h = panel_h, color = UI_PANEL_BG, fixed = true);
        
        // Title
        text!("INVENTORY", x = panel_x + 10.0, y = panel_y + 10.0, color = UI_TEXT_WHITE, fixed = true);
        
        // Materials (placeholder)
        text!("Wood: 15", x = panel_x + 20.0, y = panel_y + 50.0, color = UI_TEXT_WHITE, fixed = true);
        text!("Plastic: 8", x = panel_x + 20.0, y = panel_y + 75.0, color = UI_TEXT_WHITE, fixed = true);
        text!("Rope: 3", x = panel_x + 20.0, y = panel_y + 100.0, color = UI_TEXT_WHITE, fixed = true);
        text!("Metal: 5", x = panel_x + 20.0, y = panel_y + 125.0, color = UI_TEXT_WHITE, fixed = true);
        
        text!("Press I to close", x = panel_x + 10.0, y = panel_y + panel_h - 30.0, color = UI_TEXT_GRAY, fixed = true);
    }
    
    /// Render crafting UI
    fn render_crafting(&self) {
        let (w, h) = resolution();
        let panel_w = 400.0;
        let panel_h = 500.0;
        let panel_x = (w as f32 - panel_w) * 0.5;
        let panel_y = (h as f32 - panel_h) * 0.5;
        
        // Background
        rect!(x = panel_x, y = panel_y, w = panel_w, h = panel_h, color = UI_PANEL_BG, fixed = true);
        
        // Title
        text!("CRAFTING", x = panel_x + 10.0, y = panel_y + 10.0, color = UI_TEXT_WHITE, fixed = true);
        
        text!("Crafting system coming soon...", x = panel_x + 20.0, y = panel_y + 50.0, color = UI_TEXT_GRAY, fixed = true);
        text!("Press C to close", x = panel_x + 10.0, y = panel_y + panel_h - 30.0, color = UI_TEXT_GRAY, fixed = true);
    }
    
    /// Render paused UI
    fn render_paused(&self) {
        let (w, h) = resolution();
        let panel_w = 300.0;
        let panel_h = 200.0;
        let panel_x = (w as f32 - panel_w) * 0.5;
        let panel_y = (h as f32 - panel_h) * 0.5;
        
        // Background
        rect!(x = panel_x, y = panel_y, w = panel_w, h = panel_h, color = UI_PANEL_BG, fixed = true);
        
        // Title
        text!("PAUSED", x = panel_x + 10.0, y = panel_y + 10.0, color = UI_TEXT_WHITE, fixed = true);
        
        text!("Game is paused", x = panel_x + 20.0, y = panel_y + 50.0, color = UI_TEXT_GRAY, fixed = true);
        text!("Press ESC to resume", x = panel_x + 10.0, y = panel_y + panel_h - 30.0, color = UI_TEXT_GRAY, fixed = true);
    }
    
    /// Render common UI elements
    fn render_common_ui(&self) {
        // Render any persistent UI elements here
        for element in &self.ui_elements {
            self.render_ui_element(element);
        }
    }
    
    /// Render a single UI element
    fn render_ui_element(&self, element: &UIElement) {
        match element.element_type {
            UIElementType::Text => {
                if let Some(text) = &element.text {
                    text!(
                        text.as_str(),
                        x = element.position.x,
                        y = element.position.y,
                        color = element.color,
                        fixed = true
                    );
                }
            },
            UIElementType::Button => {
                // Button background
                rect!(
                    x = element.position.x,
                    y = element.position.y,
                    w = element.size.x,
                    h = element.size.y,
                    color = element.color,
                    fixed = true
                );
                
                // Button text
                if let Some(text) = &element.text {
                    let text_x = element.position.x + (element.size.x - text.len() as f32 * 6.0) * 0.5;
                    let text_y = element.position.y + (element.size.y - 12.0) * 0.5;
                    text!(
                        text.as_str(),
                        x = text_x,
                        y = text_y,
                        color = UI_TEXT_WHITE,
                        fixed = true
                    );
                }
            },
            UIElementType::Panel => {
                // Panel background
                rect!(
                    x = element.position.x,
                    y = element.position.y,
                    w = element.size.x,
                    h = element.size.y,
                    color = element.color,
                    fixed = true
                );
                
                // Panel title
                if let Some(text) = &element.text {
                    text!(
                        text.as_str(),
                        x = element.position.x + 10.0,
                        y = element.position.y + 10.0,
                        color = UI_TEXT_WHITE,
                        fixed = true
                    );
                }
            },
        }
    }
    
    /// Render minimap
    fn render_minimap(&self, screen_w: u32) {
        let minimap_size = 80.0;
        let minimap_x = screen_w as f32 - minimap_size - 8.0;
        let minimap_y = 8.0;
        
        // Minimap background
        rect!(
            x = minimap_x,
            y = minimap_y,
            w = minimap_size,
            h = minimap_size,
            color = 0x88000000,
            fixed = true
        );
        
        // Minimap border
        rect!(
            x = minimap_x - 2.0,
            y = minimap_y - 2.0,
            w = minimap_size + 4.0,
            h = minimap_size + 4.0,
            color = UI_TEXT_WHITE,
            fixed = true
        );
        
        // Minimap content
        rect!(
            x = minimap_x,
            y = minimap_y,
            w = minimap_size,
            h = minimap_size,
            color = 0x223344FF,
            fixed = true
        );
        
        // Points (already projected to minimap space)
        for p in &self.minimap_points {
            circ!(d = p.size, position = (minimap_x + p.x, minimap_y + p.y), color = p.color, fixed = true);
        }
        
        // Minimap title
        text!("Map", x = minimap_x, y = minimap_y - 12.0, color = UI_TEXT_WHITE, fixed = true);
    }
    
    /// Check if a point is inside a UI element
    pub fn is_point_in_ui(&self, point: &V2) -> Option<&UIElement> {
        for element in &self.ui_elements {
            if point.x >= element.position.x && 
               point.x <= element.position.x + element.size.x &&
               point.y >= element.position.y && 
               point.y <= element.position.y + element.size.y {
                return Some(element);
            }
        }
        None
    }
    
    /// Handle UI click
    pub fn handle_click(&mut self, point: &V2) -> Option<UIClickEvent> {
        if let Some(element) = self.is_point_in_ui(point) {
            match element.element_type {
                UIElementType::Button => {
                    return Some(UIClickEvent::ButtonClicked {
                        element_id: element.id.clone(),
                        position: point.clone(),
                    });
                },
                _ => {}
            }
        }
        None
    }
}

#[turbo::serialize]
pub struct HudState {
    pub tool: String,
    pub health: f32,
    pub hunger: f32,
    pub thirst: f32,
    pub status: String,
    pub player_pos: Option<String>,
    pub raft_pos: Option<String>,
}

#[turbo::serialize]
pub struct MinimapPoint {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: u32,
}

/// UI modes
#[derive(Copy, PartialEq)]
#[turbo::serialize]
pub enum UIMode {
    Playing,
    Inventory,
    Crafting,
    Paused,
}

/// UI element types
#[derive(Copy, PartialEq)]
#[turbo::serialize]
pub enum UIElementType {
    Text,
    Button,
    Panel,
}

/// UI element
#[turbo::serialize]
pub struct UIElement {
    pub id: String,
    pub element_type: UIElementType,
    pub position: V2,
    pub size: V2,
    pub color: u32,
    pub text: Option<String>,
    pub visible: bool,
}

impl UIElement {
    pub fn new_text(id: &str, position: V2, text: &str, color: u32) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Text,
            position,
            size: V2::zero(),
            color,
            text: Some(text.to_string()),
            visible: true,
        }
    }
    
    pub fn new_button(id: &str, position: V2, size: V2, text: &str, color: u32) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Button,
            position,
            size,
            color,
            text: Some(text.to_string()),
            visible: true,
        }
    }
    
    pub fn new_panel(id: &str, position: V2, size: V2, title: &str, color: u32) -> Self {
        Self {
            id: id.to_string(),
            element_type: UIElementType::Panel,
            position,
            size,
            color,
            text: Some(title.to_string()),
            visible: true,
        }
    }
}

/// UI click events
#[derive(Clone)]
pub enum UIClickEvent {
    ButtonClicked {
        element_id: String,
        position: V2,
    },
}
