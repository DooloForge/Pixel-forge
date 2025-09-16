use std::collections::HashMap;
use turbo::serialize;

/// Handles input mapping and key bindings
#[turbo::serialize]
pub struct InputMapping {
    key_bindings: HashMap<String, String>,
    default_bindings: HashMap<String, String>,
}

impl InputMapping {
    pub fn new() -> Self {
        let mut default_bindings = HashMap::new();
        default_bindings.insert("move_left".to_string(), "A".to_string());
        default_bindings.insert("move_right".to_string(), "D".to_string());
        default_bindings.insert("move_up".to_string(), "W".to_string());
        default_bindings.insert("move_down".to_string(), "S".to_string());
        default_bindings.insert("sail_left".to_string(), "J".to_string());
        default_bindings.insert("sail_right".to_string(), "L".to_string());
        default_bindings.insert("sail_forward".to_string(), "I".to_string());
        default_bindings.insert("sail_backward".to_string(), "K".to_string());
        default_bindings.insert("sail_north".to_string(), "Q".to_string());
        default_bindings.insert("sail_south".to_string(), "E".to_string());
        default_bindings.insert("use_tool".to_string(), "MOUSE_LEFT".to_string());
        default_bindings.insert("switch_tool".to_string(), "E".to_string());
        default_bindings.insert("eat_food".to_string(), "F".to_string());
        default_bindings.insert("collect_item".to_string(), "G".to_string());
        default_bindings.insert("open_inventory".to_string(), "I".to_string());
        default_bindings.insert("open_crafting".to_string(), "C".to_string());
        
        Self {
            key_bindings: default_bindings.clone(),
            default_bindings,
        }
    }
    
    /// Get key binding for an action
    pub fn get_key_binding(&self, action: &str) -> Option<&String> {
        self.key_bindings.get(action)
    }
    
    /// Set key binding for an action
    pub fn set_key_binding(&mut self, action: &str, key: &str) {
        self.key_bindings.insert(action.to_string(), key.to_string());
    }
    
    /// Reset key binding to default
    pub fn reset_key_binding(&mut self, action: &str) {
        if let Some(default_key) = self.default_bindings.get(action) {
            self.key_bindings.insert(action.to_string(), default_key.clone());
        }
    }
    
    /// Reset all key bindings to defaults
    pub fn reset_all_key_bindings(&mut self) {
        self.key_bindings = self.default_bindings.clone();
    }
    
    /// Get all current key bindings
    pub fn get_all_key_bindings(&self) -> &HashMap<String, String> {
        &self.key_bindings
    }
    
    /// Save key bindings to file
    pub fn save_key_bindings(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement file saving
        Ok(())
    }
    
    /// Load key bindings from file
    pub fn load_key_bindings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement file loading
        Ok(())
    }
}
