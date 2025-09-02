use super::*;
use crate::math::Vec2 as V2;


/// Handles AI behavior for various game entities
#[turbo::serialize]
pub struct AISystem {
    behavior_trees: std::collections::HashMap<u32, BehaviorTree>,
    pathfinding_cache: std::collections::HashMap<(i32, i32, i32, i32), Vec<V2>>,
}

impl AISystem {
    pub fn new() -> Self {
        Self {
            behavior_trees: std::collections::HashMap::new(),
            pathfinding_cache: std::collections::HashMap::new(),
        }
    }
    
    /// Update AI for all entities
    pub fn update(&mut self, entities: &mut [&mut dyn AIEntity], player_pos: &V2, delta_time: f32) {
        for entity in entities {
            let entity_id = entity.get_id();
            
            // Get or create behavior tree for this entity
            let entity_type = entity.get_entity_type();
            
            // Check if behavior tree exists first
            if !self.behavior_trees.contains_key(&entity_id) {
                let behavior_tree = self.create_behavior_tree(entity_type);
                self.behavior_trees.insert(entity_id, behavior_tree);
            }
            
            // TODO: Update behavior tree and execute actions
        }
    }
    
    /// Create behavior tree based on entity type
    fn create_behavior_tree(&self, entity_type: EntityType) -> BehaviorTree {
        match entity_type {
            EntityType::Fish => self.create_fish_behavior(),
            EntityType::Monster => self.create_monster_behavior(),
            EntityType::Shark => self.create_shark_behavior(),
            EntityType::Coral => self.create_coral_behavior(),
        }
    }
    
    /// Create fish behavior tree
    fn create_fish_behavior(&self) -> BehaviorTree {
        BehaviorTree::new(vec![
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("player_near".to_string()),
                BehaviorNode::Action("flee".to_string()),
            ]),
            BehaviorNode::Fallback(vec![
                BehaviorNode::Condition("random_swim".to_string()),
                BehaviorNode::Action("wander".to_string()),
            ]),
        ])
    }
    
    /// Create monster behavior tree
    fn create_monster_behavior(&self) -> BehaviorTree {
        BehaviorTree::new(vec![
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("player_near".to_string()),
                BehaviorNode::Action("chase".to_string()),
            ]),
            BehaviorNode::Fallback(vec![
                BehaviorNode::Condition("random_patrol".to_string()),
                BehaviorNode::Action("patrol".to_string()),
            ]),
        ])
    }
    
    /// Create shark behavior tree
    fn create_shark_behavior(&self) -> BehaviorTree {
        BehaviorTree::new(vec![
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("player_near".to_string()),
                BehaviorNode::Action("aggressive_chase".to_string()),
            ]),
            BehaviorNode::Fallback(vec![
                BehaviorNode::Condition("random_deep_patrol".to_string()),
                BehaviorNode::Action("deep_patrol".to_string()),
            ]),
        ])
    }
    
    /// Create coral behavior tree
    fn create_coral_behavior(&self) -> BehaviorTree {
        BehaviorTree::new(vec![
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("can_grow".to_string()),
                BehaviorNode::Action("grow".to_string()),
            ]),
        ])
    }
    
    /// Execute AI action
    fn execute_action(&self, entity: &mut dyn AIEntity, action: AIAction, player_pos: &V2, delta_time: f32) {
        match action {
            AIAction::Flee => {
                // Already handled in behavior tree
            },
            AIAction::Chase => {
                // Already handled in behavior tree
            },
            AIAction::Wander => {
                // Already handled in behavior tree
            },
            AIAction::Patrol => {
                // Already handled in behavior tree
            },
            AIAction::AggressiveChase => {
                // Already handled in behavior tree
            },
            AIAction::DeepPatrol => {
                // Already handled in behavior tree
            },
            AIAction::Grow => {
                // Coral growth logic
                if let Some(growth_rate) = entity.get_growth_rate() {
                    entity.grow(growth_rate * delta_time);
                }
            },
        }
    }
    
    /// Find path between two points
    pub fn find_path(&mut self, start: &V2, end: &V2) -> Vec<V2> {
        let start_x = (start.x / 32.0) as i32;
        let start_y = (start.y / 32.0) as i32;
        let end_x = (end.x / 32.0) as i32;
        let end_y = (end.y / 32.0) as i32;
        let cache_key = (start_x, start_y, end_x, end_y);
        
        if let Some(cached_path) = self.pathfinding_cache.get(&cache_key) {
            return cached_path.clone();
        }
        
        // Simple A* pathfinding (simplified for performance)
        let path = self.simple_pathfinding(start, end);
        
        // Cache the result
        self.pathfinding_cache.insert(cache_key, path.clone());
        
        path
    }
    
    /// Simple pathfinding algorithm
    fn simple_pathfinding(&self, start: &V2, end: &V2) -> Vec<V2> {
        let mut path = Vec::new();
        let mut current = start.clone();
        
        while current.distance_to(end) > 10.0 {
            let direction = V2::new(
                end.x - current.x,
                end.y - current.y,
            ).normalize();
            
            current = current.add(direction.scale(10.0));
            path.push(current.clone());
        }
        
        path
    }
}

/// Types of entities that can have AI
#[derive(Clone, Copy, PartialEq)]
pub enum EntityType {
    Fish,
    Monster,
    Shark,
    Coral,
}

/// AI actions that can be performed
#[derive(Clone, Copy, PartialEq)]
pub enum AIAction {
    Flee,
    Chase,
    Wander,
    Patrol,
    AggressiveChase,
    DeepPatrol,
    Grow,
}

/// Trait for entities that can have AI
pub trait AIEntity {
    fn get_id(&self) -> u32;
    fn get_entity_type(&self) -> EntityType;
    fn get_position(&self) -> V2;
    fn set_position(&mut self, pos: V2);
    fn get_velocity(&self) -> V2;
    fn set_velocity(&mut self, vel: V2);
    fn get_growth_rate(&self) -> Option<f32> { None }
    fn grow(&mut self, _amount: f32) {}
}

/// Behavior tree for AI decision making
#[turbo::serialize]
pub struct BehaviorTree {
    nodes: Vec<BehaviorNode>,
}

impl BehaviorTree {
    pub fn new(nodes: Vec<BehaviorNode>) -> Self {
        Self { nodes }
    }
    
    pub fn update(&self, entity: &mut dyn AIEntity, player_pos: &V2, delta_time: f32) -> AIAction {
        // TODO: Implement behavior tree logic
        AIAction::Wander // Default action for now
    }
}

/// Behavior tree node types
#[turbo::serialize]
pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Fallback(Vec<BehaviorNode>),
    Condition(String), // String identifier for condition
    Action(String),    // String identifier for action
}
