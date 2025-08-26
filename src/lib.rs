use turbo::*;
mod constants;
mod math;
mod models;

use constants::*;
use crate::math::Vec2 as V2;
use crate::models::{PhysicsBody as Body, Player, Bullet, Particle, WallGrid, MonsterGrid, Debris};

// Main game state
#[turbo::game]
struct GameState {
    physics_bodies: Vec<Body>,
    spawn_count: u32,
    gravity_enabled: bool,
    player: Player,
    bullets: Vec<Bullet>,
    particles: Vec<Particle>,
    debris: Vec<Debris>,
    wall_grid: WallGrid,
    monster: MonsterGrid,
    pending_fall: Vec<V2>,
}

impl GameState {
    pub fn new() -> Self {
        let (w, h) = resolution();
        let w = w as f32; let h = h as f32;
        let grid_h = 24.0;
        // Smaller monster
        let monster_cols = 24usize;
        let monster_rows = 20usize;
        Self {
            physics_bodies: Vec::new(),
            spawn_count: 0,
            gravity_enabled: true,
            player: Player::new(V2::new(w * 0.5, h * 0.7)),
            bullets: Vec::new(),
            particles: Vec::new(),
            debris: Vec::new(),
            // Wall spans full width at the bottom
            wall_grid: WallGrid::new(0.0, h - grid_h, w, grid_h, 0xff5a5a5a),
            // Place monster so its feet rest on the wall
            monster: MonsterGrid::new(w * 0.6, h - grid_h - monster_rows as f32 * PIXEL_SIZE, monster_cols, monster_rows, 0xff66f07a),
            pending_fall: Vec::new(),
        }
    }
    
