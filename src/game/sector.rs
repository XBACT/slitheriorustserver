

use crate::game::food::{Food, FoodCollection};
use crate::game::math::BoundingBox;
use crate::protocol::types::SnakeId;
use std::collections::HashSet;


#[derive(Debug)]
pub struct Sector {
   
    pub x: u8,
   
    pub y: u8,
   
    pub food: FoodCollection,
   
    pub snakes: HashSet<SnakeId>,
   
    pub bounds: BoundingBox,
}

impl Sector {
   
    pub fn new(x: u8, y: u8, sector_size: u16, max_food: usize) -> Self {
        let world_x = x as f32 * sector_size as f32 + sector_size as f32 / 2.0;
        let world_y = y as f32 * sector_size as f32 + sector_size as f32 / 2.0;
        let radius = (sector_size as f32 * 1.42) / 2.0;

        Self {
            x,
            y,
            food: FoodCollection::new(max_food),
            snakes: HashSet::new(),
            bounds: BoundingBox::new(world_x, world_y, radius),
        }
    }

   
    pub fn add_snake(&mut self, id: SnakeId) {
        self.snakes.insert(id);
    }

   
    pub fn remove_snake(&mut self, id: SnakeId) {
        self.snakes.remove(&id);
    }

   
    pub fn has_snake(&self, id: SnakeId) -> bool {
        self.snakes.contains(&id)
    }

   
    pub fn add_food(&mut self, food: Food) -> bool {
        self.food.add(food)
    }

   
    pub fn center(&self, sector_size: u16) -> (f32, f32) {
        let x = self.x as f32 * sector_size as f32 + sector_size as f32 / 2.0;
        let y = self.y as f32 * sector_size as f32 + sector_size as f32 / 2.0;
        (x, y)
    }

   
    pub fn is_empty(&self) -> bool {
        self.snakes.is_empty() && self.food.is_empty()
    }
}


#[derive(Debug)]
pub struct SectorGrid {
   
    sectors: Vec<Sector>,
   
    pub size: u8,
   
    pub sector_size: u16,
   
    max_food_per_sector: usize,
}

impl SectorGrid {
   
    pub fn new(sector_count: u8, sector_size: u16, max_food_per_sector: usize) -> Self {
        let total = sector_count as usize * sector_count as usize;
        let mut sectors = Vec::with_capacity(total);

        for y in 0..sector_count {
            for x in 0..sector_count {
                sectors.push(Sector::new(x, y, sector_size, max_food_per_sector));
            }
        }

        Self {
            sectors,
            size: sector_count,
            sector_size,
            max_food_per_sector,
        }
    }

   
    fn index(&self, x: u8, y: u8) -> usize {
        y as usize * self.size as usize + x as usize
    }

   
    pub fn get(&self, x: u8, y: u8) -> Option<&Sector> {
        if x < self.size && y < self.size {
            Some(&self.sectors[self.index(x, y)])
        } else {
            None
        }
    }

   
    pub fn get_mut(&mut self, x: u8, y: u8) -> Option<&mut Sector> {
        if x < self.size && y < self.size {
            let idx = self.index(x, y);
            Some(&mut self.sectors[idx])
        } else {
            None
        }
    }

   
    pub fn world_to_sector(&self, world_x: f32, world_y: f32) -> (u8, u8) {
        let x = ((world_x / self.sector_size as f32).floor() as i32)
            .clamp(0, self.size as i32 - 1) as u8;
        let y = ((world_y / self.sector_size as f32).floor() as i32)
            .clamp(0, self.size as i32 - 1) as u8;
        (x, y)
    }

   
    pub fn sectors_in_viewport(&self, viewport_x: f32, viewport_y: f32, radius: f32) -> Vec<(u8, u8)> {
        let min_x = ((viewport_x - radius) / self.sector_size as f32).floor() as i32;
        let max_x = ((viewport_x + radius) / self.sector_size as f32).ceil() as i32;
        let min_y = ((viewport_y - radius) / self.sector_size as f32).floor() as i32;
        let max_y = ((viewport_y + radius) / self.sector_size as f32).ceil() as i32;

        let mut result = Vec::new();

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if x >= 0 && x < self.size as i32 && y >= 0 && y < self.size as i32 {
                    result.push((x as u8, y as u8));
                }
            }
        }

        result
    }

   
    pub fn add_snake(&mut self, id: SnakeId, world_x: f32, world_y: f32) {
        let (sx, sy) = self.world_to_sector(world_x, world_y);
        if let Some(sector) = self.get_mut(sx, sy) {
            sector.add_snake(id);
        }
    }

   
    pub fn remove_snake(&mut self, id: SnakeId, world_x: f32, world_y: f32) {
        let (sx, sy) = self.world_to_sector(world_x, world_y);
        if let Some(sector) = self.get_mut(sx, sy) {
            sector.remove_snake(id);
        }
    }

   
    pub fn update_snake_sector(
        &mut self,
        id: SnakeId,
        old_x: f32,
        old_y: f32,
        new_x: f32,
        new_y: f32,
    ) -> Option<(u8, u8)> {
        let (old_sx, old_sy) = self.world_to_sector(old_x, old_y);
        let (new_sx, new_sy) = self.world_to_sector(new_x, new_y);

        if old_sx != new_sx || old_sy != new_sy {
            self.remove_snake(id, old_x, old_y);
            self.add_snake(id, new_x, new_y);
            Some((new_sx, new_sy))
        } else {
            None
        }
    }

   
    pub fn add_food(&mut self, food: Food) -> bool {
        let (sx, sy) = self.world_to_sector(food.x as f32, food.y as f32);
        if let Some(sector) = self.get_mut(sx, sy) {
            sector.add_food(food)
        } else {
            false
        }
    }

   
    pub fn remove_food(&mut self, x: u16, y: u16) -> Option<Food> {
        let (sx, sy) = self.world_to_sector(x as f32, y as f32);
        if let Some(sector) = self.get_mut(sx, sy) {
            sector.food.remove_at_position(x, y, 10)
        } else {
            None
        }
    }

   
    pub fn snakes_near(&self, x: f32, y: f32, radius: f32) -> HashSet<SnakeId> {
        let mut result = HashSet::new();

        for (sx, sy) in self.sectors_in_viewport(x, y, radius) {
            if let Some(sector) = self.get(sx, sy) {
                result.extend(&sector.snakes);
            }
        }

        result
    }

   
    pub fn food_near(&self, x: f32, y: f32, radius: f32) -> Vec<&Food> {
        let mut result = Vec::new();

        for (sx, sy) in self.sectors_in_viewport(x, y, radius) {
            if let Some(sector) = self.get(sx, sy) {
                result.extend(sector.food.iter());
            }
        }

        result
    }

   
    pub fn iter(&self) -> impl Iterator<Item = &Sector> {
        self.sectors.iter()
    }

   
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Sector> {
        self.sectors.iter_mut()
    }

   
    pub fn total_food(&self) -> usize {
        self.sectors.iter().map(|s| s.food.len()).sum()
    }
}


