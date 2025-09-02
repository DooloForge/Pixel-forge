/// Manages different game scenes and transitions
#[turbo::serialize]
pub struct SceneManager {
    current_scene: SceneType,
    previous_scene: Option<SceneType>,
    scene_transitions: Vec<SceneTransition>,
    scene_data: std::collections::HashMap<SceneType, SceneData>,
}

impl SceneManager {
    pub fn new() -> Self {
        let mut scene_data = std::collections::HashMap::new();
        
        // Initialize scene data
        scene_data.insert(SceneType::MainMenu, SceneData::new("Main Menu"));
        scene_data.insert(SceneType::Playing, SceneData::new("Playing"));
        scene_data.insert(SceneType::Inventory, SceneData::new("Inventory"));
        scene_data.insert(SceneType::Crafting, SceneData::new("Crafting"));
        scene_data.insert(SceneType::Paused, SceneData::new("Paused"));
        
        Self {
            current_scene: SceneType::MainMenu,
            previous_scene: None,
            scene_transitions: Vec::new(),
            scene_data,
        }
    }
    
    /// Change to a new scene
    pub fn change_scene(&mut self, new_scene: SceneType) -> bool {
        if self.can_transition_to(&new_scene) {
            self.previous_scene = Some(self.current_scene);
            self.current_scene = new_scene;
            
            // Add transition effect
            self.add_transition(SceneTransition::new(
                self.previous_scene.unwrap(),
                new_scene,
            ));
            
            true
        } else {
            false
        }
    }
    
    /// Check if transition to scene is allowed
    fn can_transition_to(&self, scene: &SceneType) -> bool {
        match (self.current_scene, scene) {
            // Main menu can go to playing
            (SceneType::MainMenu, SceneType::Playing) => true,
            
            // Playing can go to inventory, crafting, or paused
            (SceneType::Playing, SceneType::Inventory) => true,
            (SceneType::Playing, SceneType::Crafting) => true,
            (SceneType::Playing, SceneType::Paused) => true,
            
            // Inventory and crafting can return to playing
            (SceneType::Inventory, SceneType::Playing) => true,
            (SceneType::Crafting, SceneType::Playing) => true,
            
            // Paused can return to playing
            (SceneType::Paused, SceneType::Playing) => true,
            
            // Default: no transition allowed
            _ => false,
        }
    }
    
    /// Get current scene
    pub fn get_current_scene(&self) -> SceneType {
        self.current_scene
    }
    
    /// Get previous scene
    pub fn get_previous_scene(&self) -> Option<SceneType> {
        self.previous_scene
    }
    
    /// Go back to previous scene
    pub fn go_back(&mut self) -> bool {
        if let Some(previous) = self.previous_scene {
            self.change_scene(previous)
        } else {
            false
        }
    }
    
    /// Add scene transition
    fn add_transition(&mut self, transition: SceneTransition) {
        self.scene_transitions.push(transition);
    }
    
    /// Update scene transitions
    pub fn update_transitions(&mut self, delta_time: f32) {
        self.scene_transitions.retain_mut(|transition| {
            transition.update(delta_time)
        });
    }
    
    /// Get scene data
    pub fn get_scene_data(&self, scene: &SceneType) -> Option<&SceneData> {
        self.scene_data.get(scene)
    }
    
    /// Get current scene data
    pub fn get_current_scene_data(&self) -> Option<&SceneData> {
        self.scene_data.get(&self.current_scene)
    }
    
    /// Set scene data
    pub fn set_scene_data(&mut self, scene: SceneType, data: SceneData) {
        self.scene_data.insert(scene, data);
    }
    
    /// Check if scene is currently transitioning
    pub fn is_transitioning(&self) -> bool {
        !self.scene_transitions.is_empty()
    }
    
    /// Get active transitions
    pub fn get_active_transitions(&self) -> &[SceneTransition] {
        &self.scene_transitions
    }
}

/// Types of scenes in the game
#[derive(Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[turbo::serialize]
pub enum SceneType {
    MainMenu,
    Playing,
    Inventory,
    Crafting,
    Paused,
}

/// Data associated with a scene
#[turbo::serialize]
pub struct SceneData {
    pub name: String,
    pub is_initialized: bool,
    pub entities: Vec<String>, // Entity IDs in this scene
    pub ui_elements: Vec<String>, // UI element IDs in this scene
}

impl SceneData {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_initialized: false,
            entities: Vec::new(),
            ui_elements: Vec::new(),
        }
    }
    
    /// Mark scene as initialized
    pub fn initialize(&mut self) {
        self.is_initialized = true;
    }
    
    /// Add entity to scene
    pub fn add_entity(&mut self, entity_id: &str) {
        if !self.entities.contains(&entity_id.to_string()) {
            self.entities.push(entity_id.to_string());
        }
    }
    
    /// Remove entity from scene
    pub fn remove_entity(&mut self, entity_id: &str) {
        self.entities.retain(|id| id != entity_id);
    }
    
    /// Add UI element to scene
    pub fn add_ui_element(&mut self, ui_id: &str) {
        if !self.ui_elements.contains(&ui_id.to_string()) {
            self.ui_elements.push(ui_id.to_string());
        }
    }
    
    /// Remove UI element from scene
    pub fn remove_ui_element(&mut self, ui_id: &str) {
        self.ui_elements.retain(|id| id != ui_id);
    }
}

/// Scene transition effect using Turbo's tween system
#[turbo::serialize]
pub struct SceneTransition {
    pub from_scene: SceneType,
    pub to_scene: SceneType,
    pub progress: f32,
    pub duration: f32,
    pub transition_type: TransitionType,
}

impl SceneTransition {
    pub fn new(from_scene: SceneType, to_scene: SceneType) -> Self {
        Self {
            from_scene,
            to_scene,
            progress: 0.0,
            duration: 0.5, // 0.5 seconds
            transition_type: TransitionType::Fade,
        }
    }
    
    /// Update transition
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.progress += delta_time / self.duration;
        self.progress < 1.0
    }
    
    /// Get transition progress (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        self.progress.min(1.0)
    }
    
    /// Get transition alpha for rendering
    pub fn get_alpha(&self) -> f32 {
        match self.transition_type {
            TransitionType::Fade => {
                if self.progress < 0.5 {
                    // Fade out: 0.0 -> 1.0
                    self.progress * 2.0
                } else {
                    // Fade in: 1.0 -> 0.0
                    2.0 - self.progress * 2.0
                }
            }
            TransitionType::Slide => self.progress,
            TransitionType::Zoom => self.progress,
            TransitionType::Dissolve => self.progress,
        }
    }
    
    /// Set transition duration
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }
}

/// Types of scene transitions
#[derive(Copy, PartialEq)]
#[turbo::serialize]
pub enum TransitionType {
    Fade,
    Slide,
    Zoom,
    Dissolve,
}



