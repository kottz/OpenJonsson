use macroquad::prelude::*;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum CursorType {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "move")]
    Move,
    #[serde(rename = "take")]
    Take,
    #[serde(rename = "talk")]
    Talk,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct ClickableArea {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct SpawnPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SceneTransition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(rename = "targetScene")]
    pub target_scene: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockedNodeDataCollection {
    blocked_node_data: Vec<BlockedNodeData>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BlockedNodeData {
    level_id: u32,
    scene_id: u32,
    blocked_nodes: Vec<(i32, i32)>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Scene {
    pub id: u32,
    pub description: String,
    pub background: String,
    #[serde(rename = "sceneTransitions")]
    pub scene_transitions: Vec<SceneTransition>,
    pub overlay_assets: Vec<OverlayAsset>,
    pub items: Vec<ItemInstance>,
    #[serde(skip)]
    pub blocked_nodes: Vec<(i32, i32)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterData {
    pub name: String,
    pub speed: f32,
    pub run_speed: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OverlayAsset {
    pub texture_path: String,
    pub x: f32,
    pub y: f32,
    pub z_value: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Level {
    pub id: u32,
    pub name: String,
    pub scenes: Vec<Scene>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ItemTextures {
    pub in_world: String,
    pub mouse_over: String,
    pub in_inventory: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub textures: ItemTextures,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ItemInstance {
    pub item_id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Cursor {
    pub cursor_type: CursorType,
    pub texture: String,
    pub hotspot: [i32; 2],
}

#[derive(Deserialize, Debug, Clone)]
pub struct MenuItem {
    pub name: String,
    pub texture: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Deserialize, Debug, Clone)]
pub struct UI {
    pub cursors: Vec<Cursor>,
    #[serde(rename = "menuItems")]
    pub menu_items: Vec<MenuItem>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GameData {
    pub levels: Vec<Level>,
    pub characters: Vec<CharacterData>,
    pub ui: UI,
    pub items: Vec<Item>,
    #[serde(skip_deserializing)]
    pub blocked_nodes: Vec<BlockedNodeData>,
}

struct Characters {
    data: Vec<CharacterData>,
    positions: Vec<Vec2>,
    directions: Vec<Direction>,
    animation_indices: Vec<usize>,
    animation_timers: Vec<f32>,
    animation_speeds: Vec<f32>,
    paths: Vec<Option<Vec<(i32, i32)>>>,
    targets: Vec<Option<(i32, i32)>>,
    count: usize,
}

struct Scenes {
    data: Vec<Scene>,
    count: usize,
}

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

struct Grid {
    a: f32,
    m: f32,
    stretch: (f32, f32),
    grid_offset: f32,
    blocked_nodes: HashSet<(i32, i32)>,
}

impl Grid {
    fn new() -> Self {
        Self {
            a: 0.261,
            m: -1.744,
            stretch: (38.81, 10.32),
            grid_offset: 10.,
            blocked_nodes: HashSet::new(),
        }
    }

    fn get_grid_from_coord(&self, v: Vec2) -> (i32, i32) {
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

    fn get_coord_from_grid(&self, x: i32, y: i32) -> Vec2 {
        let x = (x - 1) as f32 * self.stretch.0;
        let y = (y - 17) as f32 * self.stretch.1;
        let rotated_x = self.a.cos() * x - y * self.a.sin();
        let rotated_y = self.a.sin() * x + y * self.a.cos();
        let transformed_x = rotated_x + self.m * rotated_y;
        let transformed_y = rotated_y + self.grid_offset;
        Vec2::new(transformed_x * 3.0, transformed_y * 3.0)
    }

    fn update_blocked_nodes(&mut self, blocked_nodes: Vec<(i32, i32)>) {
        self.blocked_nodes = blocked_nodes.into_iter().collect();
    }

    fn is_node_walkable(&self, node: (i32, i32)) -> bool {
        let (x, y) = node;

        // Check if the node is not blocked
        if self.blocked_nodes.contains(&node) {
            return false;
        }

        // Check boundary conditions
        if x - y >= 16 {
            return false;
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
}

struct Game {
    characters: Characters,
    levels: Vec<Level>,
    scenes: Scenes,
    textures: HashMap<String, Texture2D>,
    character_textures: HashMap<String, Texture2D>,
    cursor_textures: HashMap<CursorType, Texture2D>,
    menu_item_textures: HashMap<String, Texture2D>,
    current_level: u32,
    current_scene: u32,
    window_size: Vec2,
    game_rect: Rect,
    active_character: Option<usize>,
    grid: Grid,
    current_cursor: CursorType,
    ui: UI,
    loading_textures: HashSet<String>,
    debug_tools: DebugTools,
    debug_instant_move: bool,
    items: Vec<Item>,
    inventory: Vec<u32>,
    world_items: Vec<Vec<ItemInstance>>,
}

struct DebugTools {
    bounding_box_mode: bool,
    bounding_box_start: Option<Vec2>,
    current_bounding_box: Option<Rect>,
    active: bool,
    draw_grid: bool,
}

impl DebugTools {
    fn new() -> Self {
        DebugTools {
            bounding_box_mode: false,
            bounding_box_start: None,
            current_bounding_box: None,
            active: false,
            draw_grid: false,
        }
    }

    fn handle_bounding_box_creation(&mut self, game_pos: Vec2) {
        if let Some(start) = self.bounding_box_start {
            let width = game_pos.x - start.x;
            let height = game_pos.y - start.y;
            let (x, y) = if width < 0.0 || height < 0.0 {
                (game_pos.x.min(start.x), game_pos.y.min(start.y))
            } else {
                (start.x, start.y)
            };
            let rect = Rect::new(x, y, width.abs(), height.abs());
            self.current_bounding_box = Some(rect);
            self.bounding_box_start = None;

            println!(
                "Bounding Box: x: {}, y: {}, width: {}, height: {}",
                x,
                y,
                width.abs(),
                height.abs()
            );
        } else {
            self.bounding_box_start = Some(game_pos);
            self.current_bounding_box = None;
        }
    }
}

impl Game {
    async fn new() -> Result<Self, String> {
        let json = load_string("static/level_data.json").await.unwrap();
        let mut game_data: GameData =
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let blocked_nodes_json = load_string("static/blocked_nodes.json").await.unwrap();
        let blocked_nodes: BlockedNodeDataCollection = serde_json::from_str(&blocked_nodes_json)
            .map_err(|e| format!("Failed to parse blocked nodes JSON: {}", e))?;

        game_data.blocked_nodes = blocked_nodes.blocked_node_data;

        for level in &mut game_data.levels {
            for scene in &mut level.scenes {
                let blocked_node_data = game_data
                    .blocked_nodes
                    .iter()
                    .find(|b| b.level_id == level.id && b.scene_id == scene.id)
                    .map(|b| b.blocked_nodes.clone())
                    .unwrap_or_default();
                scene.blocked_nodes = blocked_node_data;
            }
        }

        let mut characters = Characters {
            data: Vec::new(),
            positions: Vec::new(),
            directions: Vec::new(),
            animation_indices: Vec::new(),
            animation_timers: Vec::new(),
            animation_speeds: Vec::new(),
            paths: Vec::new(),
            targets: Vec::new(),
            count: 0,
        };

        for (i, character_data) in game_data.characters.into_iter().enumerate() {
            characters.data.push(character_data);
            characters
                .positions
                .push(Vec2::new(1000.0 + i as f32 * 100.0, 800.0));
            characters.directions.push(Direction::South);
            characters.animation_indices.push(0);
            characters.animation_timers.push(0.0);
            characters.animation_speeds.push(0.1);
            characters.paths.push(None);
            characters.targets.push(None);
            characters.count += 1;
        }

        let scenes = Scenes {
            data: Vec::new(),
            count: 0,
        };

        let window_size = Vec2::new(screen_width(), screen_height());
        let game_rect = Game::calculate_game_rect(window_size);

        let mut game = Game {
            characters,
            levels: game_data.levels,
            scenes,
            textures: HashMap::new(),
            character_textures: HashMap::new(),
            cursor_textures: HashMap::new(),
            menu_item_textures: HashMap::new(),
            current_level: 0,
            current_scene: 0,
            window_size,
            game_rect,
            active_character: Some(0),
            grid: Grid::new(),
            current_cursor: CursorType::Normal,
            ui: game_data.ui,
            loading_textures: HashSet::new(),
            debug_tools: DebugTools::new(),
            debug_instant_move: false,
            items: game_data.items,
            inventory: Vec::new(),
            world_items: Vec::new(),
        };

        game.load_level_scenes(game.current_level);
        game.load_current_and_adjacent_scenes().await;
        game.load_characters().await;
        game.load_debug_textures().await;
        game.load_ui_textures().await;
        game.load_item_textures().await;

        Ok(game)
    }

    fn calculate_game_rect(window_size: Vec2) -> Rect {
        let window_aspect = window_size.x / window_size.y;
        let game_aspect = 1920.0 / 1440.0;

        if window_aspect > game_aspect {
            let height = window_size.y;
            let width = height * game_aspect;
            let x = (window_size.x - width) / 2.0;
            Rect::new(x, 0.0, width, height)
        } else {
            let width = window_size.x;
            let height = width / game_aspect;
            let y = (window_size.y - height) / 2.0;
            Rect::new(0.0, y, width, height)
        }
    }

    async fn load_current_and_adjacent_scenes(&mut self) {
        let mut textures_to_load = Vec::new();
        if let Some(current_scene) = self.get_current_scene() {
            textures_to_load.push(current_scene.background.clone());
            for overlay_asset in &current_scene.overlay_assets {
                textures_to_load.push(overlay_asset.texture_path.clone());
            }
            for transition in &current_scene.scene_transitions {
                if let Some(target_scene) = self.get_scene(transition.target_scene) {
                    textures_to_load.push(target_scene.background.clone());
                }
            }
        }
        self.load_textures(textures_to_load).await;
    }

    async fn load_textures(&mut self, textures: Vec<String>) {
        for texture_path in textures {
            if let Err(e) = self.load_texture(&texture_path).await {
                eprintln!("{}", e);
            }
        }
    }

    async fn load_texture(&mut self, texture_path: &str) -> Result<(), String> {
        if self.textures.contains_key(texture_path) || self.loading_textures.contains(texture_path)
        {
            return Ok(());
        }

        self.loading_textures.insert(texture_path.to_string());
        let full_path = format!("static/resources/{}", texture_path);
        match load_texture(&full_path).await {
            Ok(texture) => {
                self.textures.insert(texture_path.to_string(), texture);
                self.loading_textures.remove(texture_path);
                Ok(())
            }
            Err(e) => {
                self.loading_textures.remove(texture_path);
                Err(format!("Failed to load texture {}: {}", texture_path, e))
            }
        }
    }

    async fn load_characters(&mut self) {
        for index in 0..self.characters.count {
            let character_data = &self.characters.data[index];
            for dir in 1..=8 {
                for frame in 0..=7 {
                    for state in [0, 7] {
                        let filename =
                            format!("{}{}{}{}.png", character_data.name, dir, frame, state);
                        if let Ok(texture) =
                            load_texture(&format!("static/resources/berlin/Gubbar/{}", filename))
                                .await
                        {
                            self.character_textures.insert(filename, texture);
                        }
                    }
                }
            }
        }
    }

    async fn load_debug_textures(&mut self) {
        if let Ok(texture) = load_texture("static/resources/berlin/Internal/-13.png").await {
            self.textures.insert("debug_grid".to_string(), texture);
        }
    }

    async fn load_ui_textures(&mut self) {
        for cursor in &self.ui.cursors {
            if let Ok(texture) = load_texture(&format!("static/resources/{}", cursor.texture)).await
            {
                self.cursor_textures.insert(cursor.cursor_type, texture);
            }
        }

        for menu_item in &self.ui.menu_items {
            if let Ok(texture) =
                load_texture(&format!("static/resources/{}", menu_item.texture)).await
            {
                self.menu_item_textures
                    .insert(menu_item.name.clone(), texture);
            }
        }
    }

    async fn load_item_textures(&mut self) {
        let mut textures_to_load = Vec::new();

        for item in &self.items {
            textures_to_load.push(item.textures.in_world.clone());
            textures_to_load.push(item.textures.mouse_over.clone());
            textures_to_load.push(item.textures.in_inventory.clone());
        }

        self.load_textures(textures_to_load).await;
    }

    fn load_level_scenes(&mut self, level_id: u32) {
        if let Some(level) = self.levels.iter().find(|l| l.id == level_id) {
            self.scenes = Scenes {
                data: level.scenes.clone(),
                count: level.scenes.len(),
            };
            self.world_items = level.scenes.iter().map(|s| s.items.clone()).collect();
            self.current_scene = 0; // Reset to the first scene of the new level

            // Update blocked nodes in the grid
            if let Some(current_scene) = self.get_current_scene() {
                self.grid
                    .update_blocked_nodes(current_scene.blocked_nodes.clone());
            }
        }
    }

    fn get_current_scene(&self) -> Option<&Scene> {
        self.scenes.data.get(self.current_scene as usize)
    }

    fn get_scene(&self, scene_id: u32) -> Option<&Scene> {
        self.scenes.data.iter().find(|s| s.id == scene_id)
    }

    fn get_scale(&self) -> f32 {
        self.game_rect.w / 1920.0
    }

    fn get_scaled_pos(&self, x: f32, y: f32) -> (f32, f32) {
        let scale = self.get_scale();
        (self.game_rect.x + x * scale, self.game_rect.y + y * scale)
    }

    fn get_game_coordinates(&self, mouse_pos: Vec2) -> Vec2 {
        let scale = self.get_scale();
        Vec2::new(
            (mouse_pos.x - self.game_rect.x) / scale,
            (mouse_pos.y - self.game_rect.y) / scale,
        )
    }

    fn set_cursor(&mut self, cursor_type: CursorType) {
        if self.cursor_textures.contains_key(&cursor_type) {
            self.current_cursor = cursor_type;
        }
    }

    fn determine_cursor(&self, game_pos: Vec2) -> CursorType {
        // Check for items first
        let current_scene_items = &self.world_items[self.current_scene as usize];
        for item in current_scene_items {
            if game_pos.x >= item.x
                && game_pos.x <= item.x + item.width
                && game_pos.y >= item.y
                && game_pos.y <= item.y + item.height
            {
                return CursorType::Take;
            }
        }

        // Then check for clickable areas
        if let Some(current_scene) = self.get_current_scene() {
            for st in &current_scene.scene_transitions {
                if game_pos.x >= st.x
                    && game_pos.x <= st.x + st.width
                    && game_pos.y >= st.y
                    && game_pos.y <= st.y + st.height
                {
                    return CursorType::Move;
                }
            }
        }

        // Default to normal cursor
        CursorType::Normal
    }

    fn vec_to_direction(vec: Vec2) -> Direction {
        if vec.x == 0.0 && vec.y == 0.0 {
            return Direction::South;
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

    fn pathfind(&self, start: (i32, i32), goal: (i32, i32)) -> Option<Vec<(i32, i32)>> {
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
                if !self.grid.is_node_walkable(neighbor) {
                    continue; // Skip non-walkable nodes
                }

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

        directions
            .iter()
            .map(|&(dx, dy)| (pos.0 + dx, pos.1 + dy))
            .filter(|&pos| self.grid.is_node_walkable(pos))
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

    fn update_window_size(&mut self) {
        let new_window_size = Vec2::new(screen_width(), screen_height());
        if new_window_size != self.window_size {
            self.window_size = new_window_size;
            self.game_rect = Self::calculate_game_rect(self.window_size);
        }
    }

    fn handle_item_click(&mut self, game_pos: Vec2) {
        let current_scene_items = &mut self.world_items[self.current_scene as usize];
        if let Some(index) = current_scene_items.iter().position(|item| {
            game_pos.x >= item.x
                && game_pos.x <= item.x + item.width
                && game_pos.y >= item.y
                && game_pos.y <= item.y + item.height
        }) {
            let item = current_scene_items.remove(index);
            self.inventory.push(item.item_id);
        }
    }

    fn is_item_in_inventory(&self, item_id: u32) -> bool {
        self.inventory.contains(&item_id)
    }

    async fn handle_mouse_click(&mut self, game_pos: Vec2) {
        if !self
            .game_rect
            .contains(self.get_scaled_pos(game_pos.x, game_pos.y).into())
        {
            return;
        }

        // Check if a character was clicked
        if let Some(index) =
            (0..self.characters.count).find(|&i| self.is_point_in_character(game_pos, i))
        {
            if Some(index) != self.active_character {
                self.active_character = Some(index);
                return;
            }
        }

        // Check for scene transitions and handle scene changes
        if let Some(transition) = self.find_clicked_transition(game_pos) {
            if self.debug_instant_move || self.is_active_character_in_transition_area(transition) {
                let current_scene_id = self.current_scene;
                self.current_scene = transition.target_scene;
                self.transition_to_new_scene(current_scene_id).await;
                return;
            }
        }

        // Handle item clicks
        self.handle_item_click(game_pos);

        // Handle pathfinding
        self.handle_pathfinding(game_pos).await;
    }

    fn handle_right_click(&mut self, game_pos: Vec2) {
        if self.debug_tools.bounding_box_mode {
            self.debug_tools.handle_bounding_box_creation(game_pos);
        }
    }

    fn is_point_in_character(&self, point: Vec2, character_index: usize) -> bool {
        let character_width = 55.0;
        let character_height = 120.0;
        let character_pos = self.characters.positions[character_index];

        point.x >= character_pos.x - character_width / 2.0
            && point.x <= character_pos.x + character_width / 2.0
            && point.y >= character_pos.y - character_height / 2.0
            && point.y <= character_pos.y + character_height / 2.0
    }

    fn is_active_character_in_transition_area(&self, transition: &SceneTransition) -> bool {
        if let Some(active_index) = self.active_character {
            let character_pos = self.characters.positions[active_index];
            character_pos.x >= transition.x
                && character_pos.x <= transition.x + transition.width
                && character_pos.y >= transition.y
                && character_pos.y <= transition.y + transition.height
        } else {
            false
        }
    }

    fn find_clicked_transition(&self, game_pos: Vec2) -> Option<&SceneTransition> {
        self.get_current_scene().and_then(|current_scene| {
            current_scene.scene_transitions.iter().find(|transition| {
                game_pos.x >= transition.x
                    && game_pos.x <= transition.x + transition.width
                    && game_pos.y >= transition.y
                    && game_pos.y <= transition.y + transition.height
            })
        })
    }

    async fn transition_to_new_scene(&mut self, previous_scene_id: u32) {
        let transition_data = self.get_transition_data(previous_scene_id);

        if let Some((transition_area, blocked_nodes)) = transition_data {
            // Update character positions
            let spawn_position = Vec2::new(
                transition_area.x + transition_area.width / 2.0,
                transition_area.y + transition_area.height / 2.0,
            );
            let spawn_positions =
                self.generate_spawn_positions(spawn_position, self.characters.count);

            for (i, pos) in spawn_positions.into_iter().enumerate() {
                if i < self.characters.count {
                    self.characters.positions[i] = pos;
                    self.characters.directions[i] = Direction::South;
                    self.characters.paths[i] = None;
                    self.characters.targets[i] = None;
                }
            }

            // Update blocked nodes
            self.grid.update_blocked_nodes(blocked_nodes);
        }

        self.load_current_and_adjacent_scenes().await;
    }

    fn get_transition_data(
        &self,
        previous_scene_id: u32,
    ) -> Option<(SceneTransition, Vec<(i32, i32)>)> {
        self.get_current_scene().and_then(|current_scene| {
            current_scene
                .scene_transitions
                .iter()
                .find(|t| t.target_scene == previous_scene_id)
                .map(|transition| (transition.clone(), current_scene.blocked_nodes.clone()))
        })
    }

    fn generate_spawn_positions(&self, center: Vec2, count: usize) -> Vec<Vec2> {
        let mut positions = Vec::with_capacity(count);
        let spacing = 80.0;

        for i in 0..count {
            let x_offset = (i as f32 - (count - 1) as f32 / 2.0) * spacing;
            let pos = Vec2::new(center.x + x_offset, center.y);
            positions.push(pos);
        }

        positions
    }

    async fn handle_pathfinding(&mut self, target_pos: Vec2) {
        if let Some(active_index) = self.active_character {
            let target_grid = self.grid.get_grid_from_coord(target_pos);
            let mut final_target = target_grid;

            // Check if the click is within a scene transition area
            if let Some(transition) = self.find_clicked_transition(target_pos) {
                if !self.grid.is_node_walkable(target_grid) {
                    // Find the closest walkable node within the transition area
                    if let Some(closest_node) =
                        self.find_closest_walkable_node(target_grid, transition)
                    {
                        final_target = closest_node;
                    } else {
                        // No walkable nodes in the transition area, don't move
                        self.stop_character(active_index);
                        return;
                    }
                }
            } else if !self.grid.is_node_walkable(target_grid) {
                // If not in a transition area and not walkable, don't move
                self.stop_character(active_index);
                return;
            }

            let start_grid = self
                .grid
                .get_grid_from_coord(self.characters.positions[active_index]);

            if let Some(path) = self.pathfind(start_grid, final_target) {
                self.characters.paths[active_index] = Some(path);
                self.characters.targets[active_index] = Some(final_target);
            } else {
                // If no path is found, stop the character
                self.stop_character(active_index);
            }
        }
    }

    fn find_closest_walkable_node(
        &self,
        target: (i32, i32),
        transition: &SceneTransition,
    ) -> Option<(i32, i32)> {
        let start = self
            .grid
            .get_grid_from_coord(Vec2::new(transition.x, transition.y));
        let end = self.grid.get_grid_from_coord(Vec2::new(
            transition.x + transition.width,
            transition.y + transition.height,
        ));

        let mut closest_node = None;
        let mut min_distance = std::i32::MAX;

        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                if self.grid.is_node_walkable((x, y)) {
                    let distance = ((x - target.0).pow(2) + (y - target.1).pow(2)) as i32;
                    if distance < min_distance {
                        min_distance = distance;
                        closest_node = Some((x, y));
                    }
                }
            }
        }

        closest_node
    }

    async fn update(&mut self) {
        self.update_window_size();

        let mouse_pos = Vec2::from(mouse_position());
        let game_pos = self.get_game_coordinates(mouse_pos);

        if is_mouse_button_pressed(MouseButton::Left) {
            self.handle_mouse_click(game_pos).await;
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            self.handle_right_click(game_pos);
        }

        if is_key_pressed(KeyCode::G) {
            self.debug_tools.active = !self.debug_tools.active;
        }
        if is_key_pressed(KeyCode::J) {
            self.debug_tools.draw_grid = !self.debug_tools.draw_grid;
        }

        if is_key_pressed(KeyCode::F3) {
            self.debug_instant_move = !self.debug_instant_move;
            println!("Debug instant move: {}", self.debug_instant_move);
        }

        if is_key_pressed(KeyCode::B) {
            self.debug_tools.bounding_box_mode = !self.debug_tools.bounding_box_mode;
        }

        // Animation speed controls
        if is_key_pressed(KeyCode::Up) {
            for i in 0..self.characters.count {
                self.characters.animation_speeds[i] -= 0.01;
            }
        }
        if is_key_pressed(KeyCode::Down) {
            for i in 0..self.characters.count {
                self.characters.animation_speeds[i] += 0.01;
            }
        }

        // Update cursor based on game position
        let new_cursor_type = self.determine_cursor(game_pos);
        if new_cursor_type != self.current_cursor {
            self.set_cursor(new_cursor_type);
        }

        let delta_time = get_frame_time();
        self.update_characters(delta_time);
    }

    fn update_characters(&mut self, delta_time: f32) {
        for i in 0..self.characters.count {
            if let Some(path) = &mut self.characters.paths[i] {
                if !path.is_empty() {
                    let target = self.grid.get_coord_from_grid(path[0].0, path[0].1);
                    let direction = (target - self.characters.positions[i]).normalize();
                    let speed =
                        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                            self.characters.data[i].run_speed
                        } else {
                            self.characters.data[i].speed
                        };

                    self.characters.positions[i] += direction * speed * delta_time;
                    self.characters.directions[i] = Self::vec_to_direction(direction);

                    self.characters.animation_timers[i] += delta_time;
                    if self.characters.animation_timers[i] >= self.characters.animation_speeds[i] {
                        self.characters.animation_timers[i] -= self.characters.animation_speeds[i];
                        self.characters.animation_indices[i] =
                            (self.characters.animation_indices[i] + 1) % 8;
                    }

                    if (self.characters.positions[i] - target).length() < 5.0 {
                        path.remove(0);
                        if path.is_empty() {
                            self.stop_character(i);
                        }
                    }
                } else {
                    self.stop_character(i);
                }
            } else {
                // Ensure the animation is reset even if there was no path
                self.reset_character_animation(i);
            }
        }
    }

    fn stop_character(&mut self, index: usize) {
        self.characters.paths[index] = None;
        self.characters.targets[index] = None;
        self.reset_character_animation(index);
    }

    fn reset_character_animation(&mut self, index: usize) {
        self.characters.animation_indices[index] = 0;
        self.characters.animation_timers[index] = 0.0;
    }

    fn draw(&self) {
        clear_background(BLACK);

        if let Some(current_scene) = self.get_current_scene() {
            self.draw_scene(current_scene);
        } else {
            self.draw_error_message("Scene not found");
        }

        self.draw_inventory();
        self.draw_debug();
        self.draw_ui();
    }

    fn draw_debug(&self) {
        if self.debug_tools.active {
            if self.debug_tools.draw_grid {
                self.draw_debug_grid();
            }
            self.draw_debug_info();
        }
    }

    fn draw_character(&self, index: usize, scale: f32, is_active: bool) {
        let (x, y) = self.get_scaled_pos(
            self.characters.positions[index].x,
            self.characters.positions[index].y,
        );

        let cycle = if self.characters.animation_indices[index] < 4 {
            0
        } else {
            7
        };
        let frame = self.characters.animation_indices[index] % 4;

        let filename = format!(
            "{}{}{}{}.png",
            self.characters.data[index].name, self.characters.directions[index] as u8, frame, cycle
        );

        let x_offset = 0.0;
        let y_offset = 0.0;

        match self.character_textures.get(&filename) {
            Some(texture) => {
                let xt = texture.width() / 2.0 * scale;
                let yt = texture.height() / 2.0 * scale;
                draw_texture_ex(
                    texture,
                    x - xt + x_offset,
                    y - yt + y_offset,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            texture.width() * scale,
                            texture.height() * scale,
                        )),
                        ..Default::default()
                    },
                );
            }
            None => {
                println!("Texture not found for filename: {}", filename);
                draw_rectangle(x, y, 50.0 * scale, 50.0 * scale, RED);
            }
        }

        if is_active {
            let indicator_size = 10.0 * scale;
            draw_circle(
                x + x_offset,
                y - 60.0 * scale + y_offset,
                indicator_size,
                GREEN,
            );
        }
    }

    fn draw_scene(&self, scene: &Scene) {
        if let Some(texture) = self.textures.get(&scene.background) {
            self.draw_background(texture);

            self.draw_world_items();

            let scale = self.get_scale();

            for i in 0..self.characters.count {
                // TODO: some overlays rely on the character's y position
                let _character_y = self.characters.positions[i].y;

                let is_active = self.active_character == Some(i);
                self.draw_character(i, scale, is_active);

                // // Draw overlays that should appear in front of this character
                // for overlay in &sorted_overlays {
                //     if overlay.z_value > character_y as usize {
                //         self.draw_overlay_asset(overlay);
                //     }
                // }
            }

            // TODO: More sophisticated overlay sorting
            for overlay in &scene.overlay_assets {
                self.draw_overlay_asset(overlay);
            }

            //self.draw_clickable_areas();
            self.draw_scene_description(scene);
        } else {
            self.draw_loading_message(&scene.background);
        }
    }

    fn draw_overlay_asset(&self, overlay: &OverlayAsset) {
        if let Some(texture) = self.textures.get(&overlay.texture_path) {
            // We are gathering the overlay positions
            // from the original low-res .bmp files.
            // We need to scale them up to match
            // the high-res textures.
            let (ox, oy) = (overlay.x * 3.0, overlay.y * 3.0);
            let (x, y) = self.get_scaled_pos(ox, oy);
            let scale = self.get_scale();
            draw_texture_ex(
                texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(texture.width() * scale, texture.height() * scale)),
                    ..Default::default()
                },
            );
        } else {
            println!("Overlay texture not found: {}", overlay.texture_path);
        }
    }

    fn draw_background(&self, texture: &Texture2D) {
        draw_texture_ex(
            texture,
            self.game_rect.x,
            self.game_rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.game_rect.w, self.game_rect.h)),
                ..Default::default()
            },
        );
    }

    fn draw_world_items(&self) {
        let current_scene_items = &self.world_items[self.current_scene as usize];
        let mouse_pos = Vec2::from(mouse_position());
        let game_pos = self.get_game_coordinates(mouse_pos);

        for item_instance in current_scene_items {
            let item = self
                .items
                .iter()
                .find(|i| i.id == item_instance.item_id)
                .unwrap();
            let texture_path = if self.is_mouse_over_item(game_pos, item_instance) {
                &item.textures.mouse_over
            } else {
                &item.textures.in_world
            };

            if let Some(texture) = self.textures.get(texture_path) {
                let (x, y) = self.get_scaled_pos(item_instance.x, item_instance.y);
                let scale = self.get_scale();
                draw_texture_ex(
                    texture,
                    x,
                    y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            item_instance.width * scale,
                            item_instance.height * scale,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn is_mouse_over_item(&self, game_pos: Vec2, item: &ItemInstance) -> bool {
        game_pos.x >= item.x
            && game_pos.x <= item.x + item.width
            && game_pos.y >= item.y
            && game_pos.y <= item.y + item.height
    }

    fn draw_inventory(&self) {
        let inventory_start_x = 20.0;
        let inventory_start_y = self.game_rect.h - 100.0;
        let item_size = 60.0;
        let item_spacing = 10.0;

        for (index, item_id) in self.inventory.iter().enumerate() {
            if let Some(item) = self.items.iter().find(|i| i.id == *item_id) {
                if let Some(texture) = self.textures.get(&item.textures.in_inventory) {
                    let x = inventory_start_x + (item_size + item_spacing) * index as f32;
                    let (draw_x, draw_y) = self.get_scaled_pos(x, inventory_start_y);
                    let scale = self.get_scale();
                    draw_texture_ex(
                        texture,
                        draw_x,
                        draw_y,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(item_size * scale, item_size * scale)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    // fn draw_clickable_areas(&self) {
    //     if let Some(current_scene) = self.get_current_scene() {
    //         for area in &current_scene.clickable_areas {
    //             let (x, y) = self.get_scaled_pos(area.x, area.y);
    //             let width = area.width * self.get_scale();
    //             let height = area.height * self.get_scale();
    //
    //             // Draw clickable area
    //             draw_rectangle_lines(x, y, width, height, 2.0, YELLOW);
    //         }
    //
    //         // Also draw scene transitions
    //         for transition in &current_scene.scene_transitions {
    //             let (x, y) = self.get_scaled_pos(transition.area.x, transition.area.y);
    //             let width = transition.area.width * self.get_scale();
    //             let height = transition.area.height * self.get_scale();
    //
    //             // Draw scene transition area
    //             draw_rectangle_lines(x, y, width, height, 2.0, BLUE);
    //         }
    //     }
    // }

    fn draw_clickable_area(&self, area: &ClickableArea) {
        let (x, y) = self.get_scaled_pos(area.x, area.y);
        let width = area.width * self.get_scale();
        let height = area.height * self.get_scale();
        draw_rectangle_lines(x, y, width, height, 2.0, RED);
    }

    fn draw_scene_description(&self, scene: &Scene) {
        let (desc_x, desc_y) = self.get_scaled_pos(20.0, 20.0);
        draw_text(
            format!("#{} - {}", scene.id, scene.description).as_str(),
            desc_x,
            desc_y,
            30.0 * self.get_scale(),
            RED,
        );
    }

    fn draw_loading_message(&self, background: &str) {
        let (text_x, text_y) = self.get_scaled_pos(20.0, 20.0);
        draw_text(
            &format!("Loading texture: {}", background),
            text_x,
            text_y,
            30.0 * self.get_scale(),
            YELLOW,
        );
    }

    fn draw_error_message(&self, message: &str) {
        let (text_x, text_y) = self.get_scaled_pos(20.0, 20.0);
        draw_text(message, text_x, text_y, 30.0 * self.get_scale(), RED);
    }

    fn draw_ui(&self) {
        for menu_item in &self.ui.menu_items {
            if let Some(texture) = self.menu_item_textures.get(&menu_item.name) {
                let (x, y) = self.get_scaled_pos(menu_item.position[0], menu_item.position[1]);
                let scale = self.get_scale();
                draw_texture_ex(
                    texture,
                    x,
                    y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            menu_item.size[0] * scale,
                            menu_item.size[1] * scale,
                        )),
                        ..Default::default()
                    },
                );
            }
        }

        if let Some(cursor_texture) = self.cursor_textures.get(&self.current_cursor) {
            let cursor_pos = mouse_position();
            if let Some(cursor) = self
                .ui
                .cursors
                .iter()
                .find(|c| c.cursor_type == self.current_cursor)
            {
                let scale = self.get_scale();
                draw_texture_ex(
                    cursor_texture,
                    cursor_pos.0 - (cursor.hotspot[0] as f32 * scale),
                    cursor_pos.1 - (cursor.hotspot[1] as f32 * scale),
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(
                            cursor_texture.width() * scale,
                            cursor_texture.height() * scale,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
    }

    fn draw_debug_info(&self) {
        let (text_x, text_y) = self.get_scaled_pos(20.0, 60.0);
        draw_text(
            &format!("Characters: {}", self.characters.count),
            text_x,
            text_y,
            20.0 * self.get_scale(),
            WHITE,
        );

        if self.characters.count > 0 {
            let (text_x, text_y) = self.get_scaled_pos(20.0, 90.0);
            draw_text(
                &format!(
                    "Animation Speed: {:.2}",
                    self.characters.animation_speeds[0]
                ),
                text_x,
                text_y,
                20.0 * self.get_scale(),
                WHITE,
            );
        }

        if self.debug_tools.bounding_box_mode {
            self.draw_bounding_box_info();
        }

        self.draw_scene_transitions();
    }

    fn draw_scene_transitions(&self) {
        if let Some(current_scene) = self.get_current_scene() {
            for transition in &current_scene.scene_transitions {
                let (x, y) = self.get_scaled_pos(transition.x, transition.y);
                let width = transition.width * self.get_scale();
                let height = transition.height * self.get_scale();

                // Draw transition area
                draw_rectangle_lines(x, y, width, height, 2.0, BLUE);
                let text = format!("#{}", transition.target_scene);
                draw_text(&text, x, y, 40.0 * self.get_scale(), WHITE);
            }
        }
    }

    fn draw_debug_grid(&self) {
        let grid_color = Color::new(0.0, 1.0, 0.0, 0.5);
        let scale = self.get_scale();

        let grid_width = 41;
        let grid_height = 41;

        let x_delta =
            (self.grid.get_coord_from_grid(1, 0) - self.grid.get_coord_from_grid(0, 0)) / 2.0;
        for x in 0..=grid_width {
            let start = self.grid.get_coord_from_grid(x, 0) - x_delta;
            let end = self.grid.get_coord_from_grid(x, grid_height) - x_delta;
            let start = self.get_scaled_pos(start.x, start.y);
            let end = self.get_scaled_pos(end.x, end.y);
            draw_line(start.0, start.1, end.0, end.1, 2.0, grid_color);
        }

        let y_delta =
            (self.grid.get_coord_from_grid(0, 1) - self.grid.get_coord_from_grid(0, 0)) / 2.0;
        for y in 0..=grid_height {
            let start = self.grid.get_coord_from_grid(0, y) - y_delta;
            let end = self.grid.get_coord_from_grid(grid_width, y) - y_delta;
            let start = self.get_scaled_pos(start.x, start.y);
            let end = self.get_scaled_pos(end.x, end.y);
            draw_line(start.0, start.1, end.0, end.1, 2.0, grid_color);
        }

        let font_size = 20.0 * scale;
        //let (x_offset, y_offset) = self.get_scaled_pos(25.0, 5.0);
        let (x_offset, y_offset) = (0.0, 0.0);
        for x in 0..=grid_width {
            for y in 0..=grid_height {
                let pos = self.grid.get_coord_from_grid(x, y);
                let (draw_x, draw_y) = self.get_scaled_pos(pos.x, pos.y);

                // Draw black circle for blocked nodes
                if self.grid.blocked_nodes.contains(&(x, y)) {
                    let circle_radius = 5.0 * scale;
                    draw_circle(draw_x, draw_y, circle_radius, BLACK);
                }

                let color = if self.grid.is_node_walkable((x, y)) {
                    WHITE
                } else {
                    RED
                };
                draw_text(
                    &format!("{},{}", x, y),
                    draw_x + x_offset,
                    draw_y + y_offset,
                    font_size,
                    color,
                );
            }
        }
    }

    fn draw_bounding_box_info(&self) {
        let (text_x, text_y) = self.get_scaled_pos(20.0, self.game_rect.h - 40.0);
        draw_text(
            "Bounding Box Mode: ON",
            text_x,
            text_y,
            20.0 * self.get_scale(),
            GREEN,
        );

        if let Some(rect) = self.debug_tools.current_bounding_box {
            let (x, y) = self.get_scaled_pos(rect.x, rect.y);
            let width = rect.w * self.get_scale();
            let height = rect.h * self.get_scale();
            draw_rectangle_lines(x, y, width, height, 2.0, GREEN);
        }
    }
}

#[macroquad::main("OpenJönsson")]
async fn main() {
    show_mouse(false);
    match Game::new().await {
        Ok(mut game) => loop {
            game.update().await;
            game.draw();
            next_frame().await
        },
        Err(e) => {
            eprintln!("Failed to initialize game: {}", e);
        }
    }
}
