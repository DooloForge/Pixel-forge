use crate::models::ocean::FloatingItemType;

#[turbo::serialize]
pub struct CraftingRecipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub ingredients: Vec<(FloatingItemType, u32)>, // (item_type, quantity)
    pub result: (FloatingItemType, u32), // (item_type, quantity)
    pub category: CraftingCategory,
    pub discovered: bool,
    pub unlock_requirements: Vec<FloatingItemType>, // Items needed to discover recipe
}

#[turbo::serialize]
#[derive(PartialEq)]
pub enum CraftingCategory {
    Tools,
    Building,
    Food,
    Storage,
    Survival,
}

impl CraftingCategory {
    pub fn name(&self) -> &str {
        match self {
            CraftingCategory::Tools => "Tools",
            CraftingCategory::Building => "Building",
            CraftingCategory::Food => "Food",
            CraftingCategory::Storage => "Storage",
            CraftingCategory::Survival => "Survival",
        }
    }
}

#[turbo::serialize]
pub struct CraftingSystem {
    pub recipes: Vec<CraftingRecipe>,
    pub discovered_recipes: Vec<String>, // Recipe IDs that have been discovered
}

impl CraftingSystem {
    pub fn new() -> Self { 
        let mut system = Self { 
            recipes: vec![],
            discovered_recipes: vec![],
        };
        system.initialize_recipes();
        system
    }
    
    fn initialize_recipes(&mut self) {
        // Basic Tools
        self.recipes.push(CraftingRecipe {
            id: "fishing_rod".to_string(),
            name: "Fishing Rod".to_string(),
            description: "A basic fishing rod for catching fish".to_string(),
            ingredients: vec![
                (FloatingItemType::Wood, 2),
                (FloatingItemType::Rope, 1),
            ],
            result: (FloatingItemType::Wood, 1), // Placeholder - we'll need fishing rod item
            category: CraftingCategory::Tools,
            discovered: false,
            unlock_requirements: vec![FloatingItemType::Wood, FloatingItemType::Rope],
        });
        
        self.recipes.push(CraftingRecipe {
            id: "spear".to_string(),
            name: "Spear".to_string(),
            description: "A sharp spear for defense and hunting".to_string(),
            ingredients: vec![
                (FloatingItemType::Wood, 1),
                (FloatingItemType::Metal, 1),
            ],
            result: (FloatingItemType::Metal, 1), // Placeholder
            category: CraftingCategory::Tools,
            discovered: false,
            unlock_requirements: vec![FloatingItemType::Wood, FloatingItemType::Metal],
        });
        
        // Building Materials
        self.recipes.push(CraftingRecipe {
            id: "planks".to_string(),
            name: "Wood Planks".to_string(),
            description: "Processed wood planks for building".to_string(),
            ingredients: vec![
                (FloatingItemType::Wood, 3),
            ],
            result: (FloatingItemType::Wood, 5), // More efficient processing
            category: CraftingCategory::Building,
            discovered: true, // Always known
            unlock_requirements: vec![],
        });
        
        self.recipes.push(CraftingRecipe {
            id: "rope_bundle".to_string(),
            name: "Rope Bundle".to_string(),
            description: "Twisted rope for stronger binding".to_string(),
            ingredients: vec![
                (FloatingItemType::Cloth, 2),
            ],
            result: (FloatingItemType::Rope, 1),
            category: CraftingCategory::Building,
            discovered: false,
            unlock_requirements: vec![FloatingItemType::Cloth],
        });
        
        self.recipes.push(CraftingRecipe {
            id: "net".to_string(),
            name: "Fishing Net".to_string(),
            description: "A net for catching multiple fish".to_string(),
            ingredients: vec![
                (FloatingItemType::Rope, 4),
                (FloatingItemType::Cloth, 2),
            ],
            result: (FloatingItemType::Rope, 2), // Placeholder
            category: CraftingCategory::Tools,
            discovered: false,
            unlock_requirements: vec![FloatingItemType::Rope, FloatingItemType::Cloth],
        });
        
        // Storage
        self.recipes.push(CraftingRecipe {
            id: "storage_chest".to_string(),
            name: "Storage Chest".to_string(),
            description: "A chest to store extra items".to_string(),
            ingredients: vec![
                (FloatingItemType::Wood, 8),
                (FloatingItemType::Metal, 2),
                (FloatingItemType::Rope, 1),
            ],
            result: (FloatingItemType::Barrel, 1), // Using barrel as chest placeholder
            category: CraftingCategory::Storage,
            discovered: false,
            unlock_requirements: vec![FloatingItemType::Wood, FloatingItemType::Metal],
        });
        
        // Food Processing
        self.recipes.push(CraftingRecipe {
            id: "dried_fish".to_string(),
            name: "Dried Fish".to_string(),
            description: "Preserved fish that lasts longer".to_string(),
            ingredients: vec![
                (FloatingItemType::Fish, 2),
                (FloatingItemType::Cloth, 1),
            ],
            result: (FloatingItemType::Fish, 3), // More efficient food
            category: CraftingCategory::Food,
            discovered: false,
            unlock_requirements: vec![FloatingItemType::Fish],
        });
        
        // Survival
        self.recipes.push(CraftingRecipe {
            id: "water_collector".to_string(),
            name: "Water Collector".to_string(),
            description: "Collects rainwater for drinking".to_string(),
            ingredients: vec![
                (FloatingItemType::Barrel, 1),
                (FloatingItemType::Cloth, 2),
                (FloatingItemType::Rope, 1),
            ],
            result: (FloatingItemType::Bottle, 3), // Water bottles
            category: CraftingCategory::Survival,
            discovered: false,
            unlock_requirements: vec![FloatingItemType::Barrel],
        });
    }
    
