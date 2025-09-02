use crate::math::Vec2 as V2;
use crate::models::physics_body::PhysicsBody;
use crate::constants::*;

/// Handles all physics calculations and updates
#[turbo::serialize]
pub struct PhysicsSystem {
    gravity: V2,
    wind: V2,
    water_currents: Vec<WaterCurrent>,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            gravity: V2::new(0.0, GRAVITY),
            wind: V2::zero(),
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
            let water_resistance = V2::new(
                -body.velocity.x * 0.1,
                -body.velocity.y * 0.05
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
            body.position = V2::new(pos.x, 800.0);
            
            // Bounce with damping
            let vel = body.velocity;
            body.velocity = V2::new(vel.x * FRICTION, -vel.y * BOUNCE_DAMPING);
        }
        
        // Water surface collision (Y = 0)
        if body.position.y < 0.0 {
            let pos = body.position;
            body.position = V2::new(pos.x, 0.0);
            
            // Water splash effect
            let vel = body.velocity;
            body.velocity = V2::new(vel.x * 0.8, vel.y * 0.3);
        }
    }
    
    /// Set wind direction and strength
    pub fn set_wind(&mut self, direction: V2, strength: f32) {
        self.wind = direction.normalize().scale(strength);
    }
    
    /// Add water current
    pub fn add_water_current(&mut self, position: V2, direction: V2, strength: f32) {
        self.water_currents.push(WaterCurrent {
            position,
            direction: direction.normalize(),
            strength,
            radius: 100.0,
        });
    }
    
    /// Get water current at specific position
    pub fn get_water_current_at(&self, position: &V2) -> V2 {
        let mut total_current = V2::zero();
        
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
    position: V2,
    direction: V2,
    strength: f32,
    radius: f32,
}
