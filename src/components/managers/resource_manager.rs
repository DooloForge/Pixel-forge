use std::collections::HashMap;

/// Manages game resources like textures, sounds, and data
#[turbo::serialize]
pub struct ResourceManager {
    textures: HashMap<String, TextureResource>,
    sounds: HashMap<String, SoundResource>,
    data_files: HashMap<String, DataResource>,
    resource_cache: HashMap<String, CachedResource>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            sounds: HashMap::new(),
            data_files: HashMap::new(),
            resource_cache: HashMap::new(),
        }
    }
    
    /// Register a texture resource
    pub fn register_texture(&mut self, name: &str, path: &str, width: u32, height: u32) {
        let texture = TextureResource::new(name, path, width, height);
        self.textures.insert(name.to_string(), texture);
    }
    
    /// Register a sound resource
    pub fn register_sound(&mut self, name: &str, path: &str, duration: f32) {
        let sound = SoundResource::new(name, path, duration);
        self.sounds.insert(name.to_string(), sound);
    }
    
    /// Register a data file resource
    pub fn register_data_file(&mut self, name: &str, path: &str, file_type: DataFileType) {
        let data = DataResource::new(name, path, file_type);
        self.data_files.insert(name.to_string(), data);
    }
    
    /// Get texture resource
    pub fn get_texture(&self, name: &str) -> Option<&TextureResource> {
        self.textures.get(name)
    }
    
    /// Get sound resource
    pub fn get_sound(&self, name: &str) -> Option<&SoundResource> {
        self.sounds.get(name)
    }
    
    /// Get data file resource
    pub fn get_data_file(&self, name: &str) -> Option<&DataResource> {
        self.data_files.get(name)
    }
    
    /// Load a resource into cache
    pub fn load_resource(&mut self, name: &str) -> bool {
        // Check if already cached
        if self.resource_cache.contains_key(name) {
            return true;
        }
        
        // Try to load based on resource type
        if let Some(texture) = self.textures.get(name) {
            let texture_name = texture.name.clone();
            if self.load_texture(&texture_name) {
                return true;
            }
        } else if let Some(sound) = self.sounds.get(name) {
            let sound_name = sound.name.clone();
            if self.load_sound(&sound_name) {
                return true;
            }
        } else if let Some(data) = self.data_files.get(name) {
            let data_name = data.name.clone();
            if self.load_data_file(&data_name) {
                return true;
            }
        }
        
        false
    }
    
    /// Load texture into cache
    fn load_texture(&mut self, texture: &String) -> bool {
        // TODO: Implement actual texture loading
        let cached = CachedResource::Texture {
            name: texture.clone(),
            data: vec![], // Placeholder for actual texture data
            loaded: true,
        };
        
        self.resource_cache.insert(texture.clone(), cached);
        true
    }
    
    /// Load sound into cache
    fn load_sound(&mut self, sound: &String) -> bool {
        // TODO: Implement actual sound loading
        let cached = CachedResource::Sound {
            name: sound.clone(),
            data: vec![], // Placeholder for actual sound data
            loaded: true,
        };
        
        self.resource_cache.insert(sound.clone(), cached);
        true
    }
    
    /// Load data file into cache
    fn load_data_file(&mut self, data: &String) -> bool {
        // TODO: Implement actual data file loading
        let cached = CachedResource::Data {
            name: data.clone(),
            data: vec![], // Placeholder for actual data
            loaded: true,
        };
        
        self.resource_cache.insert(data.clone(), cached);
        true
    }
    
    /// Unload a resource from cache
    pub fn unload_resource(&mut self, name: &str) -> bool {
        self.resource_cache.remove(name).is_some()
    }
    
    /// Check if resource is loaded
    pub fn is_resource_loaded(&self, name: &str) -> bool {
        self.resource_cache.contains_key(name)
    }
    
    /// Get cached resource
    pub fn get_cached_resource(&self, name: &str) -> Option<&CachedResource> {
        self.resource_cache.get(name)
    }
    
    /// Clear all cached resources
    pub fn clear_cache(&mut self) {
        self.resource_cache.clear();
    }
    
    /// Get memory usage of cached resources
    pub fn get_cache_memory_usage(&self) -> usize {
        self.resource_cache.values().map(|resource| {
            match resource {
                CachedResource::Texture { data, .. } => data.len(),
                CachedResource::Sound { data, .. } => data.len(),
                CachedResource::Data { data, .. } => data.len(),
            }
        }).sum()
    }
    
    /// Preload a list of resources
    pub fn preload_resources(&mut self, resource_names: &[String]) -> Vec<String> {
        let mut failed_resources = Vec::new();
        
        for name in resource_names {
            if !self.load_resource(name) {
                failed_resources.push(name.clone());
            }
        }
        
        failed_resources
    }
}

/// Texture resource information
#[turbo::serialize]
pub struct TextureResource {
    pub name: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

impl TextureResource {
    pub fn new(name: &str, path: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            width,
            height,
            format: TextureFormat::RGBA8,
        }
    }
}

/// Sound resource information
#[turbo::serialize]
pub struct SoundResource {
    pub name: String,
    pub path: String,
    pub duration: f32,
    pub format: SoundFormat,
}

impl SoundResource {
    pub fn new(name: &str, path: &str, duration: f32) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            duration,
            format: SoundFormat::WAV,
        }
    }
}

/// Data file resource information
#[turbo::serialize]
pub struct DataResource {
    pub name: String,
    pub path: String,
    pub file_type: DataFileType,
}

impl DataResource {
    pub fn new(name: &str, path: &str, file_type: DataFileType) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            file_type,
        }
    }
}

/// Cached resource data
#[turbo::serialize]
pub enum CachedResource {
    Texture {
        name: String,
        data: Vec<u8>,
        loaded: bool,
    },
    Sound {
        name: String,
        data: Vec<u8>,
        loaded: bool,
    },
    Data {
        name: String,
        data: Vec<u8>,
        loaded: bool,
    },
}

/// Texture formats
#[derive(Copy, PartialEq)]
#[turbo::serialize]
pub enum TextureFormat {
    RGBA8,
    RGB8,
    RG8,
    R8,
}

/// Sound formats
#[derive(Copy, PartialEq)]
#[turbo::serialize]
pub enum SoundFormat {
    WAV,
    MP3,
    OGG,
}

/// Data file types
#[derive(Copy, PartialEq)]
#[turbo::serialize]
pub enum DataFileType {
    JSON,
    XML,
    CSV,
    Binary,
}
