use crate::math::Vec3 as V3;
use crate::models::physics_body::PhysicsBody;
use crate::constants::*;

/// Handles all physics calculations and updates
#[turbo::serialize]
pub struct PhysicsSystem {
    gravity: V3,
    wind: V3,
    water_currents: Vec<WaterCurrent>,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            gravity: V3::new(0.0, GRAVITY, 0.0),
            wind: V3::zero(),
            water_currents: Vec::new(),
        }
    }
    
    /// Update physics for all physics bodies
    pub fn update(&mut self, bodies: &mut [&mut PhysicsBody], delta_time: f32) {
        for body in bodies {
            self.apply_forces(body, delta_time);
            self.update_position(body, delta_time);
            self.handle_collisions(body);
        }
    }
    
    /// Apply forces (gravity, wind, water resistance)
    fn apply_forces(&self, body: &mut PhysicsBody, delta_time: f32) {
        let mut total_force = self.gravity.clone();
        
        // Add wind effect
        total_force = total_force.add(self.wind.clone());
        
        // Add water resistance if underwater
        if body.position.y > 0.0 {
            let water_resistance = V3::new(
                -body.velocity.x * 0.1,
                -body.velocity.y * 0.05,
                0.0
            );
            total_force = total_force.add(water_resistance);
        }
        
        // Apply force to velocity
        let new_vel = body.velocity.add(total_force.scale(delta_time));
        body.velocity = new_vel;
    }
    
    /// Update position based on velocity
    fn update_position(&self, body: &mut PhysicsBody, delta_time: f32) {
        let new_pos = body.position.add(body.velocity.scale(delta_time));
        body.position = new_pos;
    }
    
    /// Handle basic collision detection and response
    fn handle_collisions(&self, body: &mut PhysicsBody) {
        // Ground collision
        if body.position.y > 800.0 {
            let pos = body.position;
            body.position = V3::new(pos.x, 800.0, pos.z);
            
            // Bounce with damping
            let vel = body.velocity;
            body.velocity = V3::new(vel.x * FRICTION, -vel.y * BOUNCE_DAMPING, vel.z);
        }
        
        // Water surface collision (Y = 0)
        if body.position.y < 0.0 {
            let pos = body.position;
            body.position = V3::new(pos.x, 0.0, pos.z);
            
            // Water splash effect
            let vel = body.velocity;
            body.velocity = V3::new(vel.x * 0.8, vel.y * 0.3, vel.z);
        }
    }
    
    /// Set wind direction and strength
    pub fn set_wind(&mut self, direction: V3, strength: f32) {
        self.wind = direction.normalize().scale(strength);
    }
    
    /// Get current wind vector
    pub fn get_wind(&self) -> V3 {
        self.wind.clone()
    }
    
    /// Add water current
    pub fn add_water_current(&mut self, position: V3, direction: V3, strength: f32) {
        self.water_currents.push(WaterCurrent {
            position,
            direction: direction.normalize(),
            strength,
            radius: 100.0,
        });
    }
    
    /// Get water current at specific position
    pub fn get_water_current_at(&self, position: &V3) -> V3 {
        let mut total_current = V3::zero();
        
        for current in &self.water_currents {
            let distance = position.distance_to(&current.position);
            if distance < current.radius {
                let influence = 1.0 - (distance / current.radius);
                let current_force = current.direction.scale(current.strength * influence);
                total_current = total_current.add(current_force);
            }
        }
        
        total_current
    }
}

/// Represents a water current that affects physics
#[turbo::serialize]
struct WaterCurrent {
    position: V3,
    direction: V3,
    strength: f32,
    radius: f32,
}
