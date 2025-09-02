use turbo::*;
mod constants;
mod math;
mod models;
mod components;

use crate::components::*;

// Main game state using the new component system
#[turbo::game]
struct GameState {
    game_manager: GameManager,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            game_manager: GameManager::new(),
        }
    }
    
    pub fn update(&mut self) {
        // Update the game manager which handles all systems
        self.game_manager.update();
    }
}