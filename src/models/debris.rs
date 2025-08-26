use turbo::*;
use crate::math::Vec2 as V2;
use crate::constants::{GRAVITY, PIXEL_SIZE};
use crate::models::WallGrid;

#[turbo::serialize]
pub struct Debris {
    pub pos: V2,
    pub vel: V2,
    pub w: f32,
    pub h: f32,
    pub color: u32,
    pub grounded: bool,
    pub settled_frames: u32,
}

impl Debris {
    pub fn new(pos: V2, vel: V2, w: f32, h: f32, color: u32) -> Self {
        Self { pos, vel, w, h, color, grounded: false, settled_frames: 0 }
    }
    pub fn update(&mut self, ground_y: f32, wall: &WallGrid) -> bool {
        // gravity/fall
        if !self.grounded { self.vel.y += GRAVITY; }
        self.vel.x *= 0.98;
        // tentative move
        let next = self.pos.add(self.vel.clone());
        // collide with wall grid cells by sampling footprint
        if wall.rect_overlaps(next.x, next.y, self.w, self.h) {
            // try separate axis moves
            let try_x = V2::new(next.x, self.pos.y);
            if !wall.rect_overlaps(try_x.x, try_x.y, self.w, self.h) {
                self.pos.x = try_x.x;
            }
            let try_y = V2::new(self.pos.x, next.y);
            if !wall.rect_overlaps(try_y.x, try_y.y, self.w, self.h) {
                self.pos.y = try_y.y;
            } else {
                // blocked: attempt diagonal slide into cavities when falling
                if self.vel.y > 0.0 {
                    let slide = PIXEL_SIZE * 0.5;
                    let down = PIXEL_SIZE * 0.5;
                    let left_down = (self.pos.x - slide, self.pos.y + down);
                    let right_down = (self.pos.x + slide, self.pos.y + down);
                    if !wall.rect_overlaps(left_down.0, left_down.1, self.w, self.h) {
                        self.pos.x = left_down.0; self.pos.y = left_down.1;
                    } else if !wall.rect_overlaps(right_down.0, right_down.1, self.w, self.h) {
                        self.pos.x = right_down.0; self.pos.y = right_down.1;
                    }
                }
                // vertical block => stick on top if still blocked
                if wall.rect_overlaps(self.pos.x, self.pos.y + 0.01, self.w, self.h) || self.vel.y > 0.0 {
                    if self.vel.y > 0.0 {
                        let cy = ((self.pos.y + self.h - wall.origin.y) / PIXEL_SIZE).floor() as i32;
                        let top = wall.origin.y + cy as f32 * PIXEL_SIZE;
                        self.pos.y = top - self.h;
                        self.vel.y = 0.0;
                        self.grounded = true;
                    } else {
                        self.vel.y = 0.0;
                    }
                }
            }
        } else {
            self.pos = next;
            self.grounded = false;
        }
        // floor collision
        if self.pos.y + self.h > ground_y { self.pos.y = ground_y - self.h; self.vel.y = 0.0; self.grounded = true; }
        // track settling
        if self.grounded && self.vel.x.abs() < 0.05 && self.vel.y.abs() < 0.05 { self.settled_frames = self.settled_frames.saturating_add(1); } else { self.settled_frames = 0; }
        true
    }
    pub fn can_absorb(&self) -> bool { self.grounded && self.settled_frames > 10 }
    pub fn render(&self) {
        rect!(x = self.pos.x, y = self.pos.y, w = self.w, h = self.h, color = self.color, fixed = true);
    }
}
