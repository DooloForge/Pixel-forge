use crate::math::Vec2 as V2;
use crate::math::Vec3 as V3;
use crate::components::input::input_mapping::InputMapping;
use turbo::{keyboard, mouse};

/// Handles all input processing
#[turbo::serialize]
pub struct InputSystem {
    input_mapping: InputMapping,
    current_input_state: InputState,
    previous_input_state: InputState,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            input_mapping: InputMapping::new(),
            current_input_state: InputState::default(),
            previous_input_state: InputState::default(),
        }
    }
    
    /// Update input state
    pub fn update(&mut self) {
        self.previous_input_state = self.current_input_state.clone();
        self.current_input_state = self.poll_input();
    }
    
    /// Poll current input state
    fn poll_input(&self) -> InputState {
        let keyboard = keyboard::get();
        let mouse = mouse::screen();
        let (mx, my) = mouse.xy();
        
        InputState {
            // Movement
            move_left: keyboard.key_a().pressed(),
            move_right: keyboard.key_d().pressed(),
            move_up: keyboard.key_w().pressed(),
            move_down: keyboard.key_s().pressed(),
            
            // Raft sailing
            sail_left: keyboard.key_j().pressed(),
            sail_right: keyboard.key_l().pressed(),
            sail_forward: keyboard.key_i().pressed(),
            sail_backward: keyboard.key_k().pressed(),
            sail_north: keyboard.key_q().pressed(),
            sail_south: keyboard.key_e().pressed(),
            
            // Actions
            use_tool: mouse.left.just_pressed(),
            switch_tool: keyboard.key_e().just_pressed(),
            eat_food: keyboard.key_f().just_pressed(),
            collect_item: keyboard.key_g().just_pressed(),
            dive: keyboard.space().just_pressed(),
            
            // UI
            open_inventory: keyboard.key_i().just_pressed(),
            open_crafting: keyboard.key_c().just_pressed(),
            
            // Mouse
            mouse_pos: V2::new(mx as f32, my as f32),
            mouse_left_pressed: mouse.left.just_pressed(),
            mouse_left_held: mouse.left.pressed(),
            mouse_right_pressed: mouse.right.just_pressed(),
            
            // Camera
            camera_zoom_in: keyboard.key_e().just_pressed(),
            camera_zoom_out: keyboard.key_q().just_pressed(),

            // Crafting
            craft_item: keyboard.space().just_pressed(),
            quick_item_1: keyboard.digit_1().just_pressed(),
            quick_item_2: keyboard.digit_2().just_pressed(),
            quick_item_3: keyboard.digit_3().just_pressed(),
            quick_item_4: keyboard.digit_4().just_pressed(),
            quick_item_5: keyboard.digit_5().just_pressed(),
            quick_item_6: keyboard.digit_6().just_pressed(),
            quick_item_7: keyboard.digit_7().just_pressed(),
            quick_item_8: keyboard.digit_8().just_pressed(),
            quick_item_9: keyboard.digit_9().just_pressed(),
            quick_item_0: keyboard.digit_0().just_pressed(),
        }
    }
    
    /// Get current input state
    pub fn get_input_state(&self) -> &InputState {
        &self.current_input_state
    }
    
    /// Check if a key was just pressed
    pub fn is_key_just_pressed(&self, key: InputKey) -> bool {
        match key {
            InputKey::MoveLeft => !self.previous_input_state.move_left && self.current_input_state.move_left,
            InputKey::MoveRight => !self.previous_input_state.move_right && self.current_input_state.move_right,
            InputKey::MoveUp => !self.previous_input_state.move_up && self.current_input_state.move_up,
            InputKey::MoveDown => !self.previous_input_state.move_down && self.current_input_state.move_down,
            InputKey::SailLeft => !self.previous_input_state.sail_left && self.current_input_state.sail_left,
            InputKey::SailRight => !self.previous_input_state.sail_right && self.current_input_state.sail_right,
            InputKey::SailForward => !self.previous_input_state.sail_forward && self.current_input_state.sail_forward,
            InputKey::SailBackward => !self.previous_input_state.sail_backward && self.current_input_state.sail_backward,
            InputKey::SailNorth => !self.previous_input_state.sail_north && self.current_input_state.sail_north,
            InputKey::SailSouth => !self.previous_input_state.sail_south && self.current_input_state.sail_south,
            InputKey::UseTool => self.current_input_state.use_tool,
            InputKey::SwitchTool => self.current_input_state.switch_tool,
            InputKey::EatFood => self.current_input_state.eat_food,
            InputKey::CollectItem => self.current_input_state.collect_item,
            InputKey::OpenInventory => self.current_input_state.open_inventory,
            InputKey::OpenCrafting => self.current_input_state.open_crafting,
            InputKey::CraftItem => self.current_input_state.craft_item,
            InputKey::QuickItem1 => self.current_input_state.quick_item_1,
            InputKey::QuickItem2 => self.current_input_state.quick_item_2,
            InputKey::QuickItem3 => self.current_input_state.quick_item_3,
            InputKey::QuickItem4 => self.current_input_state.quick_item_4,
            InputKey::QuickItem5 => self.current_input_state.quick_item_5,
            InputKey::QuickItem6 => self.current_input_state.quick_item_6,
            InputKey::QuickItem7 => self.current_input_state.quick_item_7,
            InputKey::QuickItem8 => self.current_input_state.quick_item_8,
            InputKey::QuickItem9 => self.current_input_state.quick_item_9,
            InputKey::QuickItem0 => self.current_input_state.quick_item_0,
            InputKey::CameraZoomIn => self.current_input_state.camera_zoom_in,
            InputKey::CameraZoomOut => self.current_input_state.camera_zoom_out,
        }
    }
    
    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: InputKey) -> bool {
        match key {
            InputKey::MoveLeft => self.current_input_state.move_left,
            InputKey::MoveRight => self.current_input_state.move_right,
            InputKey::MoveUp => self.current_input_state.move_up,
            InputKey::MoveDown => self.current_input_state.move_down,
            InputKey::SailLeft => self.current_input_state.sail_left,
            InputKey::SailRight => self.current_input_state.sail_right,
            InputKey::SailForward => self.current_input_state.sail_forward,
            InputKey::SailBackward => self.current_input_state.sail_backward,
            InputKey::SailNorth => self.current_input_state.sail_north,
            InputKey::SailSouth => self.current_input_state.sail_south,
            InputKey::UseTool => self.current_input_state.use_tool,
            InputKey::SwitchTool => self.current_input_state.switch_tool,
            InputKey::EatFood => self.current_input_state.eat_food,
            InputKey::CollectItem => self.current_input_state.collect_item,
            InputKey::OpenInventory => self.current_input_state.open_inventory,
            InputKey::OpenCrafting => self.current_input_state.open_crafting,
            InputKey::CraftItem => self.current_input_state.craft_item,
            InputKey::QuickItem1 => self.current_input_state.quick_item_1,
            InputKey::QuickItem2 => self.current_input_state.quick_item_2,
            InputKey::QuickItem3 => self.current_input_state.quick_item_3,
            InputKey::QuickItem4 => self.current_input_state.quick_item_4,
            InputKey::QuickItem5 => self.current_input_state.quick_item_5,
            InputKey::QuickItem6 => self.current_input_state.quick_item_6,
            InputKey::QuickItem7 => self.current_input_state.quick_item_7,
            InputKey::QuickItem8 => self.current_input_state.quick_item_8,
            InputKey::QuickItem9 => self.current_input_state.quick_item_9,
            InputKey::QuickItem0 => self.current_input_state.quick_item_0,
            InputKey::CameraZoomIn => self.current_input_state.camera_zoom_in,
            InputKey::CameraZoomOut => self.current_input_state.camera_zoom_out,
        }
    }
    
    /// Get movement vector from input
    pub fn get_movement_vector(&self) -> V3 {
        let mut movement = V3::zero();
        
        if self.current_input_state.move_left {
            movement.x -= 1.0;
        }
        if self.current_input_state.move_right {
            movement.x += 1.0;
        }
        if self.current_input_state.move_up {
            movement.y -= 1.0;
        }
        if self.current_input_state.move_down {
            movement.y += 1.0;
        }
        
        // Don't normalize - this allows for faster diagonal movement
        // and more responsive controls
        movement
    }
    
    /// Get sailing input
    pub fn get_sailing_input(&self) -> SailingInput {
        SailingInput {
            left: self.current_input_state.sail_left,
            right: self.current_input_state.sail_right,
            forward: self.current_input_state.sail_forward,
            backward: self.current_input_state.sail_backward,
            north: self.current_input_state.sail_north,
            south: self.current_input_state.sail_south,
        }
    }
    
    /// Get mouse position in world coordinates
    pub fn get_world_mouse_position(&self, camera_pos: &V2) -> V2 {
        let (screen_w, screen_h) = turbo::resolution();
        let screen_mouse = self.current_input_state.mouse_pos;
        
        V2::new(
            screen_mouse.x - screen_w as f32 * 0.5 + camera_pos.x,
            screen_mouse.y - screen_h as f32 * 0.5 + camera_pos.y,
        )
    }
    
    /// Get mouse position in screen coordinates
    pub fn get_screen_mouse_position(&self) -> V2 {
        self.current_input_state.mouse_pos
    }
    
    /// Check if mouse left button was just pressed
    pub fn is_mouse_left_just_pressed(&self) -> bool {
        self.current_input_state.mouse_left_pressed
    }
    
    /// Check if mouse left button is held
    pub fn is_mouse_left_held(&self) -> bool {
        self.current_input_state.mouse_left_held
    }
    
    /// Check if mouse right button was just pressed
    pub fn is_mouse_right_just_pressed(&self) -> bool {
        self.current_input_state.mouse_right_pressed
    }
}

