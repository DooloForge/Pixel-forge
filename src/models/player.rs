use crate::math::Vec3 as V3;
use crate::models::ocean::FloatingItemType;
use crate::constants::*;

#[derive(PartialEq)]
#[turbo::serialize]
pub enum Tool {
    Hook,
    Builder,
    Axe,
    Hammer,
}

#[turbo::serialize]
pub struct InventorySlot {
    pub item_type: Option<FloatingItemType>,
    pub quantity: u32,
    pub max_stack: u32,
}

impl InventorySlot {
    pub fn new() -> Self {
        Self {
            item_type: None,
            quantity: 0,
            max_stack: 64, // Default stack size
        }
    }
    
    pub fn new_with_item(item_type: FloatingItemType, quantity: u32) -> Self {
        Self {
            item_type: Some(item_type),
            quantity,
            max_stack: item_type.max_stack_size(),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.item_type.is_none() || self.quantity == 0
    }
    
    pub fn can_add(&self, item_type: FloatingItemType, amount: u32) -> bool {
        if self.is_empty() {
            return amount <= item_type.max_stack_size();
        }
        if let Some(current_type) = self.item_type {
            return current_type == item_type && self.quantity + amount <= self.max_stack;
        }
        false
    }
    
    pub fn add_items(&mut self, item_type: FloatingItemType, amount: u32) -> u32 {
        if self.is_empty() {
            self.item_type = Some(item_type);
            self.max_stack = item_type.max_stack_size();
            self.quantity = amount.min(self.max_stack);
            return amount - self.quantity;
        }
        
        if let Some(current_type) = self.item_type {
            if current_type == item_type {
                let can_add = self.max_stack - self.quantity;
                let added = amount.min(can_add);
                self.quantity += added;
                return amount - added;
            }
        }
        amount // Return all items if can't add
    }
    
    pub fn remove_items(&mut self, amount: u32) -> u32 {
        let removed = amount.min(self.quantity);
        self.quantity -= removed;
        if self.quantity == 0 {
            self.item_type = None;
        }
        removed
    }
}

#[turbo::serialize]
pub struct Inventory {
    pub slots: Vec<InventorySlot>,
    pub max_slots: usize,
    pub selected_slot: Option<usize>,
    pub quick_slots: Vec<Option<usize>>, // References to inventory slots for quick use
}

impl Inventory {
    pub fn new() -> Self {
        let max_slots = 40; // 10 hotbar (0-9) + 30 bag (10-39)
        let mut slots = Vec::with_capacity(max_slots);
        for _ in 0..max_slots {
            slots.push(InventorySlot::new());
        }
        
        Self {
            slots,
            max_slots,
            selected_slot: None,
            quick_slots: vec![None; 10], // retained for compatibility, not used
        }
    }
    
    pub fn add_material(&mut self, material: FloatingItemType, amount: u32) -> bool {
        let mut remaining = amount;
        
        // First try to add to existing stacks
        for slot in &mut self.slots {
            if slot.can_add(material, remaining) {
                remaining = slot.add_items(material, remaining);
                if remaining == 0 {
                    return true;
                }
            }
        }
        
        // If there are still items remaining, try to find empty slots
        if remaining > 0 {
            for slot in &mut self.slots {
                if slot.is_empty() {
                    remaining = slot.add_items(material, remaining);
                    if remaining == 0 {
                        return true;
                    }
                }
            }
        }
        
        // Return true if we managed to add at least some items
        remaining < amount
    }
    
    pub fn get_count(&self, material: FloatingItemType) -> u32 {
        self.slots.iter()
            .filter(|slot| slot.item_type == Some(material))
            .map(|slot| slot.quantity)
            .sum()
    }
    
    pub fn get_total_items(&self) -> u32 {
        self.slots.iter().map(|slot| slot.quantity).sum()
    }
    
    pub fn has_space(&self) -> bool {
        self.slots.iter().any(|slot| slot.is_empty())
    }
    
    pub fn remove_material(&mut self, material: FloatingItemType, amount: u32) -> bool {
        let mut remaining = amount;
        
        for slot in &mut self.slots {
            if let Some(slot_type) = slot.item_type {
                if slot_type == material && remaining > 0 {
                    let removed = slot.remove_items(remaining);
                    remaining -= removed;
                }
            }
        }
        
        remaining == 0
    }
    
    pub fn get_slot(&self, index: usize) -> Option<&InventorySlot> {
        self.slots.get(index)
    }
    
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut InventorySlot> {
        self.slots.get_mut(index)
    }
    
    pub fn swap_slots(&mut self, slot1: usize, slot2: usize) -> bool {
        if slot1 < self.slots.len() && slot2 < self.slots.len() {
            self.slots.swap(slot1, slot2);
            return true;
        }
        false
    }
    
    pub fn move_to_quick_slot(&mut self, inventory_slot: usize, quick_slot: usize) -> bool {
        if quick_slot < self.quick_slots.len() && inventory_slot < self.slots.len() {
            self.quick_slots[quick_slot] = Some(inventory_slot);
            return true;
        }
        false
    }
    
    pub fn use_quick_slot(&mut self, quick_slot: usize) -> Option<(FloatingItemType, u32)> {
        if let Some(Some(slot_index)) = self.quick_slots.get(quick_slot) {
            if let Some(slot) = self.slots.get_mut(*slot_index) {
                if let Some(item_type) = slot.item_type {
                    let used = slot.remove_items(1);
                    if used > 0 {
                        return Some((item_type, used));
                    }
                }
            }
        }
        None
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
        inventory.add_material(FloatingItemType::Coconut, 2);
        // Seed hotbar with up to 10 distinct item types (no repeats by type)
        let mut chosen_indices: Vec<usize> = Vec::new();
        let mut seen_types: std::collections::HashSet<FloatingItemType> = std::collections::HashSet::new();
        for (inv_idx, slot) in inventory.slots.iter().enumerate() {
            if let Some(t) = slot.item_type {
                if !seen_types.contains(&t) {
                    chosen_indices.push(inv_idx);
                    seen_types.insert(t);
                    if chosen_indices.len() >= inventory.quick_slots.len() { break; }
                }
            }
        }
        for (qi, inv_idx) in chosen_indices.into_iter().enumerate() {
            let _ = inventory.move_to_quick_slot(inv_idx, qi);
        }
        
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
    
    pub fn consume_item(&mut self, item_type: FloatingItemType) -> bool {
        if item_type.is_consumable() && self.inventory.remove_material(item_type, 1) {
            self.hunger = (self.hunger + item_type.hunger_restore()).min(100.0);
            self.thirst = (self.thirst + item_type.thirst_restore()).min(100.0);
            return true;
        }
        false
    }
    
    pub fn use_quick_item(&mut self, hotbar_index: usize) -> bool {
        // Hotbar mapped to inventory slots 0..9
        if hotbar_index < 10 {
            if let Some(slot) = self.inventory.get_slot_mut(hotbar_index) {
                if let Some(item_type) = slot.item_type {
                    let used = slot.remove_items(1);
                    if used > 0 {
                        if item_type.is_consumable() {
                            self.hunger = (self.hunger + item_type.hunger_restore()).min(100.0);
                            self.thirst = (self.thirst + item_type.thirst_restore()).min(100.0);
                        }
                        return true;
                    }
                }
            }
        }
        false
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
