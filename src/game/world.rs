

use crate::config::GameConfig;
use crate::game::food::Food;
use crate::game::math::SimpleRng;
use crate::game::sector::SectorGrid;
use crate::game::snake::{random_bot_name, Snake};
use crate::protocol::types::SnakeId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;


pub struct World {
   
    pub config: GameConfig,
   
    snakes: HashMap<SnakeId, Snake>,
   
    pub sectors: SectorGrid,
   
    next_snake_id: SnakeId,
   
    pub tick_count: u64,
   
    pub frame_count: u32,
   
    rng: SimpleRng,
   
    changed_snakes: Vec<SnakeId>,
   
    dead_snakes: Vec<SnakeId>,
   
    new_food: Vec<Food>,
   
    eaten_food: Vec<(SnakeId, Food)>,
}

impl World {
   
    pub fn new(config: GameConfig) -> Self {
        let sector_count = config.sector_count_along_edge as u8;
        let sectors = SectorGrid::new(sector_count, config.sector_size, 100);

        Self {
            config,
            snakes: HashMap::new(),
            sectors,
            next_snake_id: 1,
            tick_count: 0,
            frame_count: 0,
            rng: SimpleRng::new(12345),
            changed_snakes: Vec::new(),
            dead_snakes: Vec::new(),
            new_food: Vec::new(),
            eaten_food: Vec::new(),
        }
    }

   
    pub fn init(&mut self) {
        self.spawn_initial_food();

       
        for _ in 0..self.config.initial_bots {
            self.spawn_bot();
        }
    }

   
    fn spawn_initial_food(&mut self) {
        let game_radius = self.config.game_radius as f32;
        let target_food = self.config.sector_count_along_edge as usize * 50;

        for _ in 0..target_food {
            let food = Food::random(self.config.game_radius, &mut || self.rng.next_f32());
            self.sectors.add_food(food);
        }
    }

   
    pub fn create_snake(&mut self, name: String, skin: u8) -> SnakeId {
        let id = self.next_snake_id;
        self.next_snake_id += 1;

       
        let (x, y) = self.find_safe_spawn();

        let start_length = self.config.human_snake_start_score as usize + 5;
        let mut snake = Snake::new(id, x, y, name, skin, start_length);

       
        self.sectors.add_snake(id, x, y);

        self.snakes.insert(id, snake);
        self.changed_snakes.push(id);

        id
    }

   
    pub fn spawn_bot(&mut self) -> SnakeId {
        let id = self.next_snake_id;
        self.next_snake_id += 1;

        let (x, y) = self.find_safe_spawn();
        let name = random_bot_name(&mut || self.rng.next_f32());
        let skin = (self.rng.next_f32() * 9.0) as u8;

        let start_length = self.config.bot_snake_start_score as usize + 5;
        let mut snake = Snake::new(id, x, y, name, skin, start_length);
        snake.is_bot = true;

        self.sectors.add_snake(id, x, y);
        self.snakes.insert(id, snake);
        self.changed_snakes.push(id);

        id
    }

   
    fn find_safe_spawn(&mut self) -> (f32, f32) {
        let game_radius = self.config.game_radius as f32;
        let spawn_radius = game_radius * 0.8;

        for _ in 0..100 {
            let angle = self.rng.next_f32() * std::f32::consts::PI * 2.0;
            let r = self.rng.next_f32().sqrt() * spawn_radius;

            let x = game_radius + r * angle.cos();
            let y = game_radius + r * angle.sin();

            if self.is_location_safe(x, y, 100.0) {
                return (x, y);
            }
        }

       
        (game_radius, game_radius)
    }

   
    fn is_location_safe(&self, x: f32, y: f32, radius: f32) -> bool {
        let nearby_snakes = self.sectors.snakes_near(x, y, radius * 2.0);

        for &snake_id in &nearby_snakes {
            if let Some(snake) = self.snakes.get(&snake_id) {
                let (hx, hy) = snake.head_pos();
                let dist_sq = (x - hx).powi(2) + (y - hy).powi(2);
                if dist_sq < radius * radius {
                    return false;
                }
            }
        }

        true
    }

   
    pub fn remove_snake(&mut self, id: SnakeId) {
        if let Some(snake) = self.snakes.remove(&id) {
            let (hx, hy) = snake.head_pos();
            self.sectors.remove_snake(id, hx, hy);
        }
    }

   
    pub fn get_snake(&self, id: SnakeId) -> Option<&Snake> {
        self.snakes.get(&id)
    }

   
    pub fn get_snake_mut(&mut self, id: SnakeId) -> Option<&mut Snake> {
        self.snakes.get_mut(&id)
    }

   
    pub fn snakes(&self) -> &HashMap<SnakeId, Snake> {
        &self.snakes
    }

   
    pub fn snake_count(&self) -> usize {
        self.snakes.len()
    }

   
    pub fn tick(&mut self, dt_ms: u64) {
        self.tick_count += 1;
        self.frame_count = self.frame_count.wrapping_add(1);

        self.changed_snakes.clear();
        self.dead_snakes.clear();
        self.new_food.clear();
        self.eaten_food.clear();

        let game_radius = self.config.game_radius as f32;

       
        let snake_ids: Vec<_> = self.snakes.keys().copied().collect();
        for id in snake_ids {
            if let Some(snake) = self.snakes.get_mut(&id) {
                let (old_x, old_y) = snake.head_pos();

               
                snake.tick(dt_ms, game_radius);

               
                if snake.is_bot {
                    snake.tick_ai(dt_ms);
                }

                let (new_x, new_y) = snake.head_pos();

               
                if let Some(_) = self.sectors.update_snake_sector(id, old_x, old_y, new_x, new_y) {
                   
                }

               
                if snake.changes.0 != 0 {
                    self.changed_snakes.push(id);
                }

               
                if snake.dead {
                    self.dead_snakes.push(id);
                }
            }
        }

       
        self.check_collisions();

       
        self.process_eating();

       
        self.spawn_food();

       
        self.process_dead_snakes();

       
        if self.config.bot_respawn {
            self.respawn_bots();
        }
    }

   
    fn check_collisions(&mut self) {
        let snake_ids: Vec<_> = self.snakes.keys().copied().collect();

        for i in 0..snake_ids.len() {
            let id1 = snake_ids[i];

            for j in (i + 1)..snake_ids.len() {
                let id2 = snake_ids[j];

               
                let collides_1_with_2;
                let collides_2_with_1;

                {
                    let snake1 = self.snakes.get(&id1).unwrap();
                    let snake2 = self.snakes.get(&id2).unwrap();

                    if snake1.dead || snake2.dead {
                        continue;
                    }

                    collides_1_with_2 = snake1.collides_with(snake2);
                    collides_2_with_1 = snake2.collides_with(snake1);
                }

               
                if collides_1_with_2 {
                    if let Some(snake) = self.snakes.get_mut(&id1) {
                        snake.kill(&mut || self.rng.next_f32());
                        self.dead_snakes.push(id1);
                    }

                   
                    if let Some(killer) = self.snakes.get_mut(&id2) {
                        killer.kills += 1;
                    }
                }

                if collides_2_with_1 {
                    if let Some(snake) = self.snakes.get_mut(&id2) {
                        snake.kill(&mut || self.rng.next_f32());
                        self.dead_snakes.push(id2);
                    }

                   
                    if let Some(killer) = self.snakes.get_mut(&id1) {
                        killer.kills += 1;
                    }
                }
            }
        }
    }

   
    fn process_eating(&mut self) {
        let snake_ids: Vec<_> = self.snakes.keys().copied().collect();

        for id in snake_ids {
            if let Some(snake) = self.snakes.get(&id) {
                if snake.dead {
                    continue;
                }

                let (hx, hy) = snake.head_pos();
                let eat_radius = snake.body_radius() + 10.0;

               
                let foods_to_eat: Vec<_> = self
                    .sectors
                    .food_near(hx, hy, eat_radius)
                    .iter()
                    .filter(|f| {
                        let dist_sq = ((f.x as f32 - hx).powi(2) + (f.y as f32 - hy).powi(2));
                        dist_sq < eat_radius * eat_radius
                    })
                    .map(|f| **f)
                    .collect();

               
                for food in foods_to_eat {
                    if let Some(removed) = self.sectors.remove_food(food.x, food.y) {
                        if let Some(snake) = self.snakes.get_mut(&id) {
                            snake.eat_food(removed);
                            self.eaten_food.push((id, removed));
                        }
                    }
                }
            }
        }
    }

   
    fn spawn_food(&mut self) {
        let current_food = self.sectors.total_food();
        let target_food = self.config.sector_count_along_edge as usize * 50;

        if current_food < target_food {
            let spawn_count = self.config.food_spawn_rate.min((target_food - current_food) as u16);

            for _ in 0..spawn_count {
                let food = Food::random(self.config.game_radius, &mut || self.rng.next_f32());
                if self.sectors.add_food(food) {
                    self.new_food.push(food);
                }
            }
        }
    }

   
    fn process_dead_snakes(&mut self) {
        let dead_ids: Vec<_> = self.dead_snakes.drain(..).collect();

        for id in dead_ids {
            if let Some(snake) = self.snakes.get(&id) {
               
                for food in &snake.foods_spawned {
                    if self.sectors.add_food(*food) {
                        self.new_food.push(*food);
                    }
                }
            }
        }
    }

   
    fn respawn_bots(&mut self) {
        let bot_count = self.snakes.values().filter(|s| s.is_bot && !s.dead).count();
        let target = self.config.initial_bots as usize;

        if bot_count < target {
            self.spawn_bot();
        }
    }

   
    pub fn changed_snakes(&self) -> &[SnakeId] {
        &self.changed_snakes
    }

   
    pub fn new_food(&self) -> &[Food] {
        &self.new_food
    }

   
    pub fn eaten_food(&self) -> &[(SnakeId, Food)] {
        &self.eaten_food
    }

   
    pub fn leaderboard(&self, count: usize) -> Vec<(&Snake, u32)> {
        let mut snakes: Vec<_> = self
            .snakes
            .values()
            .filter(|s| !s.dead)
            .map(|s| (s, s.score()))
            .collect();

        snakes.sort_by(|a, b| b.1.cmp(&a.1));
        snakes.truncate(count);
        snakes
    }

   
    pub fn player_rank(&self, id: SnakeId) -> Option<usize> {
        let mut snakes: Vec<_> = self
            .snakes
            .values()
            .filter(|s| !s.dead)
            .map(|s| (s.id, s.score()))
            .collect();

        snakes.sort_by(|a, b| b.1.cmp(&a.1));

        snakes.iter().position(|(sid, _)| *sid == id).map(|p| p + 1)
    }

   
    pub fn minimap_data(&self, grid_size: u16) -> Vec<u8> {
        let game_diameter = self.config.game_radius * 2;
        let cell_size = game_diameter / grid_size as u32;

        let mut data = vec![0u8; (grid_size * grid_size / 8) as usize + 1];

        for snake in self.snakes.values() {
            if snake.dead {
                continue;
            }

            let (hx, hy) = snake.head_pos();
            let grid_x = (hx as u32 / cell_size).min(grid_size as u32 - 1) as u16;
            let grid_y = (hy as u32 / cell_size).min(grid_size as u32 - 1) as u16;

            let bit_index = (grid_y * grid_size + grid_x) as usize;
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;

            if byte_index < data.len() {
                data[byte_index] |= 1 << bit_offset;
            }
        }

        data
    }
}


pub type SharedWorld = Arc<RwLock<World>>;


pub fn create_shared_world(config: GameConfig) -> SharedWorld {
    let mut world = World::new(config);
    world.init();
    Arc::new(RwLock::new(world))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let config = GameConfig::default();
        let world = World::new(config);
        assert_eq!(world.snake_count(), 0);
    }

    #[test]
    fn test_snake_creation() {
        let config = GameConfig::default();
        let mut world = World::new(config);

        let id = world.create_snake("Test".to_string(), 0);
        assert!(world.get_snake(id).is_some());
        assert_eq!(world.snake_count(), 1);
    }

    #[test]
    fn test_world_tick() {
        let config = GameConfig::default();
        let mut world = World::new(config);
        world.init();

        world.create_snake("Test".to_string(), 0);
        world.tick(8);

        assert!(world.tick_count > 0);
    }
}