#[derive(Debug, Clone)]
pub enum SectorEvent {
   
    Entered { x: u8, y: u8 },
   
    Left { x: u8, y: u8 },
}


#[derive(Debug, Default)]
pub struct SectorTracker {
   
    visible: HashSet<(u8, u8)>,
}

impl SectorTracker {
    pub fn new() -> Self {
        Self::default()
    }

   
    pub fn update(
        &mut self,
        grid: &SectorGrid,
        viewport_x: f32,
        viewport_y: f32,
        view_radius: f32,
    ) -> Vec<SectorEvent> {
        let new_visible: HashSet<_> = grid
            .sectors_in_viewport(viewport_x, viewport_y, view_radius)
            .into_iter()
            .collect();

        let mut events = Vec::new();

       
        for &(x, y) in new_visible.difference(&self.visible) {
            events.push(SectorEvent::Entered { x, y });
        }

       
        for &(x, y) in self.visible.difference(&new_visible) {
            events.push(SectorEvent::Left { x, y });
        }

        self.visible = new_visible;
        events
    }

   
    pub fn visible_sectors(&self) -> &HashSet<(u8, u8)> {
        &self.visible
    }

   
    pub fn is_visible(&self, x: u8, y: u8) -> bool {
        self.visible.contains(&(x, y))
    }

   
    pub fn clear(&mut self) {
        self.visible.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_grid_creation() {
        let grid = SectorGrid::new(90, 480, 100);
        assert_eq!(grid.size, 90);
        assert_eq!(grid.sectors.len(), 90 * 90);
    }

    #[test]
    fn test_world_to_sector() {
        let grid = SectorGrid::new(90, 480, 100);

        let (sx, sy) = grid.world_to_sector(0.0, 0.0);
        assert_eq!((sx, sy), (0, 0));

        let (sx, sy) = grid.world_to_sector(480.0, 480.0);
        assert_eq!((sx, sy), (1, 1));

        let (sx, sy) = grid.world_to_sector(21600.0, 21600.0);
        assert_eq!((sx, sy), (44, 44));
    }

    #[test]
    fn test_add_remove_snake() {
        let mut grid = SectorGrid::new(90, 480, 100);

        grid.add_snake(1, 500.0, 500.0);
        let (sx, sy) = grid.world_to_sector(500.0, 500.0);
        assert!(grid.get(sx, sy).unwrap().has_snake(1));

        grid.remove_snake(1, 500.0, 500.0);
        assert!(!grid.get(sx, sy).unwrap().has_snake(1));
    }

    #[test]
    fn test_sector_tracker() {
        let grid = SectorGrid::new(90, 480, 100);
        let mut tracker = SectorTracker::new();

        let events = tracker.update(&grid, 1000.0, 1000.0, 500.0);
        assert!(!events.is_empty());

       
        let events = tracker.update(&grid, 1000.0, 1000.0, 500.0);
        assert!(events.is_empty());
    }
}
