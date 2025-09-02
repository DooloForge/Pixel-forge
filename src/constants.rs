// Physics constants
pub const GRAVITY: f32 = 0.5;
pub const FRICTION: f32 = 0.98;
pub const BOUNCE_DAMPING: f32 = 0.7;

// Gameplay constants
pub const PLAYER_RADIUS: f32 = 10.0;
pub const PLAYER_SPEED: f32 = 3.5;
pub const BULLET_RADIUS: f32 = 3.0;
pub const BULLET_SPEED: f32 = 8.0;
pub const SHOOT_INTERVAL_TICKS: u32 = 20;
pub const PARTICLE_LIFETIME_TICKS: u32 = 30;

// Pixel walls
pub const PIXEL_SIZE: f32 = 3.0;
pub const PIXEL_WALL_COLOR: u32 = 0xff808080;

// World generation
pub const CHUNK_SIZE: usize = 32;
pub const RENDER_DISTANCE: i32 = 3;

// Terrain durability
pub const SAND_HP: f32 = 50.0;
pub const STONE_HP: f32 = 120.0;
pub const IRON_HP: f32 = 180.0;
pub const WATER_HP: f32 = 1.0;

// Player survival and diving
pub const SURFACE_DEPTH: i32 = 0;
pub const SHALLOW_DEPTH: i32 = -50;
pub const DEEP_DEPTH: i32 = -150;
pub const ABYSS_DEPTH: i32 = -300;

pub const MAX_BREATH: f32 = 100.0;
pub const BREATH_LOSS_RATE: f32 = 15.0;      // per second while diving
pub const BREATH_RECOVERY_RATE: f32 = 25.0;  // per second on surface

// Depth tint overlays (RGBA)
pub const SURFACE_TINT: u32 = 0x87CEEB22; // LightSkyBlue, subtle alpha
pub const SHALLOW_TINT: u32 = 0x4169E144; // RoyalBlue
pub const DEEP_TINT: u32 = 0x001F3F66;    // Very dark blue
pub const ABYSS_TINT: u32 = 0x000A1A88;   // Almost black blue

// Entity colors (RGBA)
pub const PLAYER_ON_RAFT_COLOR: u32 = 0xFFD27AFF;   // Warm light skin/orange
pub const PLAYER_SWIMMING_COLOR: u32 = 0x87CEFAFF;  // Light blue underwater
pub const RAFT_WOOD_FLOOR_COLOR: u32 = 0xC2A36BFF;  // Wood plank color
pub const PARTICLE_COLOR: u32 = 0xFFFFFFFF;         // White particle

// UI colors (RGBA)
pub const UI_TEXT_WHITE: u32 = 0xFFFFFFFF;
pub const UI_TEXT_RED: u32 = 0xFF5555FF;
pub const UI_TEXT_ORANGE: u32 = 0xFFA500FF;
pub const UI_TEXT_BLUE: u32 = 0x1E90FFFF;  // DodgerBlue
pub const UI_TEXT_GRAY: u32 = 0xAAAAAAFF;
pub const UI_PANEL_BG: u32 = 0x223344CC;   // Semi-transparent panel