/// Input keys that can be checked
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKey {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    SailLeft,
    SailRight,
    SailForward,
    SailBackward,
    SailNorth,
    SailSouth,
    UseTool,
    SwitchTool,
    EatFood,
    CollectItem,
    OpenInventory,
    OpenCrafting,
    CraftItem,
    QuickItem1,
    QuickItem2,
    QuickItem3,
    QuickItem4,
    QuickItem5,
    QuickItem6,
    QuickItem7,
    QuickItem8,
    QuickItem9,
    QuickItem0,
    CameraZoomIn,
    CameraZoomOut,
}

/// Current input state
#[turbo::serialize]
pub struct InputState {
    // Movement
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    
    // Raft sailing
    pub sail_left: bool,
    pub sail_right: bool,
    pub sail_forward: bool,
    pub sail_backward: bool,
    pub sail_north: bool,
    pub sail_south: bool,
    
    // Actions
    pub use_tool: bool,
    pub switch_tool: bool,
    pub eat_food: bool,
    pub collect_item: bool,
    pub dive: bool,
    
    // UI
    pub open_inventory: bool,
    pub open_crafting: bool,
    
    // Mouse
    pub mouse_pos: V2,
    pub mouse_left_pressed: bool,
    pub mouse_left_held: bool,
    pub mouse_right_pressed: bool,
    
