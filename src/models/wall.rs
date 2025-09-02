use turbo::*;   
use crate::math::Vec2 as V2;
use crate::constants::PIXEL_SIZE;

#[turbo::serialize]
pub struct Wall {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Wall {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self { Self { x, y, w, h } }
    pub fn contains(&self, p: &V2) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }
}

#[turbo::serialize]
pub struct Pixel {
    pub alive: bool,
}

#[turbo::serialize]
pub struct WallGrid {
    pub origin: V2,
    pub cols: usize,
    pub rows: usize,
    pub color: u32,
    pub cells: Vec<Pixel>,
}

impl WallGrid {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: u32) -> Self {
        let cols = (width / PIXEL_SIZE).floor().max(1.0) as usize;
        let rows = (height / PIXEL_SIZE).floor().max(1.0) as usize;
        let mut cells = Vec::with_capacity(cols * rows);
        for _ in 0..(cols * rows) { cells.push(Pixel { alive: true }); }
        Self { origin: V2::new(x, y), cols, rows, color, cells }
    }
    pub fn index(&self, cx: isize, cy: isize) -> Option<usize> {
        if cx < 0 || cy < 0 { return None; }
        let (cx, cy) = (cx as usize, cy as usize);
        if cx >= self.cols || cy >= self.rows { return None; }
        Some(cy * self.cols + cx)
    }
    pub fn cell_pos(&self, cx: usize, cy: usize) -> (f32, f32) {
        (self.origin.x + cx as f32 * PIXEL_SIZE, self.origin.y + cy as f32 * PIXEL_SIZE)
    }
    pub fn destroy_circle_to_debris(&mut self, x: f32, y: f32, radius: f32) -> Vec<V2> {
        let mut out = Vec::new();
        let min_cx = ((x - self.origin.x - radius) / PIXEL_SIZE).floor() as isize;
        let max_cx = ((x - self.origin.x + radius) / PIXEL_SIZE).ceil() as isize;
        let min_cy = ((y - self.origin.y - radius) / PIXEL_SIZE).floor() as isize;
        let max_cy = ((y - self.origin.y + radius) / PIXEL_SIZE).ceil() as isize;
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if let Some(i) = self.index(cx, cy) {
                    if self.cells[i].alive {
                        let (px, py) = self.cell_pos(cx as usize, cy as usize);
                        let cxm = px + PIXEL_SIZE * 0.5;
                        let cym = py + PIXEL_SIZE * 0.5;
                        let dx = cxm - x; let dy = cym - y;
                        if (dx*dx + dy*dy).sqrt() <= radius {
                            self.cells[i].alive = false;
                            out.push(V2::new(cxm, cym));
                        }
                    }
                }
            }
        }
        out
    }
    pub fn hit_circle(&mut self, x: f32, y: f32, radius: f32) -> usize {
        self.destroy_circle_to_debris(x, y, radius).len()
    }
    pub fn render(&self) {
        for cy in 0..self.rows {
            for cx in 0..self.cols {
                let i = cy * self.cols + cx;
                if self.cells[i].alive {
                    let (x, y) = self.cell_pos(cx, cy);
                    rect!(x = x, y = y, w = PIXEL_SIZE, h = PIXEL_SIZE, color = self.color, fixed = true);
                }
            }
        }
    }
    // Query if a world point is inside any alive cell
    pub fn solid_at(&self, wx: f32, wy: f32) -> bool {
        let cx = ((wx - self.origin.x) / PIXEL_SIZE).floor() as isize;
        let cy = ((wy - self.origin.y) / PIXEL_SIZE).floor() as isize;
        if let Some(i) = self.index(cx, cy) { self.cells[i].alive } else { false }
    }
    // Check if a rect overlaps any alive cell (simple corner sampling and coarse iteration)
    pub fn rect_overlaps(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let min_cx = ((x - self.origin.x) / PIXEL_SIZE).floor() as isize;
        let max_cx = (((x + w) - self.origin.x) / PIXEL_SIZE).ceil() as isize;
        let min_cy = ((y - self.origin.y) / PIXEL_SIZE).floor() as isize;
        let max_cy = (((y + h) - self.origin.y) / PIXEL_SIZE).ceil() as isize;
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if let Some(i) = self.index(cx, cy) {
                    if self.cells[i].alive { return true; }
                }
            }
        }
        false
    }
    // Remove unsupported pixels (not connected to bottom row)
    pub fn pop_unsupported(&mut self) -> Vec<V2> {
        let total = self.cols * self.rows;
        let mut supported = vec![false; total];
        let mut stack: Vec<(usize, usize)> = Vec::new();
        // seed from bottom row
        let br = self.rows - 1;
        for c in 0..self.cols { if self.cells[br*self.cols + c].alive { stack.push((br, c)); supported[br*self.cols + c] = true; } }
        // flood fill
        while let Some((r,c)) = stack.pop() {
            let candidates = [
                (r.wrapping_sub(1), c), (r+1, c), (r, c.wrapping_sub(1)), (r, c+1)
            ];
            for (nr,nc) in candidates.into_iter() {
                if nr < self.rows && nc < self.cols {
                    let idx = nr*self.cols + nc;
                    if self.cells[idx].alive && !supported[idx] { supported[idx] = true; stack.push((nr,nc)); }
                }
            }
        }
        // collect unsupported
        let mut removed = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                let idx = r*self.cols + c;
                if self.cells[idx].alive && !supported[idx] {
                    self.cells[idx].alive = false;
                    let x = self.origin.x + c as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    let y = self.origin.y + r as f32 * PIXEL_SIZE + PIXEL_SIZE*0.5;
                    removed.push(V2::new(x, y));
                }
            }
        }
        removed
    }

    // Try to absorb a debris block into the grid if empty cells fit
    pub fn absorb_debris(&mut self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let min_cx = ((x - self.origin.x) / PIXEL_SIZE).floor() as isize;
        let max_cx = (((x + w) - self.origin.x) / PIXEL_SIZE).ceil() as isize - 1;
        let min_cy = ((y - self.origin.y) / PIXEL_SIZE).floor() as isize;
        let max_cy = (((y + h) - self.origin.y) / PIXEL_SIZE).ceil() as isize - 1;
        // check empty
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if let Some(i) = self.index(cx, cy) { if self.cells[i].alive { return false; } } else { return false; }
            }
        }
        // fill
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if let Some(i) = self.index(cx, cy) { self.cells[i].alive = true; }
            }
        }
        true
    }
}
