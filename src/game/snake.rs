

use crate::config::snake_consts;
use crate::game::food::Food;
use crate::game::math::{
    distance_squared, lerp_angle, move_towards_angle, normalize_angle, BoundingBox, Viewport,
};
use crate::protocol::types::{SnakeChanges, SnakeId};
use std::collections::VecDeque;
use std::f32::consts::PI;


#[derive(Debug, Clone, Copy)]
pub struct BodyPart {
    pub x: f32,
    pub y: f32,
}

impl BodyPart {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn as_u16(&self) -> (u16, u16) {
        (self.x as u16, self.y as u16)
    }
}


#[derive(Debug, Clone)]
pub struct Snake {
   
    pub id: SnakeId,
   
    pub skin: u8,
   
    pub changes: SnakeChanges,
   
    pub accelerating: bool,
   
    pub is_bot: bool,
   
    pub newly_spawned: bool,
   
    pub name: String,
   
    pub custom_skin: Option<String>,
   
    pub speed: f32,
   
    pub angle: f32,
   
    pub target_angle: f32,
   
    pub fullness: u32,
   
    pub bounding_box: BoundingBox,
   
    pub viewport: Viewport,
   
    pub body: VecDeque<BodyPart>,
   
    pub foods_eaten: Vec<Food>,
   
    pub foods_spawned: Vec<Food>,
   
    pub kills: u32,
   
    pub dying: bool,
   
    pub dead: bool,
   
    rot_time_accum: u64,
   
    ai_time_accum: u64,
   
    prev_head_x: f32,
    prev_head_y: f32,
}

impl Snake {
   
