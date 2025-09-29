use crate::components::input_system::InputKey;

use super::*;

pub fn update(gm: &mut GameManager) {
    // Update recipe discovery
    if let Some(player) = &gm.game_state.player {
        gm.game_state.crafting_system.discover_recipes(&player.inventory);
    }

    // Handle crafting input (simplified - in a full implementation you'd track selected recipe)
    if gm.input_system.is_key_just_pressed(InputKey::CraftItem) {
        // Try to craft the first available recipe that can be crafted
        let available_recipes = gm.game_state.crafting_system.get_available_recipes();
        if let Some(player) = &mut gm.game_state.player {
            // First, find a craftable recipe id using only immutable access
            let craftable_id: Option<String> = available_recipes
                .into_iter()
                .find(|recipe| gm.game_state.crafting_system.can_craft(&recipe.id, &player.inventory))
                .map(|r| r.id.clone());

            // Then, craft using a separate mutable borrow
            if let Some(id) = craftable_id {
                let _ = gm.game_state.crafting_system.craft_item(&id, &mut player.inventory);
            }
        }
    }
    
    // Quick craft specific items with number keys
    if gm.input_system.is_key_just_pressed(InputKey::QuickItem1) {
        if let Some(player) = &mut gm.game_state.player {
            let _ = gm.game_state.crafting_system.craft_item("planks", &mut player.inventory);
        }
    }
}

