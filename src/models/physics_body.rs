use crate::math::Vec3;
use crate::constants::{GRAVITY, FRICTION, BOUNCE_DAMPING};
use turbo::*;

#[turbo::serialize]
pub struct PhysicsBody {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub radius: f32,
    pub destroyed: bool,
    pub health: f32,
    pub max_health: f32,
}

impl PhysicsBody {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec3::new(x, y, 0.0),
            velocity: Vec3::zero(),
            mass: 1.0,
            radius: 6.0,
            destroyed: false,
            health: 100.0,
            max_health: 100.0,
        }
    }
    
    pub fn update(&mut self) {
        if self.destroyed { return; }
        self.velocity.y += GRAVITY;
        self.velocity = self.velocity.mul(FRICTION);
        self.position = self.position.add(self.velocity.clone());

        let (w, h) = resolution();
        let (w, h) = (w as f32, h as f32);
        if self.position.x - self.radius < 0.0 {
            self.position.x = self.radius;
            self.velocity.x = -self.velocity.x * BOUNCE_DAMPING;
        } else if self.position.x + self.radius > w {
            self.position.x = w - self.radius;
            self.velocity.x = -self.velocity.x * BOUNCE_DAMPING;
        }
        if self.position.y - self.radius < 0.0 {
            self.position.y = self.radius;
            self.velocity.y = -self.velocity.y * BOUNCE_DAMPING;
        } else if self.position.y + self.radius > h {
            self.position.y = h - self.radius;
            self.velocity.y = -self.velocity.y * BOUNCE_DAMPING;
        }
    }

    pub fn take_damage(&mut self, damage: f32) {
        if self.destroyed { return; }
        self.health -= damage;
        if self.health <= 0.0 { self.destroyed = true; }
    }

    pub fn distance_to(&self, other: &PhysicsBody) -> f32 {
        let diff = self.position.sub(other.position.clone());
        diff.length()
    }

    pub fn check_collision(&self, other: &PhysicsBody) -> bool {
        if self.destroyed || other.destroyed { return false; }
        let distance = self.distance_to(other);
        distance < (self.radius + other.radius)
    }

    pub fn resolve_collision(&mut self, other: &mut PhysicsBody) {
        if self.destroyed || other.destroyed { return; }
        let diff = self.position.sub(other.position.clone());
        let distance = diff.length();
        if distance == 0.0 { return; }

        let normal = diff.mul(1.0 / distance);
        let overlap = (self.radius + other.radius) - distance;
        let separation = normal.mul(overlap * 0.5);
        self.position = self.position.add(separation.clone());
        other.position = other.position.sub(separation.clone());

        let relative_velocity = self.velocity.sub(other.velocity.clone());
        let velocity_along_normal = relative_velocity.x * normal.x + relative_velocity.y * normal.y;
        if velocity_along_normal > 0.0 { return; }

        let restitution = 0.5;
        let j = -(1.0 + restitution) * velocity_along_normal;
        let impulse = normal.mul(j);
        self.velocity = self.velocity.add(impulse.mul(1.0 / self.mass));
        other.velocity = other.velocity.sub(impulse.mul(1.0 / other.mass));

        let collision_force = velocity_along_normal.abs();
        if collision_force > 5.0 {
            let damage = collision_force * 2.0;
            self.take_damage(damage);
            other.take_damage(damage);
        }
    }
}
