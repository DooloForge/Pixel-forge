use crate::math::Vec3 as V3;
use crate::math::Vec2 as V2;

#[turbo::serialize]
pub struct Hook {
    pub position: V3,
    pub velocity: V3,
    pub direction: V2,
    pub length: f32,
    pub max_length: f32,
    pub speed: f32,
    pub state: HookState,
    pub attached_items: Vec<u32>, // Entity IDs of attached items
    pub owner_id: u32, // Player entity ID
}

#[turbo::serialize]
#[derive(PartialEq, Copy)]
pub enum HookState {
    Retracted,    // Hook is at player
    Extending,    // Hook is moving away from player
    Extended,     // Hook has reached max length
    Retracting,   // Hook is returning to player
}

impl Hook {
    pub fn new(owner_id: u32) -> Self {
        Self {
            position: V3::zero(),
            velocity: V3::zero(),
            direction: V2::new(1.0, 0.0), // Default right direction
            length: 0.0,
            max_length: 100.0,
            speed: 80.0, // Much faster speed - 20 units per second
            state: HookState::Retracted,
            attached_items: Vec::new(),
            owner_id,
        }
    }
    
    pub fn launch(&mut self, start_pos: V3, direction: V2) {
        self.position = start_pos;
        // Guard against zero-length direction to avoid NaNs
        let dir_len = direction.length();
        let safe_dir = if dir_len < 1e-3 { V2::new(1.0, 0.0) } else { direction.normalize() };
        self.direction = safe_dir;
        self.velocity = V3::new(safe_dir.x * self.speed, safe_dir.y * self.speed, 0.0);
        self.length = 0.0;
        self.state = HookState::Extending;
        self.attached_items.clear();
    }
    
    pub fn update(&mut self, delta_time: f32, player_pos: V3) -> bool {
        match self.state {
            HookState::Retracted => {
                // Hook is at player, do nothing
                false
            },
            HookState::Extending => {
                // Move hook away from player
                let move_distance = self.speed * delta_time;
                self.position = self.position.add(self.velocity.scale(delta_time));
                self.length += move_distance;
                
                // Check if we've reached max length
                if self.length >= self.max_length {
                    self.state = HookState::Extended;
                }
                false
            },
            HookState::Extended => {
                // Hook is at max length, start retracting
                self.state = HookState::Retracting;
                false
            },
            HookState::Retracting => {
                // Move hook back towards player
                let to_player = player_pos.sub(self.position);
                let distance_to_player = to_player.length();
                
                if distance_to_player <= 10.0 {
                    // Hook has returned to player
                    self.state = HookState::Retracted;
                    self.length = 0.0;
                    return true; // Hook has completed its cycle
                }
                
                // Move towards player (faster return speed)
                let direction_to_player = to_player.normalize();
                self.velocity = direction_to_player.scale(self.speed * 1.5); // 50% faster return
                self.position = self.position.add(self.velocity.scale(delta_time));
                self.length = distance_to_player;
                false
            }
        }
    }
    
    pub fn attach_item(&mut self, item_id: u32) {
        if !self.attached_items.contains(&item_id) {
            self.attached_items.push(item_id);
        }
    }
    
    pub fn detach_all_items(&mut self) -> Vec<u32> {
        let items = self.attached_items.clone();
        self.attached_items.clear();
        items
    }
    
    pub fn is_active(&self) -> bool {
        self.state != HookState::Retracted
    }
    
    pub fn get_hook_tip_position(&self) -> V3 {
        // position already tracks the hook head; return it directly
        self.position
    }
}