    pub fn update(&mut self) {
        // Input
        let keyboard = keyboard::get();
        let mouse = mouse::screen();
        
        // Player physics movement
        if keyboard.key_a().pressed() { self.player.vel.x -= 0.6; }
        if keyboard.key_d().pressed() { self.player.vel.x += 0.6; }
        if keyboard.key_w().just_pressed() && self.player.grounded { self.player.vel.y = -8.0; }
        // gravity and damping
        self.player.vel.y += GRAVITY;
        self.player.vel.x *= 0.90;
        self.player.pos = self.player.pos.add(self.player.vel.clone());
        
        // collide player with ground (wall top)
        let ground_y = self.wall_grid.origin.y;
        if self.player.pos.y + PLAYER_RADIUS > ground_y {
            self.player.pos.y = ground_y - PLAYER_RADIUS;
            self.player.vel.y = 0.0;
            self.player.grounded = true;
        } else { self.player.grounded = false; }
        
        // Aim and shoot
        let (mx, my) = mouse.xy();
        let aim_vec = V2::new(mx as f32 - self.player.pos.x, my as f32 - self.player.pos.y);
        self.player.facing = aim_vec.y.atan2(aim_vec.x);
        if mouse.left.just_pressed() {
            let muzzle = self.player.pos.add(V2::new(PLAYER_RADIUS * self.player.facing.cos(), PLAYER_RADIUS * self.player.facing.sin()));
            self.bullets.push(Bullet::new(muzzle, self.player.facing));
        }
        
        // Update monster
        let min_x = 0.0; let max_x = (resolution().0 as f32);
        self.monster.update(ground_y, min_x, max_x, GRAVITY);
        
        // Update bullets and apply to grids
        let mut i = 0;
        while i < self.bullets.len() {
            if !self.bullets[i].update() { self.bullets.remove(i); continue; }
            let mut hit = false;
            {
                let bx = self.bullets[i].pos.x; let by = self.bullets[i].pos.y;
                // Directly destroy wall pixels and convert to debris for immediate fall
                let destroyed_pixels = self.wall_grid.destroy_circle_to_debris(bx, by, BULLET_RADIUS * 2.0);
                if !destroyed_pixels.is_empty() {
                    hit = true; let impact = V2::new(bx, by); self.spawn_impact_particles(&impact, destroyed_pixels.len());
                    for p in destroyed_pixels {
                        let sz = PIXEL_SIZE;
                        let vel = V2::new((random::f32()-0.5)*1.0, -0.5 + (random::f32()-0.5)*0.6);
                        self.debris.push(Debris::new(V2::new(p.x - sz*0.5, p.y - sz*0.5), vel, sz, sz, 0xff7a7a7a));
                    }
                    // then collapse unsupported
                    let fallen = self.wall_grid.pop_unsupported();
                    self.pending_fall.extend(fallen);
                }
            }
            if !hit {
                let bx = self.bullets[i].pos.x; let by = self.bullets[i].pos.y;
                let destroyed = self.monster.hit_circle(bx, by, BULLET_RADIUS * 2.0);
                if destroyed > 0 { 
                    hit = true; 
                    let impact = V2::new(bx, by); self.spawn_impact_particles(&impact, destroyed);
                    let fallen = self.monster.pop_unsupported();
                    self.pending_fall.extend(fallen);
                }
            }
            if hit { self.bullets.remove(i); } else { i += 1; }
        }
        
        // Convert a few queued pixels into chunk debris each frame
        let (screen_w, screen_h) = resolution();
        let screen_h = screen_h as f32;
        let screen_ground_y = screen_h; // allow debris to fall to bottom of screen, independent of wall top
        let emit = 24usize.min(self.pending_fall.len());
        for _ in 0..emit {
            if let Some(p) = self.pending_fall.pop() {
                let sz = PIXEL_SIZE;
                let vel = V2::new((random::f32()-0.5)*1.2, -0.6 + (random::f32()-0.5)*0.6);
                self.debris.push(Debris::new(V2::new(p.x - sz*0.5, p.y - sz*0.5), vel, sz, sz, 0xff88f090));
            }
        }
        
        // Update debris; use screen bottom as global ground to ensure they fall to bottom
        let mut k = 0; while k < self.debris.len() {
            if !self.debris[k].update(screen_ground_y, &self.wall_grid) || (self.debris[k].pos.y > screen_h + 50.0) {
                self.debris.remove(k);
                continue;
            }
            // (disabled) absorption into wall grid to prevent auto-repair
            k += 1;
        }
        
        // Occasionally collapse unsupported wall pixels as well
        if turbo::time::tick() % 20 == 0 {
            let fallen = self.wall_grid.pop_unsupported();
            for p in fallen.into_iter().take(48) {
                let sz = PIXEL_SIZE;
                let vel = V2::new((random::f32()-0.5)*1.0, -0.5 + (random::f32()-0.5)*0.6);
                self.debris.push(Debris::new(V2::new(p.x - sz*0.5, p.y - sz*0.5), vel, sz, sz, 0xff7a7a7a));
            }
        }
        
        // Update particles (global gravity inside Particle), remove if off-screen bottom
        let mut j = 0; while j < self.particles.len() {
            if !self.particles[j].update() || (self.particles[j].pos.y > screen_h + 50.0) { self.particles.remove(j); } else { j += 1; }
        }
        
        // Render
        self.render();
    }
    
    fn spawn_impact_particles(&mut self, pos: &V2, count: usize) {
        for _ in 0..(4 + count.min(8)) {
            let a = random::f32() * 6.28318; let s = 1.0 + random::f32() * 3.0;
            self.particles.push(Particle::new(pos.clone(), V2::new(a.cos() * s, a.sin() * s - 1.5)));
        }
    }
    
    fn render(&mut self) {
        clear(0x0a0a0aff);
        self.wall_grid.render();
        self.monster.render();
        for d in &self.debris { d.render(); }
        circ!(d = PLAYER_RADIUS * 2.0, position = (self.player.pos.x, self.player.pos.y), color = 0xff66ccff, fixed = true);
        for b in &self.bullets { circ!(d = BULLET_RADIUS * 2.0, position = (b.pos.x, b.pos.y), color = 0xffffff66, fixed = true); }
        for p in &self.particles { rect!(x = p.pos.x, y = p.pos.y, w = 1.0, h = 1.0, color = 0xffffaa55, fixed = true); }
        text!("Bullets: {}", self.bullets.len(); x = 10, y = 10, color = 0xffffffff);
        text!("Particles: {}", self.particles.len(); x = 10, y = 26, color = 0xffffffff);
        text!("Debris: {}", self.debris.len(); x = 10, y = 42, color = 0xffffffff);
    }
}