    pub fn can_craft(&self, recipe_id: &str, inventory: &crate::models::player::Inventory) -> bool {
        if let Some(recipe) = self.recipes.iter().find(|r| r.id == recipe_id) {
            if !recipe.discovered && !self.discovered_recipes.contains(&recipe.id) {
                return false;
            }
            
            // Check if player has all required ingredients
            for (item_type, required_amount) in &recipe.ingredients {
                if inventory.get_count(*item_type) < *required_amount {
                    return false;
                }
            }
            return true;
        }
        false
    }
    
    pub fn craft_item(&mut self, recipe_id: &str, inventory: &mut crate::models::player::Inventory) -> bool {
        if !self.can_craft(recipe_id, inventory) {
            return false;
        }
        
        if let Some(recipe) = self.recipes.iter().find(|r| r.id == recipe_id) {
            // Remove ingredients
            for (item_type, required_amount) in &recipe.ingredients {
                if !inventory.remove_material(*item_type, *required_amount) {
                    return false; // This shouldn't happen if can_craft passed
                }
            }
            
            // Add result
            let (result_type, result_amount) = recipe.result;
            inventory.add_material(result_type, result_amount);
            
            return true;
        }
        false
    }
    
    pub fn discover_recipes(&mut self, inventory: &crate::models::player::Inventory) {
        for recipe in &mut self.recipes {
            if !recipe.discovered && !self.discovered_recipes.contains(&recipe.id) {
                // Check if player has unlock requirements
                let mut can_discover = true;
                for required_item in &recipe.unlock_requirements {
                    if inventory.get_count(*required_item) == 0 {
                        can_discover = false;
                        break;
                    }
                }
                
                if can_discover {
                    recipe.discovered = true;
                    self.discovered_recipes.push(recipe.id.clone());
                }
            }
        }
    }
    
    pub fn get_available_recipes(&self) -> Vec<&CraftingRecipe> {
        self.recipes.iter()
            .filter(|r| r.discovered || self.discovered_recipes.contains(&r.id))
            .collect()
    }
    
    pub fn get_recipes_by_category(&self, category: CraftingCategory) -> Vec<&CraftingRecipe> {
        self.get_available_recipes().into_iter()
            .filter(|r| r.category == category)
            .collect()
    }
}