    pub fn new(id: SnakeId, x: f32, y: f32, name: String, skin: u8, start_length: usize) -> Self {
        let mut body = VecDeque::with_capacity(start_length.max(10));

       
        for i in 0..start_length {
            body.push_back(BodyPart::new(x, y - (i as f32 * snake_consts::TAIL_STEP_DISTANCE)));
        }

        let mut snake = Self {
            id,
            skin,
            changes: SnakeChanges::default(),
            accelerating: false,
            is_bot: false,
            newly_spawned: true,
            name,
            custom_skin: None,
            speed: snake_consts::BASE_MOVE_SPEED as f32,
            angle: PI / 2.0,
            target_angle: PI / 2.0,
            fullness: 0,
            bounding_box: BoundingBox::new(x, y, 50.0),
            viewport: Viewport::default(),
            body,
            foods_eaten: Vec::new(),
            foods_spawned: Vec::new(),
            kills: 0,
            dying: false,
            dead: false,
            rot_time_accum: 0,
            ai_time_accum: 0,
            prev_head_x: x,
            prev_head_y: y,
        };

        snake.update_bounding_box();
        snake
    }

   
    pub fn head(&self) -> Option<&BodyPart> {
        self.body.front()
    }

   
    pub fn head_pos(&self) -> (f32, f32) {
        self.body
            .front()
            .map(|p| (p.x, p.y))
            .unwrap_or((0.0, 0.0))
    }

   
    pub fn head_pos_u16(&self) -> (u16, u16) {
        self.body
            .front()
            .map(|p| (p.x as u16, p.y as u16))
            .unwrap_or((0, 0))
    }

   
    pub fn tail(&self) -> Option<&BodyPart> {
        self.body.back()
    }

   
    pub fn length(&self) -> usize {
        self.body.len()
    }

   
    pub fn scale(&self) -> f32 {
        let base = 1.0;
        let growth = (self.fullness as f32 / 10000.0).min(2.0);
        base + growth * 0.5
    }

   
    pub fn body_radius(&self) -> f32 {
        14.0 * self.scale()
    }

   
    pub fn score(&self) -> u32 {
       
        let parts = self.body.len() as f32;
        let fam = self.fullness as f32 / 16777215.0;
        ((15.0 * (parts - 1.0) + fam) / 3.0 - 8.0).floor().max(1.0) as u32
    }

   
    pub fn tick(&mut self, dt_ms: u64, game_radius: f32) {
        if self.dead {
            return;
        }

        self.changes.clear();
        self.foods_eaten.clear();

       
        self.rot_time_accum += dt_ms;
        while self.rot_time_accum >= snake_consts::ROT_STEP_INTERVAL_MS {
            self.rot_time_accum -= snake_consts::ROT_STEP_INTERVAL_MS;
            self.update_rotation();
        }

       
        self.update_speed(dt_ms);

       
        self.move_forward(dt_ms);

       
        let (hx, hy) = self.head_pos();
        let dist_from_center = (hx * hx + hy * hy).sqrt();
        if dist_from_center > game_radius * 0.98 {
            self.dying = true;
            self.changes.set_dying();
        }

       
        if self.accelerating && self.fullness > 0 {
            self.handle_boost_cost();
        }

       
        self.update_bounding_box();

       
        self.prev_head_x = hx;
        self.prev_head_y = hy;
    }

   
    pub fn tick_ai(&mut self, dt_ms: u64) {
        if !self.is_bot || self.dead {
            return;
        }

        self.ai_time_accum += dt_ms;
        if self.ai_time_accum >= snake_consts::AI_STEP_INTERVAL_MS {
            self.ai_time_accum = 0;

           
            let random = (self.id as f32 * 0.1 + self.fullness as f32 * 0.001) % 1.0;
            if random < 0.1 {
               
                self.target_angle += (random - 0.05) * PI;
                self.target_angle = normalize_angle(self.target_angle);
            }
        }
    }

   
    fn update_rotation(&mut self) {
        let prev_angle = self.angle;
        self.angle = move_towards_angle(
            self.angle,
            self.target_angle,
            snake_consts::ANGULAR_SPEED * 0.001,
        );

        if (self.angle - prev_angle).abs() > 0.001 {
            self.changes.set_angle();
        }
    }

   
    fn update_speed(&mut self, dt_ms: u64) {
        let target_speed = if self.accelerating {
            snake_consts::BOOST_SPEED as f32
        } else {
            snake_consts::BASE_MOVE_SPEED as f32
        };

        let speed_diff = target_speed - self.speed;
        let change = speed_diff.signum()
            * (snake_consts::SPEED_ACCELERATION as f32 * dt_ms as f32 / 1000.0).min(speed_diff.abs());

        if change.abs() > 0.1 {
            self.speed += change;
            self.changes.set_speed();
        }
    }

   
    fn move_forward(&mut self, dt_ms: u64) {
        if self.body.is_empty() {
            return;
        }

        let move_dist = self.speed * dt_ms as f32 / 1000.0;
        let dx = move_dist * self.angle.cos();
        let dy = move_dist * self.angle.sin();

       
        let head = self.body.front_mut().unwrap();
        head.x += dx;
        head.y += dy;

       
        for i in 1..self.body.len() {
            let (prev_x, prev_y) = {
                let prev = &self.body[i - 1];
                (prev.x, prev.y)
            };

            let curr = &mut self.body[i];
            let dist_sq = distance_squared(curr.x, curr.y, prev_x, prev_y);
            let target_dist = snake_consts::TAIL_STEP_DISTANCE;

            if dist_sq > target_dist * target_dist {
                let dist = dist_sq.sqrt();
                let ratio = (dist - target_dist * snake_consts::TAIL_K) / dist;
                curr.x += (prev_x - curr.x) * ratio;
                curr.y += (prev_y - curr.y) * ratio;
            }
        }

        self.changes.set_pos();
    }

   
    fn handle_boost_cost(&mut self) {
       
        if self.fullness >= snake_consts::BOOST_COST as u32 {
            self.fullness -= snake_consts::BOOST_COST as u32;
            self.changes.set_fullness();
        }
    }

   
    pub fn set_target_angle(&mut self, angle: f32) {
        let new_angle = normalize_angle(angle);
        if (new_angle - self.target_angle).abs() > 0.001 {
            self.target_angle = new_angle;
            self.changes.set_wangle();
        }
    }

   
    pub fn set_accelerating(&mut self, accelerating: bool) {
        self.accelerating = accelerating;
    }

   
    pub fn eat_food(&mut self, food: Food) {
        let value = food.value() as u32;
        self.fullness += value;
        self.foods_eaten.push(food);
        self.changes.set_fullness();

       
        self.try_grow();
    }

   
    fn try_grow(&mut self) {
       
        let target_parts = (self.fullness / 100).min(500) as usize + 10;
        if self.body.len() < target_parts {
            if let Some(tail) = self.body.back() {
                let new_part = BodyPart::new(tail.x, tail.y);
                self.body.push_back(new_part);
            }
        }
    }

   
    pub fn kill(&mut self, rng: &mut impl FnMut() -> f32) {
        self.dying = true;
        self.dead = true;
        self.changes.set_dead();

       
        for part in &self.body {
            let food = Food::near(part.x as u16, part.y as u16, 20.0, rng);
            self.foods_spawned.push(food);
        }
    }

   
    fn update_bounding_box(&mut self) {
        if self.body.is_empty() {
            return;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for part in &self.body {
            min_x = min_x.min(part.x);
            max_x = max_x.max(part.x);
            min_y = min_y.min(part.y);
            max_y = max_y.max(part.y);
        }

        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;
        let radius = ((max_x - min_x).max(max_y - min_y) / 2.0) + self.body_radius();

        self.bounding_box = BoundingBox::new(center_x, center_y, radius);
    }

   
    pub fn update_viewport(&mut self) {
        let (hx, hy) = self.head_pos();
        self.viewport.x = hx;
        self.viewport.y = hy;
    }

   
    pub fn head_delta(&self) -> (i16, i16) {
        let (hx, hy) = self.head_pos();
        let dx = (hx - self.prev_head_x) as i16;
        let dy = (hy - self.prev_head_y) as i16;
        (dx, dy)
    }

   
    pub fn body_as_u16(&self) -> Vec<(u16, u16)> {
        self.body.iter().map(|p| (p.x as u16, p.y as u16)).collect()
    }

   
    pub fn collides_with(&self, other: &Snake) -> bool {
        if self.id == other.id {
            return false;
        }

       
        if !self.bounding_box.intersects(&other.bounding_box) {
            return false;
        }

       
        let (head_x, head_y) = self.head_pos();
        let head_radius = self.body_radius();

        for (i, part) in other.body.iter().enumerate() {
           
            if i < snake_consts::PARTS_SKIP_COUNT {
                continue;
            }

            let part_radius = other.body_radius();
            let combined_radius = head_radius + part_radius;

            if distance_squared(head_x, head_y, part.x, part.y)
                <= combined_radius * combined_radius
            {
                return true;
            }
        }

        false
    }
}


pub const BOT_NAMES: &[&str] = &[
    "Bumba",
    "nick26",
    "jjjjj",
    "Rigor",
    "meow",
    "snake",
    "pro",
    "ROOMBA",
    "Player",
    "Slither",
    "Python",
    "Anaconda",
    "Viper",
    "Cobra",
    "Noodle",
    "Wiggles",
    "Snek",
];


pub fn random_bot_name(rng: &mut impl FnMut() -> f32) -> String {
    let idx = (rng() * BOT_NAMES.len() as f32) as usize;
    BOT_NAMES[idx % BOT_NAMES.len()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_creation() {
        let snake = Snake::new(1, 1000.0, 1000.0, "Test".to_string(), 0, 10);
        assert_eq!(snake.id, 1);
        assert_eq!(snake.length(), 10);
        assert!(snake.head().is_some());
    }

    #[test]
    fn test_snake_movement() {
        let mut snake = Snake::new(1, 1000.0, 1000.0, "Test".to_string(), 0, 10);
        snake.angle = 0.0;
        snake.target_angle = 0.0;

        let (initial_x, _) = snake.head_pos();
        snake.tick(100, 21600.0);
        let (new_x, _) = snake.head_pos();

        assert!(new_x > initial_x);
    }

    #[test]
    fn test_snake_eat_food() {
        let mut snake = Snake::new(1, 1000.0, 1000.0, "Test".to_string(), 0, 10);
        let initial_fullness = snake.fullness;

        let food = Food::new(1000, 1000, 10, 0);
        snake.eat_food(food);

        assert!(snake.fullness > initial_fullness);
    }
}
