use macroquad::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Clone, Eq, PartialEq)]
struct Node {
    position: (i32, i32),
    f_score: i32,
    g_score: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Grid {
    a: f32,
    m: f32,
    stretch: (f32, f32),
    grid_offset: f32,
    pub blocked_nodes: HashSet<(i32, i32)>,
}

// The original game uses a grid system to determine the position of the characters and
// blocked areas of the map.
// The grid is sheared and rotated to fit the isometric perspective of the game.
// The a, m and stretch values have been experimentally determined to match the grid of the original game.
impl Grid {
    pub fn new() -> Self {
        Self {
            a: 0.261,
            m: -1.744,
            stretch: (38.81, 10.32),
            grid_offset: 10.,
            blocked_nodes: HashSet::new(),
        }
    }

    pub fn get_grid_from_coord(&self, v: Vec2) -> (i32, i32) {
        let v = Vec2::new(v.x / 3.0, v.y / 3.0);
        let v = Vec2::new(v.x, v.y - self.grid_offset);
        let untransformed_x = v.x - self.m * v.y;
        let untransformed_y = v.y;
        let rotated_x = self.a.cos() * untransformed_x + untransformed_y * self.a.sin();
        let rotated_y = -self.a.sin() * untransformed_x + untransformed_y * self.a.cos();
        let x = (rotated_x / self.stretch.0).round() as i32;
        let y = (rotated_y / self.stretch.1).round() as i32;
        (x + 1, y + 17)
    }

    pub fn get_coord_from_grid(&self, x: i32, y: i32) -> Vec2 {
        let x = (x - 1) as f32 * self.stretch.0;
        let y = (y - 17) as f32 * self.stretch.1;
        let rotated_x = self.a.cos() * x - y * self.a.sin();
        let rotated_y = self.a.sin() * x + y * self.a.cos();
        let transformed_x = rotated_x + self.m * rotated_y;
        let transformed_y = rotated_y + self.grid_offset;
        Vec2::new(transformed_x * 3.0, transformed_y * 3.0)
    }

    pub fn update_blocked_nodes(&mut self, blocked_nodes: Vec<(i32, i32)>) {
        self.blocked_nodes = blocked_nodes.into_iter().collect();
    }

    pub fn is_node_walkable(&self, node: (i32, i32)) -> bool {
        let (x, y) = node;

        if self.blocked_nodes.contains(&node) {
            return false;
        }

        // Check boundary conditions
        if x - y >= 16 {
            return false; // Off the screen to the right
        }
        if y - x >= 16 {
            return false; // Off the screen on the left
        }
        if x + y > 64 {
            return false; // Off the screen on the bottom
        }
        if x + y <= 17 {
            return false; // Off the screen on the top
        }

        true
    }

    fn blocked_diagonals(&self, node: (i32, i32)) -> Vec<(i32, i32)> {
        let mut blocked_diagonals = vec![];
        for x in [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 0)].windows(2) {
            let u = (node.0 + x[0].0, node.1 + x[0].1);
            let v = (node.0 + x[1].0, node.1 + x[1].1);
            let diagonal = (node.0 + x[0].0 + x[1].0, node.1 + x[0].1 + x[1].1);
            if self.blocked_nodes.contains(&u) && self.blocked_nodes.contains(&v) {
                blocked_diagonals.push(diagonal);
            }
        }
        blocked_diagonals
    }

    pub fn pathfind(&self, start: (i32, i32), goal: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut f_score = HashMap::new();

        g_score.insert(start, 0);
        f_score.insert(start, self.heuristic(start, goal));
        open_set.push(Node {
            position: start,
            f_score: f_score[&start],
            g_score: 0,
        });

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                return Some(self.reconstruct_path(came_from, current.position));
            }

            for neighbor in self.get_neighbors(current.position) {
                let tentative_g_score = g_score[&current.position] + 1;

                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                    came_from.insert(neighbor, current.position);
                    g_score.insert(neighbor, tentative_g_score);
                    let f = tentative_g_score + self.heuristic(neighbor, goal);
                    f_score.insert(neighbor, f);
                    open_set.push(Node {
                        position: neighbor,
                        f_score: f,
                        g_score: tentative_g_score,
                    });
                }
            }
        }

        None
    }

    fn get_neighbors(&self, pos: (i32, i32)) -> Vec<(i32, i32)> {
        let directions = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        
        // If the top and right neighbors are blocked, the top-right diagonal
        // should be blocked. The same is true for the other diagonals
        let blocked_diagonals = self.blocked_diagonals(pos);
        directions
            .iter()
            .map(|&(dx, dy)| (pos.0 + dx, pos.1 + dy))
            .filter(|&pos| self.is_node_walkable(pos) && !blocked_diagonals.contains(&pos))
            .collect()
    }

    fn heuristic(&self, a: (i32, i32), b: (i32, i32)) -> i32 {
        let dx = (a.0 - b.0).abs();
        let dy = (a.1 - b.1).abs();
        (dx + dy) + (1414 - 1000) * dx.min(dy)
    }

    fn reconstruct_path(
        &self,
        came_from: HashMap<(i32, i32), (i32, i32)>,
        mut current: (i32, i32),
    ) -> Vec<(i32, i32)> {
        let mut path = vec![current];
        while let Some(&prev) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        }
        path.reverse();
        path
    }
}
