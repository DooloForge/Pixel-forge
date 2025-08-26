use turbo::*;
use crate::math::Vec2 as V2;
use crate::constants::PIXEL_SIZE;

#[turbo::serialize]
pub struct MonsterGrid {
    pub origin: V2,
    pub cols: usize,
    pub rows: usize,
    pub color: u32,
    pub cells: Vec<bool>,
    pub vx: f32,
    pub vy: f32,
    pub grounded: bool,
}

impl MonsterGrid {
    pub fn new(x: f32, y: f32, cols: usize, rows: usize, color: u32) -> Self {
        let mut cells = vec![false; cols * rows];
        // Cute monster silhouette using simple shapes
        let cw = cols as f32; let ch = rows as f32;
        let cx = cw * 0.5; let cy = ch * 0.45;
        let body_rx = cw * 0.22; let body_ry = ch * 0.28; // body ellipse radii
        let head_cy = ch * 0.18; let head_r = ch * 0.12; // head circle
        let eye_off = cw * 0.06; let eye_r = ch * 0.025; // eyes as cutouts
        for r in 0..rows {
            for c in 0..cols {
                let mut alive = false;
                let x = c as f32; let y = r as f32;
                // body ellipse
                let nx = (x - cx) / body_rx; let ny = (y - cy) / body_ry;
                if nx*nx + ny*ny <= 1.0 { alive = true; }
                // head circle
                let dxh = (x - cx); let dyh = (y - head_cy);
                if dxh*dxh + dyh*dyh <= head_r*head_r { alive = true; }
                // horns triangles
                if y < head_cy - head_r * 0.6 {
                    let t = (head_cy - head_r * 0.6) - y; // height above
                    if (x > cx - head_r*0.9 - t && x < cx - head_r*0.2 + t) ||
                       (x > cx + head_r*0.2 - t && x < cx + head_r*0.9 + t) { alive = true; }
                }
                // arms bands
                if y > cy - body_ry*0.2 && y < cy + body_ry*0.05 && (x < cx - body_rx*0.8 || x > cx + body_rx*0.8) {
                    alive = true;
                }
                // legs rectangles
                if y > cy + body_ry*0.6 && y < ch {
                    if x > cx - body_rx*0.4 && x < cx - body_rx*0.15 { alive = true; }
                    if x < cx + body_rx*0.4 && x > cx + body_rx*0.15 { alive = true; }
                }
                // eyes cutout
                let ex1 = cx - eye_off; let ex2 = cx + eye_off; let ey = head_cy;
                let e1 = (x - ex1)*(x - ex1) + (y - ey)*(y - ey) <= eye_r*eye_r;
                let e2 = (x - ex2)*(x - ex2) + (y - ey)*(y - ey) <= eye_r*eye_r;
                if e1 || e2 { alive = false; }
                cells[r*cols + c] = alive;
            }
        }
        Self { origin: V2::new(x, y), cols, rows, color, cells, vx: 1.5, vy: 0.0, grounded: false }
    }

    pub fn hit_circle(&mut self, x: f32, y: f32, radius: f32) -> usize {
        let mut destroyed = 0;
        let min_cx = ((x - self.origin.x - radius) / PIXEL_SIZE).floor() as isize;
        let max_cx = ((x - self.origin.x + radius) / PIXEL_SIZE).ceil() as isize;
        let min_cy = ((y - self.origin.y - radius) / PIXEL_SIZE).floor() as isize;
        let max_cy = ((y - self.origin.y + radius) / PIXEL_SIZE).ceil() as isize;
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if cx < 0 || cy < 0 { continue; }
                let (cxu, cyu) = (cx as usize, cy as usize);
                if cxu >= self.cols || cyu >= self.rows { continue; }
                let i = cyu * self.cols + cxu;
                if self.cells[i] {
                    let px = self.origin.x + cxu as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    let py = self.origin.y + cyu as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    let dx = px - x; let dy = py - y;
                    if (dx*dx + dy*dy).sqrt() <= radius {
                        self.cells[i] = false;
                        destroyed += 1;
                    }
                }
            }
        }
        destroyed
    }

    pub fn update(&mut self, ground_y: f32, min_x: f32, max_x: f32, gravity: f32) {
        // gravity
        self.vy += gravity;
        self.origin.y += self.vy;
        // collide with ground
        let feet_y = self.origin.y + self.rows as f32 * PIXEL_SIZE;
        if feet_y > ground_y {
            let penetration = feet_y - ground_y;
            self.origin.y -= penetration;
            self.vy = 0.0;
            self.grounded = true;
        } else {
            self.grounded = false;
        }
        // horizontal walk
        self.origin.x += self.vx;
        let max_origin_x = max_x - self.cols as f32 * PIXEL_SIZE;
        if self.origin.x < min_x { self.origin.x = min_x; self.vx = -self.vx; }
        if self.origin.x > max_origin_x { self.origin.x = max_origin_x; self.vx = -self.vx; }
    }

    // Compute center of mass of alive pixels in world coords
    pub fn center_of_mass(&self) -> Option<V2> {
        let mut sumx = 0.0; let mut sumy = 0.0; let mut n = 0.0;
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.cells[r*self.cols + c] {
                    sumx += self.origin.x + c as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    sumy += self.origin.y + r as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    n += 1.0;
                }
            }
        }
        if n > 0.0 { Some(V2::new(sumx / n, sumy / n)) } else { None }
    }

    // Remove unsupported pixels (not connected to bottom row) and return their world positions
    pub fn pop_unsupported(&mut self) -> Vec<V2> {
        let total = self.cols * self.rows;
        let mut supported = vec![false; total];
        let mut stack: Vec<(usize, usize)> = Vec::new();
        // seed from bottom row alive pixels
        let br = self.rows - 1;
        for c in 0..self.cols { if self.cells[br*self.cols + c] { stack.push((br, c)); supported[br*self.cols + c] = true; } }
        // 4-neighbor flood fill upward
        while let Some((r,c)) = stack.pop() {
            let neigh = [(r.wrapping_sub(1), c), (r+1, c), (r, c.wrapping_sub(1)), (r, c+1)];
            for (nr, nc) in neigh.into_iter() {
                if nr < self.rows && nc < self.cols {
                    let i = nr*self.cols + nc;
                    if self.cells[i] && !supported[i] { supported[i] = true; stack.push((nr,nc)); }
                }
            }
        }
        // collect unsupported
        let mut removed = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                let idx = r*self.cols + c;
                if self.cells[idx] && !supported[idx] {
                    self.cells[idx] = false;
                    let x = self.origin.x + c as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    let y = self.origin.y + r as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    removed.push(V2::new(x, y));
                }
            }
        }
        removed
    }

    pub fn render(&self) {
        for r in 0..self.rows {
            for c in 0..self.cols {
                let i = r * self.cols + c;
                if self.cells[i] {
                    let x = self.origin.x + c as f32 * PIXEL_SIZE;
                    let y = self.origin.y + r as f32 * PIXEL_SIZE;
                    rect!(x = x, y = y, w = PIXEL_SIZE, h = PIXEL_SIZE, color = self.color, fixed = true);
                }
            }
        }
    }
}
