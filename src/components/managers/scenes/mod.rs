pub mod main_menu;
pub mod playing;
pub mod inventory;
pub mod crafting;
pub mod paused;

use crate::math::Vec2 as V2;
use crate::components::managers::game_manager::GameManager;
use crate::components::systems::spawn_system::SpawnType;