    // Camera
    pub camera_zoom_in: bool,
    pub camera_zoom_out: bool,

    // Crafting
    pub craft_item: bool,
    pub quick_item_1: bool,
    pub quick_item_2: bool,
    pub quick_item_3: bool,
    pub quick_item_4: bool,
    pub quick_item_5: bool,     
    pub quick_item_6: bool,
    pub quick_item_7: bool,
    pub quick_item_8: bool,
    pub quick_item_9: bool,
    pub quick_item_0: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            sail_left: false,
            sail_right: false,
            sail_forward: false,
            sail_backward: false,
            sail_north: false,
            sail_south: false,
            use_tool: false,
            switch_tool: false,
            eat_food: false,
            collect_item: false,
            dive: false,
            open_inventory: false,
            open_crafting: false,
            craft_item: false,
            quick_item_1: false,
            quick_item_2: false,
            quick_item_3: false,
            quick_item_4: false,
            quick_item_5: false,
            quick_item_6: false,
            quick_item_7: false,
            quick_item_8: false,
            quick_item_9: false,
            quick_item_0: false,
            mouse_pos: V2::zero(),
            mouse_left_pressed: false,
            mouse_left_held: false,
            mouse_right_pressed: false,
            camera_zoom_in: false,
            camera_zoom_out: false,
        }
    }
}

/// Sailing input state
#[derive(Clone)]
pub struct SailingInput {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,
    pub north: bool,
    pub south: bool,
}
