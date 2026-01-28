

use std::f32::consts::PI;


#[inline]
pub fn fast_inv_sqrt(x: f32) -> f32 {
    let i = x.to_bits();
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    y * (1.5 - 0.5 * x * y * y)
}


#[inline]
pub fn fast_sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    x * fast_inv_sqrt(x)
}


#[inline]
pub fn distance_squared(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}


#[inline]
pub fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    fast_sqrt(distance_squared(x1, y1, x2, y2))
}


#[inline]
pub fn point_in_circle(px: f32, py: f32, cx: f32, cy: f32, radius: f32) -> bool {
    distance_squared(px, py, cx, cy) <= radius * radius
}


#[inline]
pub fn circles_intersect(x1: f32, y1: f32, r1: f32, x2: f32, y2: f32, r2: f32) -> bool {
    let combined_radius = r1 + r2;
    distance_squared(x1, y1, x2, y2) <= combined_radius * combined_radius
}


#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    let two_pi = 2.0 * PI;
    ((angle % two_pi) + two_pi) % two_pi
}


#[inline]
pub fn angle_difference(from: f32, to: f32) -> f32 {
    let diff = normalize_angle(to - from);
    if diff > PI {
        diff - 2.0 * PI
    } else {
        diff
    }
}


#[inline]
pub fn lerp_angle(from: f32, to: f32, t: f32) -> f32 {
    let diff = angle_difference(from, to);
    normalize_angle(from + diff * t)
}


#[inline]
pub fn move_towards_angle(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = angle_difference(current, target);
    if diff.abs() <= max_delta {
        target
    } else {
        normalize_angle(current + max_delta * diff.signum())
    }
}


pub fn segments_intersect(
    x1: f32, y1: f32, x2: f32, y2: f32,
    x3: f32, y3: f32, x4: f32, y4: f32,
) -> bool {
    let d1 = direction(x3, y3, x4, y4, x1, y1);
    let d2 = direction(x3, y3, x4, y4, x2, y2);
    let d3 = direction(x1, y1, x2, y2, x3, y3);
    let d4 = direction(x1, y1, x2, y2, x4, y4);

    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }

    if d1 == 0.0 && on_segment(x3, y3, x4, y4, x1, y1) {
        return true;
    }
    if d2 == 0.0 && on_segment(x3, y3, x4, y4, x2, y2) {
        return true;
    }
    if d3 == 0.0 && on_segment(x1, y1, x2, y2, x3, y3) {
        return true;
    }
    if d4 == 0.0 && on_segment(x1, y1, x2, y2, x4, y4) {
        return true;
    }

    false
}


#[inline]
fn direction(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> f32 {
    (x3 - x1) * (y2 - y1) - (x2 - x1) * (y3 - y1)
}


#[inline]
fn on_segment(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> bool {
    x3 >= x1.min(x2) && x3 <= x1.max(x2) && y3 >= y1.min(y2) && y3 <= y1.max(y2)
}


pub fn segment_circle_intersect(
    x1: f32, y1: f32, x2: f32, y2: f32,
    cx: f32, cy: f32, radius: f32,
) -> bool {
   
    let dx = x2 - x1;
    let dy = y2 - y1;

   
    let fx = x1 - cx;
    let fy = y1 - cy;

    let a = dx * dx + dy * dy;
    let b = 2.0 * (fx * dx + fy * dy);
    let c = fx * fx + fy * fy - radius * radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return false;
    }

    let discriminant = fast_sqrt(discriminant);
    let t1 = (-b - discriminant) / (2.0 * a);
    let t2 = (-b + discriminant) / (2.0 * a);

    (t1 >= 0.0 && t1 <= 1.0) || (t2 >= 0.0 && t2 <= 1.0)
}


pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() as f32) / (u64::MAX as f32)
    }

    pub fn range(&mut self, min: u32, max: u32) -> u32 {
        let range = max - min;
        if range == 0 {
            return min;
        }
        min + (self.next_u64() % range as u64) as u32
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

impl Default for SimpleRng {
    fn default() -> Self {
        Self::new(0x853c49e6748fea9b)
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl BoundingBox {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Self { x, y, radius }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        circles_intersect(self.x, self.y, self.radius, other.x, other.y, other.radius)
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        point_in_circle(px, py, self.x, self.y, self.radius)
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x - self.width / 2.0
            && px <= self.x + self.width / 2.0
            && py >= self.y - self.height / 2.0
            && py <= self.y + self.height / 2.0
    }

    pub fn intersects_circle(&self, cx: f32, cy: f32, radius: f32) -> bool {
       
        let half_w = self.width / 2.0 + radius;
        let half_h = self.height / 2.0 + radius;
        let dx = (cx - self.x).abs();
        let dy = (cy - self.y).abs();
        dx <= half_w && dy <= half_h
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1920.0, 1080.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance() {
        assert!((distance(0.0, 0.0, 3.0, 4.0) - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_normalize_angle() {
        assert!((normalize_angle(3.0 * PI) - PI).abs() < 0.001);
        assert!((normalize_angle(-PI) - PI).abs() < 0.001);
    }

    #[test]
    fn test_angle_difference() {
        assert!((angle_difference(0.0, PI / 2.0) - PI / 2.0).abs() < 0.001);
        assert!((angle_difference(0.0, 3.0 * PI / 2.0) + PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_circles_intersect() {
        assert!(circles_intersect(0.0, 0.0, 5.0, 3.0, 0.0, 5.0));
        assert!(!circles_intersect(0.0, 0.0, 1.0, 10.0, 0.0, 1.0));
    }

    #[test]
    fn test_simple_rng() {
        let mut rng = SimpleRng::new(12345);
        let v1 = rng.next_f32();
        let v2 = rng.next_f32();
        assert!(v1 >= 0.0 && v1 <= 1.0);
        assert!(v2 >= 0.0 && v2 <= 1.0);
        assert_ne!(v1, v2);
    }
}
