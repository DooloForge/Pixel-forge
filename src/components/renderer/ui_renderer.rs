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

        // Hotbar (10 fixed slots like Minecraft)
        self.render_hotbar();
    }
    
    /// Render inventory UI
    fn render_inventory(&self) {
        self.render_inventory_with_data(None);
        // Render context menu if present (simple two-option menu)
        if let Some(hud) = &self.hud_state {
            // GameState is not accessible here; context menu rendering should be fed via HUD or a UI element list in future.
            // Placeholder: skipped drawing here. Menu handling will be in scene logic.
        }
    }
    
    /// Render inventory UI with actual player data
    pub fn render_inventory_with_data(&self, inventory_data: Option<&crate::models::player::Inventory>) {
        self.render_inventory_with_data_and_drag(inventory_data, None);
    }

    /// Render inventory UI with drag preview
    pub fn render_inventory_with_data_and_drag(&self, inventory_data: Option<&crate::models::player::Inventory>, dragging: Option<(u32, u32, f32, f32)>) {
        let (w, h) = resolution();
        // Full-screen panel with small margins
        let panel_margin = 8.0_f32;
        let panel_x = panel_margin;
        let panel_y = panel_margin;
        let panel_w = w as f32 - panel_margin * 2.0;
        let panel_h = h as f32 - panel_margin * 2.0;
        
        // Background
        rect!(x = panel_x, y = panel_y, w = panel_w, h = panel_h, color = UI_PANEL_BG, fixed = true);
        
        // Title
        text!("INVENTORY", x = panel_x + 10.0, y = panel_y + 10.0, color = UI_TEXT_WHITE, fixed = true);
        
        if let Some(inventory) = inventory_data {
            // Layout: 10-wide full-screen grid
            let hotbar_cols = 10usize; // 0..9
            let cols = 10usize; // bag grid columns
            let bag_count = inventory.max_slots.saturating_sub(hotbar_cols); // expected 30
            let rows = (bag_count + cols - 1) / cols; // ceil division (should be 3)
            let desired_slot = 32.0_f32;
            let slot_margin = 4.0;
            // Compute max slot size that fits the panel width with margins
            let available_w = panel_w - 40.0 - (cols as f32 - 1.0) * slot_margin;
            let slot_size_w = (available_w / cols as f32).floor();
            let mut slot_size = desired_slot.min(slot_size_w).max(22.0_f32);
            // Ensure hotbar + grid fits vertically
            let total_h = (hotbar_cols > 0) as i32 as f32 * (slot_size + 16.0) + rows as f32 * (slot_size + slot_margin) - slot_margin + 120.0;
            if total_h > panel_h {
                let available_h = (panel_h - 120.0).max(100.0);
                let per_row = (available_h / (rows as f32 + 1.0 + (16.0 / (slot_size + slot_margin)))).floor();
                // fallback: recompute slot size from width only (already bounded)
                let _ = per_row; // keep simple; width-bound dominates
            }
            // Hotbar section
            let hotbar_slot_size = slot_size.min(32.0);
            let hotbar_total_w = hotbar_cols as f32 * (hotbar_slot_size + slot_margin) - slot_margin;
            let hotbar_start_x = panel_x + (panel_w - hotbar_total_w) * 0.5;
            let hotbar_start_y = panel_y + 40.0;

            // Draw hotbar slots from inventory slots 0..9
            for i in 0..hotbar_cols {
                let slot_x = hotbar_start_x + i as f32 * (hotbar_slot_size + slot_margin);
                let slot_y = hotbar_start_y;
                // Background and border
                rect!(x = slot_x, y = slot_y, w = hotbar_slot_size, h = hotbar_slot_size, color = 0x333333CC, fixed = true);
                rect!(x = slot_x - 1.0, y = slot_y - 1.0, w = hotbar_slot_size + 2.0, h = hotbar_slot_size + 2.0, color = UI_TEXT_GRAY, fixed = true);
                // Item preview if linked
                if let Some(slot) = inventory.get_slot(i) {
                    if let Some(item_type) = slot.item_type {
                        let s = hotbar_slot_size * 0.7;
                        rect!(x = slot_x + (hotbar_slot_size - s) * 0.5, y = slot_y + (hotbar_slot_size - s) * 0.5, w = s, h = s, color = item_type.color(), fixed = true);
                        if slot.quantity > 1 {
                            let qty_text = format!("{}", slot.quantity);
                            text!(qty_text.as_str(), x = slot_x + hotbar_slot_size - 12.0, y = slot_y + hotbar_slot_size - 12.0, color = UI_TEXT_WHITE, fixed = true);
                        }
                    }
                }
                // Index label (1-9,0)
                let label = if i < 9 { (i + 1).to_string() } else { "0".to_string() };
                text!(label.as_str(), x = slot_x + 2.0, y = slot_y + 2.0, color = UI_TEXT_WHITE, fixed = true);
            }

            // Inventory grid below hotbar
            let grid_start_x = panel_x + 20.0;
            let grid_start_y = hotbar_start_y + hotbar_slot_size + 16.0;
            
            // Draw bag slots 10..(max_slots-1) in 10 columns
            for i in 10..inventory.max_slots {
                let grid_i = i - 10;
                let col = grid_i % cols;
                let row = grid_i / cols;
                let slot_x = grid_start_x + col as f32 * (slot_size + slot_margin);
                let slot_y = grid_start_y + row as f32 * (slot_size + slot_margin);
                
                // Slot background
                let slot_color = if Some(i) == inventory.selected_slot {
                    0xFFFFFF44 // Highlighted slot
                } else {
                    0x444444FF // Normal slot
                };
                rect!(x = slot_x, y = slot_y, w = slot_size, h = slot_size, color = slot_color, fixed = true);
                
                // Slot border
                rect!(x = slot_x - 1.0, y = slot_y - 1.0, w = slot_size + 2.0, h = slot_size + 2.0, color = UI_TEXT_GRAY, fixed = true);
                rect!(x = slot_x, y = slot_y, w = slot_size, h = slot_size, color = slot_color, fixed = true);
                
                // Item in slot
                if let Some(slot) = inventory.get_slot(i) {
                    if let Some(item_type) = slot.item_type {
                        // Item color/icon (simplified as colored square)
                        let item_color = item_type.color();
                        let item_size = slot_size * 0.7;
                        let item_x = slot_x + (slot_size - item_size) * 0.5;
                        let item_y = slot_y + (slot_size - item_size) * 0.5;
                        rect!(x = item_x, y = item_y, w = item_size, h = item_size, color = item_color, fixed = true);
                        
                        // Quantity text
                        if slot.quantity > 1 {
                            let qty_text = format!("{}", slot.quantity);
                            text!(qty_text.as_str(), x = slot_x + slot_size - 16.0, y = slot_y + slot_size - 12.0, color = UI_TEXT_WHITE, fixed = true);
                        }
                    }
                }
            }
            
            // Inventory stats
            let stats_y = (grid_start_y + rows as f32 * (slot_size + slot_margin) + 12.0).min(panel_y + panel_h - 70.0);
            let total_items = inventory.get_total_items();
            let capacity_text = format!("Items: {}/{}", total_items, inventory.max_slots * 64); // Rough capacity estimate
            text!(capacity_text.as_str(), x = grid_start_x, y = stats_y, color = UI_TEXT_WHITE, fixed = true);

            // Drag preview on top if requested (color, qty, mouse x, mouse y)
            if let Some((color, qty, mx, my)) = dragging {
                let s = 22.0_f32;
                rect!(x = mx - s * 0.5, y = my - s * 0.5, w = s, h = s, color = color, fixed = true);
                if qty > 1 { let qty_text = format!("{}", qty); text!(qty_text.as_str(), x = mx + 6.0, y = my + 6.0, color = UI_TEXT_WHITE, fixed = true); }
            }
            
        } else {
            // Fallback when no inventory data available
            text!("Loading inventory...", x = panel_x + 20.0, y = panel_y + 50.0, color = UI_TEXT_GRAY, fixed = true);
        }
        
        // Instructions
        let instr_y1 = panel_y + panel_h - 52.0;
        let instr_y2 = panel_y + panel_h - 32.0;
        text!("Click to select, Drag to move, Right-click for quick slot", x = panel_x + 10.0, y = instr_y1, color = UI_TEXT_GRAY, fixed = true);
        text!("Press I to close", x = panel_x + 10.0, y = instr_y2, color = UI_TEXT_GRAY, fixed = true);
    }
    
    /// Render crafting UI
    fn render_crafting(&self) {
        self.render_crafting_with_data(None, None);
    }
    
    /// Render crafting UI with actual game data
    pub fn render_crafting_with_data(&self, crafting_system: Option<&crate::models::crafting::CraftingSystem>, inventory: Option<&crate::models::player::Inventory>) {
        let (w, h) = resolution();
        let panel_w = 600.0;
        let panel_h = 500.0;
        let panel_x = (w as f32 - panel_w) * 0.5;
        let panel_y = (h as f32 - panel_h) * 0.5;
        
        // Background
        rect!(x = panel_x, y = panel_y, w = panel_w, h = panel_h, color = UI_PANEL_BG, fixed = true);
        
        // Title
        text!("CRAFTING", x = panel_x + 10.0, y = panel_y + 10.0, color = UI_TEXT_WHITE, fixed = true);
        
        if let (Some(crafting), Some(inventory)) = (crafting_system, inventory) {
            let categories = vec![
                crate::models::crafting::CraftingCategory::Tools,
                crate::models::crafting::CraftingCategory::Building,
                crate::models::crafting::CraftingCategory::Food,
                crate::models::crafting::CraftingCategory::Storage,
                crate::models::crafting::CraftingCategory::Survival,
            ];
            
            // Category tabs
            let tab_width = (panel_w - 40.0) / categories.len() as f32;
            let tab_height = 30.0;
            let tab_y = panel_y + 35.0;
            
            for (i, category) in categories.iter().enumerate() {
                let tab_x = panel_x + 20.0 + i as f32 * tab_width;
                let recipes = crafting.get_recipes_by_category(category.clone());
                let tab_color = if recipes.is_empty() { 0x666666FF } else { 0x888888FF };
                
                rect!(x = tab_x, y = tab_y, w = tab_width - 2.0, h = tab_height, color = tab_color, fixed = true);
                text!(category.name(), x = tab_x + 5.0, y = tab_y + 8.0, color = UI_TEXT_WHITE, fixed = true);
                
                let count_text = format!("({})", recipes.len());
                text!(count_text.as_str(), x = tab_x + 5.0, y = tab_y + 18.0, color = UI_TEXT_GRAY, fixed = true);
            }
            
            // Recipe list area
            let list_start_y = tab_y + tab_height + 10.0;
            
            // Show all available recipes (simplified for now)
            let available_recipes = crafting.get_available_recipes();
            let mut y_offset = 0.0;
            
            for recipe in available_recipes.iter().take(8) { // Limit to 8 visible recipes
                let recipe_y = list_start_y + y_offset;
                let recipe_height = 45.0;
                
                // Recipe background
                let can_craft = crafting.can_craft(&recipe.id, inventory);
                let recipe_color = if can_craft { 0x444444FF } else { 0x222222FF };
                rect!(x = panel_x + 20.0, y = recipe_y, w = panel_w - 40.0, h = recipe_height, color = recipe_color, fixed = true);
                
                // Recipe name and description
                let name_color = if can_craft { UI_TEXT_WHITE } else { UI_TEXT_GRAY };
                text!(recipe.name.as_str(), x = panel_x + 30.0, y = recipe_y + 5.0, color = name_color, fixed = true);
                text!(recipe.description.as_str(), x = panel_x + 30.0, y = recipe_y + 18.0, color = UI_TEXT_GRAY, fixed = true);
                
                // Ingredients
                let mut ingredient_x = panel_x + 30.0;
                text!("Needs:", x = ingredient_x, y = recipe_y + 30.0, color = UI_TEXT_GRAY, fixed = true);
                ingredient_x += 45.0;
                
                for (item_type, amount) in &recipe.ingredients {
                    let has_amount = inventory.get_count(*item_type);
                    let ingredient_color = if has_amount >= *amount { 0x00FF00FF } else { 0xFF0000FF };
                    let ingredient_text = format!("{}x{}", amount, format!("{:?}", item_type));
                    text!(ingredient_text.as_str(), x = ingredient_x, y = recipe_y + 30.0, color = ingredient_color, fixed = true);
                    ingredient_x += 80.0;
                }
                
                // Result
                let (result_type, result_amount) = recipe.result;
                let result_text = format!("-> {}x{:?}", result_amount, result_type);
                text!(result_text.as_str(), x = panel_x + panel_w - 150.0, y = recipe_y + 18.0, color = UI_TEXT_WHITE, fixed = true);
                
                // Craft button area (visual indication only for now)
                if can_craft {
                    rect!(x = panel_x + panel_w - 80.0, y = recipe_y + 5.0, w = 60.0, h = 20.0, color = 0x00AA00FF, fixed = true);
                    text!("CRAFT", x = panel_x + panel_w - 75.0, y = recipe_y + 8.0, color = UI_TEXT_WHITE, fixed = true);
                }
                
                y_offset += recipe_height + 5.0;
            }
            
            if available_recipes.is_empty() {
                text!("No recipes discovered yet.", x = panel_x + 30.0, y = list_start_y + 20.0, color = UI_TEXT_GRAY, fixed = true);
                text!("Collect materials to discover new recipes!", x = panel_x + 30.0, y = list_start_y + 35.0, color = UI_TEXT_GRAY, fixed = true);
            }
            
        } else {
            text!("Loading crafting system...", x = panel_x + 20.0, y = panel_y + 50.0, color = UI_TEXT_GRAY, fixed = true);
        }
        
        text!("Click recipe to craft (when available)", x = panel_x + 10.0, y = panel_y + panel_h - 50.0, color = UI_TEXT_GRAY, fixed = true);
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

    /// Render 10-slot hotbar anchored at bottom center
    fn render_hotbar(&self) {
        let (w, h) = resolution();
        let slot_size = 24.0_f32;
        let margin = 4.0_f32;
        let count = 10usize;
        let total_w = count as f32 * slot_size + (count as f32 - 1.0) * margin;
        let start_x = (w as f32 - total_w) * 0.5;
        let y = h as f32 - slot_size - 8.0;
        let active_index: Option<usize> = if let Some(h) = &self.hud_state { h.hotbar_active } else { None };
        let items: Option<Vec<Option<(u32, u32)>>> = if let Some(h) = &self.hud_state { h.hotbar_items.clone() } else { None };

        for i in 0..count {
            let x = start_x + i as f32 * (slot_size + margin);
            // Background
            rect!(x = x, y = y, w = slot_size, h = slot_size, color = 0x333333CC, fixed = true);
            // Border
            let border_color = if Some(i) == active_index { UI_TEXT_WHITE } else { UI_TEXT_GRAY };
            rect!(x = x - 1.0, y = y - 1.0, w = slot_size + 2.0, h = slot_size + 2.0, color = border_color, fixed = true);

            // Item preview (simple square) and count if provided
            if let Some(Some((color, qty))) = items.as_ref().and_then(|v| v.get(i)).cloned() {
                let s = slot_size * 0.7;
                rect!(x = x + (slot_size - s) * 0.5, y = y + (slot_size - s) * 0.5, w = s, h = s, color = color, fixed = true);
                if qty > 1 { let txt = format!("{}", qty); text!(txt.as_str(), x = x + slot_size - 12.0, y = y + slot_size - 12.0, color = UI_TEXT_WHITE, fixed = true); }
            }

            // Slot index label (1-9,0) drawn LAST so it is not occluded by item preview
            let label = if i < 9 { (i + 1).to_string() } else { "0".to_string() };
            text!(label.as_str(), x = x + 2.0, y = y + 2.0, color = UI_TEXT_WHITE, fixed = true);
        }
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
    pub hotbar_items: Option<Vec<Option<(u32, u32)>>>,
    pub hotbar_active: Option<usize>,
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
