use macroquad::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    SouthWest = 1,
    West = 2,
    NorthWest = 3,
    North = 4,
    NorthEast = 5,
    East = 6,
    SouthEast = 7,
    South = 8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterData {
    pub name: String,
    pub speed: f32,
    pub run_speed: f32,
}

pub struct Character {
    pub data: CharacterData,
    pub position: Vec2,
    pub direction: Direction,
    pub animation_index: usize,
    animation_timer: f32,
    pub animation_speed: f32,
    pub path: Option<Vec<(i32, i32)>>,
    pub target: Option<(i32, i32)>,
}

impl Character {
    pub fn new(data: CharacterData, position: Vec2) -> Self {
        Character {
            data,
            position,
            direction: Direction::South,
            animation_index: 0,
            animation_timer: 0.0,
            animation_speed: 0.1,
            path: None,
            target: None,
        }
    }

    pub fn update(&mut self, movement: Vec2, delta_time: f32) {
        if movement.length() > 0.0 {
            let speed = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                self.data.run_speed
            } else {
                self.data.speed
            };
            self.position += movement.normalize() * speed * delta_time;
            self.direction = self.vec_to_direction(movement);

            self.animation_timer += delta_time;
            if self.animation_timer >= self.animation_speed {
                self.animation_timer -= self.animation_speed;
                self.animation_index = (self.animation_index + 1) % 8;
            }
        } else {
            self.animation_index = 0;
            self.animation_timer = 0.0;
        }
    }

    fn vec_to_direction(&self, vec: Vec2) -> Direction {
        if vec.x == 0.0 && vec.y == 0.0 {
            return self.direction;
        }
        let angle = vec.y.atan2(vec.x);
        let angle_deg = angle.to_degrees();
        let adjusted_angle = (angle_deg + 360.0) % 360.0;
        match adjusted_angle as u32 {
            338..=360 | 0..=22 => Direction::East,
            23..=67 => Direction::SouthEast,
            68..=112 => Direction::South,
            113..=157 => Direction::SouthWest,
            158..=202 => Direction::West,
            203..=247 => Direction::NorthWest,
            248..=292 => Direction::North,
            293..=337 => Direction::NorthEast,
            _ => unreachable!(),
        }
    }

    pub fn set_animation_speed(&mut self, speed: f32) {
        self.animation_speed = speed;
    }

    pub fn set_path(&mut self, path: Option<Vec<(i32, i32)>>, target: Option<(i32, i32)>) {
        self.path = path;
        self.target = target;
    }
}
