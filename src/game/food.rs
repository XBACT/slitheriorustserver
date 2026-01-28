

use crate::protocol::outgoing::FoodData;


#[derive(Debug, Clone, Copy)]
pub struct Food {
    pub x: u16,
    pub y: u16,
    pub size: u8,
    pub color: u8,
}

impl Food {
   
    pub fn new(x: u16, y: u16, size: u8, color: u8) -> Self {
        Self { x, y, size, color }
    }

   
    pub fn random(game_radius: u32, rng: &mut impl FnMut() -> f32) -> Self {
       
        let angle = rng() * std::f32::consts::PI * 2.0;
        let r = rng().sqrt() * (game_radius as f32 * 0.95);

        let x = (game_radius as f32 + r * angle.cos()) as u16;
        let y = (game_radius as f32 + r * angle.sin()) as u16;

        let size = (rng() * 10.0) as u8 + 5;
        let color = (rng() * 28.0) as u8;

        Self { x, y, size, color }
    }

   
    pub fn near(x: u16, y: u16, offset: f32, rng: &mut impl FnMut() -> f32) -> Self {
        let angle = rng() * std::f32::consts::PI * 2.0;
        let r = rng() * offset;

        let new_x = (x as f32 + r * angle.cos()) as u16;
        let new_y = (y as f32 + r * angle.sin()) as u16;

        let size = (rng() * 15.0) as u8 + 10;
        let color = (rng() * 28.0) as u8;

        Self {
            x: new_x,
            y: new_y,
            size,
            color,
        }
    }

   
    pub fn value(&self) -> u16 {
        self.size as u16 * 2
    }

   
    pub fn to_packet_data(&self) -> FoodData {
        FoodData {
            x: self.x,
            y: self.y,
            size: self.size,
            color: self.color,
        }
    }

   
    pub fn sector_coords(&self, sector_size: u16) -> (u8, u8) {
        let sx = (self.x / sector_size) as u8;
        let sy = (self.y / sector_size) as u8;
        (sx, sy)
    }
}

impl From<Food> for FoodData {
    fn from(food: Food) -> Self {
        food.to_packet_data()
    }
}


#[derive(Debug, Clone, Copy)]
pub struct FoodEaten {
    pub food: Food,
    pub snake_id: u16,
}


#[derive(Debug, Clone, Copy)]
pub struct FoodSpawned {
    pub food: Food,
   
    pub from_snake: bool,
}


#[derive(Debug, Clone, Default)]
pub struct FoodCollection {
    foods: Vec<Food>,
    max_capacity: usize,
}

impl FoodCollection {
    pub fn new(max_capacity: usize) -> Self {
        Self {
            foods: Vec::with_capacity(max_capacity),
            max_capacity,
        }
    }

    pub fn add(&mut self, food: Food) -> bool {
        if self.foods.len() < self.max_capacity {
            self.foods.push(food);
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Food> {
        if index < self.foods.len() {
            Some(self.foods.swap_remove(index))
        } else {
            None
        }
    }

    pub fn remove_at_position(&mut self, x: u16, y: u16, tolerance: u16) -> Option<Food> {
        let tolerance_sq = (tolerance as u32).pow(2);

        for i in 0..self.foods.len() {
            let food = &self.foods[i];
            let dx = (food.x as i32 - x as i32).abs() as u32;
            let dy = (food.y as i32 - y as i32).abs() as u32;

            if dx * dx + dy * dy <= tolerance_sq {
                return Some(self.foods.swap_remove(i));
            }
        }

        None
    }

    pub fn foods(&self) -> &[Food] {
        &self.foods
    }

    pub fn len(&self) -> usize {
        self.foods.len()
    }

    pub fn is_empty(&self) -> bool {
        self.foods.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.foods.len() >= self.max_capacity
    }

    pub fn clear(&mut self) {
        self.foods.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Food> {
        self.foods.iter()
    }

   
    pub fn find_in_radius(&self, x: u16, y: u16, radius: u16) -> Vec<(usize, &Food)> {
        let radius_sq = (radius as u32).pow(2);

        self.foods
            .iter()
            .enumerate()
            .filter(|(_, food)| {
                let dx = (food.x as i32 - x as i32).abs() as u32;
                let dy = (food.y as i32 - y as i32).abs() as u32;
                dx * dx + dy * dy <= radius_sq
            })
            .collect()
    }
}


pub mod colors {
   
    pub const COLOR_COUNT: u8 = 28;

   
    pub fn random_color(rng: &mut impl FnMut() -> f32) -> u8 {
        (rng() * COLOR_COUNT as f32) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_creation() {
        let food = Food::new(1000, 2000, 10, 5);
        assert_eq!(food.x, 1000);
        assert_eq!(food.y, 2000);
        assert_eq!(food.size, 10);
        assert_eq!(food.color, 5);
    }

    #[test]
    fn test_food_value() {
        let food = Food::new(0, 0, 10, 0);
        assert_eq!(food.value(), 20);
    }

    #[test]
    fn test_food_collection() {
        let mut collection = FoodCollection::new(10);

        for i in 0..10 {
            assert!(collection.add(Food::new(i * 100, i * 100, 5, 0)));
        }

        assert!(collection.is_full());
        assert!(!collection.add(Food::new(1000, 1000, 5, 0)));

        let removed = collection.remove(0);
        assert!(removed.is_some());
        assert!(!collection.is_full());
    }
}
