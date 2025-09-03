use crate::math::Vec3 as V3;
use crate::models::ocean::FloatingItemType;
use crate::constants::*;
use std::collections::HashMap;

#[derive(PartialEq)]
#[turbo::serialize]
pub enum Tool {
    Hook,
    Builder,
    Axe,
    Hammer,
}

#[turbo::serialize]
pub struct Inventory {
    pub materials: HashMap<FloatingItemType, u32>,
    pub max_capacity: u32,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
            max_capacity: 100,
        }
    }
    
    pub fn add_material(&mut self, material: FloatingItemType, amount: u32) -> bool {
        let current_total = self.get_total_items();
        if current_total + amount <= self.max_capacity {
            *self.materials.entry(material).or_insert(0) += amount;
            true
        } else {
            false
        }
    }
    
    pub fn get_count(&self, material: FloatingItemType) -> u32 {
        self.materials.get(&material).copied().unwrap_or(0)
    }
    
    pub fn get_total_items(&self) -> u32 {
        self.materials.values().sum()
    }
    
    pub fn has_space(&self) -> bool {
        self.get_total_items() < self.max_capacity
    }
    
    pub fn remove_material(&mut self, material: FloatingItemType, amount: u32) -> bool {
        if let Some(current) = self.materials.get_mut(&material) {
            if *current >= amount {
                *current -= amount;
                if *current == 0 {
                    self.materials.remove(&material);
                }
                return true;
            }
        }
        false
    }
}

#[turbo::serialize]
pub struct Player {
    pub pos: V3,
    pub vel: V3,
    pub on_raft: bool,
    pub facing: f32,
    pub current_tool: Tool,
    pub inventory: Inventory,
    pub action_cooldown: u32,
    pub hunger: f32,
    pub thirst: f32,
    pub health: f32,
    pub depth: i32,         // Current depth (0 = surface, negative = underwater)
    pub breath: f32,        // Oxygen/breath level
    pub is_diving: bool,    // Whether player is underwater
}

impl Player {
    pub fn new(pos: V3) -> Self { 
        let mut inventory = Inventory::new();
        // Give player some starting materials
        inventory.add_material(FloatingItemType::Wood, 10);
        inventory.add_material(FloatingItemType::Plastic, 5);
        
        Self { 
            pos, 
            vel: V3::zero(), 
            on_raft: true, 
            facing: 0.0,
            current_tool: Tool::Hook,
            inventory,
            action_cooldown: 0,
            hunger: 100.0,
            thirst: 100.0,
            health: 100.0,
            depth: SURFACE_DEPTH,
            breath: MAX_BREATH,
            is_diving: false,
        } 
    }
    
    pub fn switch_tool(&mut self) {
        self.current_tool = match self.current_tool {
            Tool::Hook => Tool::Builder,
            Tool::Builder => Tool::Axe,
            Tool::Axe => Tool::Hammer,
            Tool::Hammer => Tool::Hook,
        };
    }
    
    pub fn update_cooldowns(&mut self) {
        if self.action_cooldown > 0 {
            self.action_cooldown -= 1;
        }
        
        // Update breath system
        if self.is_diving {
            // Lose breath underwater
            self.breath -= BREATH_LOSS_RATE / 60.0; // Convert to per-frame rate
            if self.breath <= 0.0 {
                self.breath = 0.0;
                self.health -= 0.5; // Take damage when out of breath
            }
        } else {
            // Recover breath on surface
            self.breath += BREATH_RECOVERY_RATE / 60.0;
            self.breath = self.breath.min(MAX_BREATH);
        }
        
        // Decrease survival stats over time
        self.hunger -= 0.02; // Decrease faster
        self.thirst -= 0.03; // Thirst decreases fastest
        
        // Health decreases if hungry or thirsty
        if self.hunger <= 0.0 || self.thirst <= 0.0 {
            self.health -= 0.1;
        }
        
        // Clamp values
        self.hunger = self.hunger.max(0.0);
        self.thirst = self.thirst.max(0.0);
        self.health = self.health.max(0.0).min(100.0);
    }
    
    pub fn can_use_hook(&self) -> bool {
        self.current_tool == Tool::Hook && self.action_cooldown == 0
    }
    
    pub fn can_build(&self) -> bool {
        self.current_tool == Tool::Builder && 
        self.inventory.get_count(FloatingItemType::Wood) > 0
    }
    
    pub fn start_action(&mut self) {
        self.action_cooldown = 15; // Cooldown in frames
    }
    
    pub fn eat_food(&mut self, food_type: FloatingItemType) {
        match food_type {
            FloatingItemType::Coconut => {
                self.hunger = (self.hunger + 25.0).min(100.0);
                self.thirst = (self.thirst + 15.0).min(100.0);
            },
            FloatingItemType::Fish => {
                self.hunger = (self.hunger + 40.0).min(100.0);
            },
            FloatingItemType::Seaweed => {
                self.hunger = (self.hunger + 15.0).min(100.0);
                self.health = (self.health + 5.0).min(100.0);
            },
            _ => {},
        }
    }
    
    pub fn drink_water(&mut self) {
        self.thirst = (self.thirst + 50.0).min(100.0);
    }
    
    pub fn dive_down(&mut self) {
        if !self.on_raft && self.breath > 20.0 { // Need some breath to dive
            self.depth = (self.depth - 10).max(ABYSS_DEPTH);
            self.is_diving = self.depth < SURFACE_DEPTH;
        }
    }
    
    pub fn surface_up(&mut self) {
        self.depth = (self.depth + 10).min(SURFACE_DEPTH);
        self.is_diving = self.depth < SURFACE_DEPTH;
    }
    
    pub fn get_depth_name(&self) -> &'static str {
        match self.depth {
            SURFACE_DEPTH => "Surface",
            d if d >= SHALLOW_DEPTH => "Shallow Water",
            d if d >= DEEP_DEPTH => "Deep Ocean", 
            _ => "Abyss",
        }
    }
    
    pub fn get_depth_tint(&self) -> u32 {
        match self.depth {
            SURFACE_DEPTH => SURFACE_TINT,
            d if d >= SHALLOW_DEPTH => SHALLOW_TINT,
            d if d >= DEEP_DEPTH => DEEP_TINT,
            _ => ABYSS_TINT,
        }
    }
}
