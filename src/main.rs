use macroquad::prelude::*;
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
mod character;
use character::{Character, CharacterData, Direction};

#[derive(Deserialize, Debug, Clone)]
struct ClickableArea {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    #[serde(rename = "targetScene")]
    target_scene: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct Scene {
    id: u32,
    description: String,
    background: String,
    #[serde(rename = "clickableAreas")]
    clickable_areas: Vec<ClickableArea>,
}

#[derive(Deserialize, Debug, Clone)]
struct Level {
    id: u32,
    name: String,
    scenes: Vec<Scene>,
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
}

impl Grid {
    fn new() -> Self {
        Self {
            a: 0.261,
            m: -1.744,
            stretch: (38.81, 10.32),
            grid_offset: 10.,
        }
    }

    fn get_grid_from_coord(&self, v: Vec2) -> (i32, i32) {
        // Adjust the input coordinates to match the original game's coordinate system
        let v = Vec2::new(v.x / 3.0, v.y / 3.0); // Scale up by 3 to match original game's resolution

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
}

struct DebugTools {
    bounding_box_mode: bool,
    bounding_box_start: Option<Vec2>,
    current_bounding_box: Option<Rect>,
    draw_grid: bool,
}

impl DebugTools {
    fn new() -> Self {
        DebugTools {
            bounding_box_mode: false,
            bounding_box_start: None,
            current_bounding_box: None,
            draw_grid: false,
        }
    }

    fn toggle_bounding_box_mode(&mut self) {
        self.bounding_box_mode = !self.bounding_box_mode;
        self.bounding_box_start = None;
        self.current_bounding_box = None;
        println!(
            "Bounding box mode: {}",
            if self.bounding_box_mode { "ON" } else { "OFF" }
        );
    }

    fn handle_bounding_box_creation(&mut self, game_coordinates: (f32, f32)) {
        let (game_x, game_y) = game_coordinates;

        if let Some(start) = self.bounding_box_start {
            let width = game_x - start.x;
            let height = game_y - start.y;
            let (x, y) = if width < 0.0 || height < 0.0 {
                (game_x.min(start.x), game_y.min(start.y))
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
            self.bounding_box_start = Some(Vec2::new(game_x, game_y));
            self.current_bounding_box = None;
        }
    }

    fn draw_bounding_box_info(&self, game: &Game) {
        let (text_x, text_y) = game.get_scaled_pos(20.0, game.game_rect.h - 40.0);
        draw_text(
            "Bounding Box Mode: ON",
            text_x,
            text_y,
            20.0 * game.get_scale(),
            GREEN,
        );

        if let Some(rect) = self.current_bounding_box {
            let (x, y) = game.get_scaled_pos(rect.x, rect.y);
            let width = rect.w * game.get_scale();
            let height = rect.h * game.get_scale();
            draw_rectangle_lines(x, y, width, height, 2.0, GREEN);
        }
    }

    fn toggle_grid(&mut self) {
        self.draw_grid = !self.draw_grid;
        println!(
            "Grid display: {}",
            if self.draw_grid { "ON" } else { "OFF" }
        );
    }
}

struct Game {
    textures: HashMap<String, Texture2D>,
    current_level: u32,
    current_scene: u32,
    game_data: GameData,
    loading_textures: HashSet<String>,
    window_size: Vec2,
    game_rect: Rect,
    debug_tools: Option<DebugTools>,
    characters: Vec<Character>,
    character_textures: HashMap<String, Texture2D>,
    active_character: Option<usize>,
    grid: Grid,
    cursor_textures: HashMap<CursorType, Texture2D>,
    menu_item_textures: HashMap<String, Texture2D>,
    current_cursor: CursorType,
    debug_instant_move: bool,
    changing_scene: bool,
}

impl Game {
    async fn new() -> Result<Self, String> {
        let json = load_string("static/level_data.json").await.unwrap();
        let game_data: GameData =
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let window_size = Vec2::new(screen_width(), screen_height());
        let game_rect = Game::calculate_game_rect(window_size);

        let mut game = Game {
            textures: HashMap::new(),
            current_level: 1,
            current_scene: 1,
            game_data,
            loading_textures: HashSet::new(),
            window_size,
            game_rect,
            debug_tools: Some(DebugTools::new()),
            characters: Vec::new(),
            character_textures: HashMap::new(),
            active_character: Some(0),
            grid: Grid::new(),
            cursor_textures: HashMap::new(),
            menu_item_textures: HashMap::new(),
            current_cursor: CursorType::Normal,
            debug_instant_move: false,
            changing_scene: false,
        };

        game.load_current_and_adjacent_scenes().await;
        game.load_characters().await;
        game.load_debug_textures().await;
        game.load_ui_textures().await;

        Ok(game)
    }

    fn current_level(&self) -> Option<&Level> {
        self.game_data
            .levels
            .iter()
            .find(|l| l.id == self.current_level)
    }

    fn current_scene(&self) -> Option<&Scene> {
        self.current_level()
            .and_then(|level| level.scenes.iter().find(|s| s.id == self.current_scene))
    }

    fn get_scene(&self, scene_id: u32) -> Option<&Scene> {
        self.current_level()
            .and_then(|level| level.scenes.iter().find(|s| s.id == scene_id))
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

    async fn load_texture(&mut self, bg: &str) -> Result<(), String> {
        if self.textures.contains_key(bg) || self.loading_textures.contains(bg) {
            return Ok(());
        }

        self.loading_textures.insert(bg.to_string());
        let texture_path = format!("static/resources/{}", bg);
        match load_texture(&texture_path).await {
            Ok(texture) => {
                self.textures.insert(bg.to_string(), texture);
                self.loading_textures.remove(bg);
                Ok(())
            }
            Err(e) => {
                self.loading_textures.remove(bg);
                Err(format!("Failed to load texture {}: {}", bg, e))
            }
        }
    }

    async fn load_textures(&mut self, backgrounds: Vec<String>) {
        for bg in backgrounds {
            if let Err(e) = self.load_texture(&bg).await {
                eprintln!("{}", e);
            }
        }
    }

    async fn load_current_and_adjacent_scenes(&mut self) {
        let mut backgrounds_to_load = Vec::new();

        if let Some(current_scene) = self.current_scene() {
            backgrounds_to_load.push(current_scene.background.clone());

            for area in &current_scene.clickable_areas {
                if let Some(target_scene) = self.get_scene(area.target_scene) {
                    backgrounds_to_load.push(target_scene.background.clone());
                }
            }
        }

        self.load_textures(backgrounds_to_load).await;
    }

    async fn load_characters(&mut self) {
        for (index, character_data) in self.game_data.characters.iter().enumerate() {
            let start_x = 100.0 + (index as f32 * 100.0);
            let start_y = 100.0;
            let character = Character::new(character_data.clone(), Vec2::new(start_x, start_y));
            self.characters.push(character);

            // Load character textures
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
        for cursor in &self.game_data.ui.cursors {
            if let Ok(texture) = load_texture(&format!("static/resources/{}", cursor.texture)).await
            {
                self.cursor_textures.insert(cursor.cursor_type, texture);
            }
        }

        for menu_item in &self.game_data.ui.menu_items {
            if let Ok(texture) =
                load_texture(&format!("static/resources/{}", menu_item.texture)).await
            {
                self.menu_item_textures
                    .insert(menu_item.name.clone(), texture);
            }
        }
    }

    fn set_cursor(&mut self, cursor_type: CursorType) {
        if self.cursor_textures.contains_key(&cursor_type) {
            self.current_cursor = cursor_type;
        }
    }

    fn draw_ui(&self) {
        // Draw menu items (unchanged)
        for menu_item in &self.game_data.ui.menu_items {
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

        // Draw custom cursor
        if let Some(cursor_texture) = self.cursor_textures.get(&self.current_cursor) {
            let cursor_pos = mouse_position();
            if let Some(cursor) = self
                .game_data
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

    async fn update(&mut self) {
        self.update_window_size();

        let mouse_pos = Vec2::from(mouse_position());
        let game_pos = self.get_game_coordinates(mouse_pos);

        if is_mouse_button_pressed(MouseButton::Left) {
            self.handle_mouse_click(game_pos).await;
        }

        if is_key_pressed(KeyCode::G) {
            if let Some(debug_tools) = &mut self.debug_tools {
                debug_tools.toggle_grid();
            }
        }

        // Toggle debug mode
        if is_key_pressed(KeyCode::F3) {
            // You can change F3 to any key you prefer
            self.debug_instant_move = !self.debug_instant_move;
            println!("Debug instant move: {}", self.debug_instant_move);
        }

        if is_key_pressed(KeyCode::B) {
            if let Some(debug_tools) = &mut self.debug_tools {
                debug_tools.toggle_bounding_box_mode();
            }
        }

        // Animation speed controls
        if is_key_pressed(KeyCode::Up) {
            for character in &mut self.characters {
                character.set_animation_speed(character.animation_speed - 0.01);
            }
        }
        if is_key_pressed(KeyCode::Down) {
            for character in &mut self.characters {
                character.set_animation_speed(character.animation_speed + 0.01);
            }
        }

        let new_cursor_type = self.determine_cursor(game_pos);

        // Set the new cursor type if it has changed
        if new_cursor_type != self.current_cursor {
            self.set_cursor(new_cursor_type);
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let game_coordinates = self.get_game_coordinates(mouse_position().into());

            if let Some(debug_tools) = &mut self.debug_tools {
                if debug_tools.bounding_box_mode {
                    debug_tools.handle_bounding_box_creation(game_coordinates.into());
                } else {
                    self.handle_mouse_click(game_coordinates).await;
                }
            } else {
                self.handle_mouse_click(game_coordinates).await;
            }
        }

        let delta_time = get_frame_time();
        let grid = &self.grid; // Create a reference to avoid borrowing issues

        for character in &mut self.characters {
            let mut remove_first = false;

            if let Some(path) = &character.path {
                if !path.is_empty() {
                    let target = grid.get_coord_from_grid(path[0].0, path[0].1);
                    let direction = (target - character.position).normalize();
                    let speed =
                        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                            character.data.run_speed
                        } else {
                            character.data.speed
                        };

                    character.position += direction * speed * delta_time;
                    character.direction = Self::vec_to_direction(direction);

                    character.animation_timer += delta_time;
                    if character.animation_timer >= character.animation_speed {
                        character.animation_timer -= character.animation_speed;
                        character.animation_index = (character.animation_index + 1) % 8;
                    }

                    if (character.position - target).length() < 5.0 {
                        remove_first = true;
                    }
                }
            }

            if remove_first {
                if let Some(path) = &mut character.path {
                    path.remove(0);
                    if path.is_empty() {
                        character.path = None;
                        character.target = None;
                        character.animation_index = 0;
                    }
                }
            }
        }
    }

    fn determine_cursor(&self, game_pos: Vec2) -> CursorType {
        if let Some(current_scene) = self.current_scene() {
            for area in &current_scene.clickable_areas {
                if game_pos.x >= area.x
                    && game_pos.x <= area.x + area.width
                    && game_pos.y >= area.y
                    && game_pos.y <= area.y + area.height
                {
                    return CursorType::Move;
                }
            }
        }
        CursorType::Normal
    }
    // Move vec_to_direction out of Character impl and make it a standalone function
    fn vec_to_direction(vec: Vec2) -> Direction {
        if vec.x == 0.0 && vec.y == 0.0 {
            return Direction::South; // Default direction
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

    fn heuristic(&self, a: (i32, i32), b: (i32, i32)) -> i32 {
        let dx = (a.0 - b.0).abs();
        let dy = (a.1 - b.1).abs();
        (dx + dy) + (1414 - 1000) * dx.min(dy)
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
            .filter(|&(x, y)| x >= 0 && x < 41 && y >= 0 && y < 41) // Adjust grid size as needed
            .collect()
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
    fn get_character_movement(&self) -> Vec2 {
        let mut movement = Vec2::new(0.0, 0.0);
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            let c = 0.4472;
            movement.x -= c * 2.0;
            movement.y -= c * 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            let c = 0.4472;
            movement.x += c * 2.0;
            movement.y += c * 1.0;
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
            let c = 0.4452;
            movement.x -= c * -2.016;
            movement.y -= c * 1.0;
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
            let c = 0.4452;
            movement.x += c * -2.016;
            movement.y += c * 1.0;
        }
        movement
    }

    fn update_window_size(&mut self) {
        let new_window_size = Vec2::new(screen_width(), screen_height());
        if new_window_size != self.window_size {
            self.window_size = new_window_size;
            self.game_rect = Game::calculate_game_rect(self.window_size);
        }
    }

    async fn handle_mouse_click(&mut self, game_pos: Vec2) {
        if is_mouse_button_pressed(MouseButton::Left) {
            // Check if a character was clicked
            if let Some(index) = self
                .characters
                .iter()
                .position(|character| self.is_point_in_character(game_pos, character))
            {
                self.active_character = Some(index);
                println!("Character {} selected", self.characters[index].data.name);
                return;
            }

            // Check for clickable areas and handle scene changes
            if let Some(area) = self.find_clicked_area(game_pos) {
                if self.debug_instant_move || self.is_active_character_in_area(&area) {
                    // Perform scene change
                    let current_scene_id = self.current_scene;
                    self.current_scene = area.target_scene;
                    self.changing_scene = true;
                    self.transition_to_new_scene(current_scene_id).await;

                    return; // Exit after scene change
                } else {
                }
            }

            // If we haven't returned yet, handle pathfinding
            if !self.changing_scene {
                self.handle_pathfinding(game_pos).await;
            }
            self.changing_scene = false;
        }
    }

    async fn handle_pathfinding(&mut self, target_pos: Vec2) {
        if let Some(active_index) = self.active_character {
            let target_grid = self.grid.get_grid_from_coord(target_pos);
            let start_grid = self
                .grid
                .get_grid_from_coord(self.characters[active_index].position);

            if let Some(path) = self.pathfind(start_grid, target_grid) {
                self.characters[active_index].path = Some(path);
                self.characters[active_index].target = Some(target_grid);
            }
        }
    }

    async fn transition_to_new_scene(&mut self, previous_scene_id: u32) {
        if let Some(spawn_point) = self.find_spawn_point(previous_scene_id) {
            let spawn_positions = self.generate_spawn_positions(spawn_point, self.characters.len());

            // Move characters to their respective spawn positions
            for (character, &position) in self.characters.iter_mut().zip(spawn_positions.iter()) {
                character.position = position;
                character.direction = Direction::South;
                character.path = None;
                character.target = None;
            }
        }

        self.load_current_and_adjacent_scenes().await;
    }

    fn find_spawn_point(&self, previous_scene_id: u32) -> Option<Vec2> {
        if let Some(current_scene) = self.get_scene(self.current_scene) {
            for area in &current_scene.clickable_areas {
                if area.target_scene == previous_scene_id {
                    // Return the center of the clickable area
                    return Some(Vec2::new(
                        area.x + area.width / 2.0,
                        area.y + area.height / 2.0,
                    ));
                }
            }
        }
        None
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

    fn is_point_in_character(&self, point: Vec2, character: &Character) -> bool {
        // You may need to adjust these values based on your character size
        let character_width = 55.0;
        let character_height = 120.0;

        point.x >= character.position.x - character_width / 2.0
            && point.x <= character.position.x + character_width / 2.0
            && point.y >= character.position.y - character_height / 2.0
            && point.y <= character.position.y + character_height / 2.0
    }

    fn is_active_character_in_area(&self, area: &ClickableArea) -> bool {
        if let Some(active_index) = self.active_character {
            let character = &self.characters[active_index];
            character.position.x >= area.x
                && character.position.x <= area.x + area.width
                && character.position.y >= area.y
                && character.position.y <= area.y + area.height
        } else {
            false
        }
    }

    fn find_clicked_area(&self, game_pos: Vec2) -> Option<&ClickableArea> {
        self.current_scene().and_then(|scene| {
            scene.clickable_areas.iter().find(|area| {
                game_pos.x >= area.x
                    && game_pos.x <= area.x + area.width
                    && game_pos.y >= area.y
                    && game_pos.y <= area.y + area.height
            })
        })
    }

    fn draw_debug_grid(&self) {
        if let Some(debug_tools) = &self.debug_tools {
            if !debug_tools.draw_grid {
                return;
            }

            if let Some(texture) = self.textures.get("debug_grid") {
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

            let grid_color = Color::new(0.0, 1.0, 0.0, 0.5); // Semi-transparent green
            let scale = self.get_scale();

            // Calculate the number of grid cells based on the original resolution
            let grid_width = 41; // From 0 to 40 inclusive
            let grid_height = 41;

            // The get_coord_from_grid function returns the center of the grid cell
            // We need to adjust the start and end points to draw the lines in between the cells
            // Find the difference between two adjacent centers and divide by 2 to find the middle
            // point between them
            let x_delta =
                (self.grid.get_coord_from_grid(1, 0) - self.grid.get_coord_from_grid(0, 0)) / 2.0;
            // Draw vertical lines
            for x in 0..=grid_width {
                let start = self.grid.get_coord_from_grid(x, 0) - x_delta;
                let end = self.grid.get_coord_from_grid(x, grid_height) - x_delta;
                let start = self.get_scaled_pos(start.x, start.y);
                let end = self.get_scaled_pos(end.x, end.y);
                draw_line(start.0, start.1, end.0, end.1, 2.0, grid_color);
            }

            let y_delta =
                (self.grid.get_coord_from_grid(0, 1) - self.grid.get_coord_from_grid(0, 0)) / 2.0;
            // Draw horizontal lines
            for y in 0..=grid_height {
                let start = self.grid.get_coord_from_grid(0, y) - y_delta;
                let end = self.grid.get_coord_from_grid(grid_width, y) - y_delta;
                let start = self.get_scaled_pos(start.x, start.y);
                let end = self.get_scaled_pos(end.x, end.y);
                draw_line(start.0, start.1, end.0, end.1, 2.0, grid_color);
            }

            // Draw grid coordinates
            let font_size = 15.0 * scale;
            for x in 0..=grid_width {
                for y in 0..=grid_height {
                    let pos = self.grid.get_coord_from_grid(x, y);
                    let (draw_x, draw_y) = self.get_scaled_pos(pos.x, pos.y);
                    draw_text(&format!("{},{}", x, y), draw_x, draw_y, font_size, WHITE);
                }
            }
        }
    }

    fn draw(&self) {
        clear_background(BLACK);

        if let Some(current_scene) = self.current_scene() {
            self.draw_scene(current_scene);
        } else {
            self.draw_error_message("Scene not found");
        }

        self.draw_debug_grid();

        let scale = self.get_scale();
        for (index, character) in self.characters.iter().enumerate() {
            let is_active = self.active_character == Some(index);
            self.draw_character(character, scale, is_active);
        }

        self.draw_ui();

        let (text_x, text_y) = self.get_scaled_pos(20.0, 60.0);
        draw_text(
            &format!("Characters: {}", self.characters.len()),
            text_x,
            text_y,
            20.0 * self.get_scale(),
            WHITE,
        );

        if let Some(character) = self.characters.first() {
            let (text_x, text_y) = self.get_scaled_pos(20.0, 90.0);
            draw_text(
                &format!("Animation Speed: {:.2}", character.animation_speed),
                text_x,
                text_y,
                20.0 * self.get_scale(),
                WHITE,
            );
        }

        if let Some(debug_tools) = &self.debug_tools {
            if debug_tools.bounding_box_mode {
                debug_tools.draw_bounding_box_info(self);
            }
        }
    }

    fn draw_character(&self, character: &Character, scale: f32, is_active: bool) {
        let (x, y) = self.get_scaled_pos(character.position.x, character.position.y);

        let cycle = if character.animation_index < 4 { 0 } else { 7 };
        let frame = character.animation_index % 4;

        let filename = format!(
            "{}{}{}{}.png",
            character.data.name, character.direction as u8, frame, cycle
        );

        let x_offset = 0.0;
        //let y_offset = -65.0;
        let y_offset = 0.0;

        match self.character_textures.get(&filename) {
            Some(texture) => {
                // The character assets are centered, so we need to offset the x position
                // by half the width of the texture
                // The width of the textures are different, but the character is always centered
                // Then we play around with an offset to make it look better
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
                // Draw a placeholder rectangle for debugging
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

        // Debug info
        let (text_x, text_y) = self.get_scaled_pos(x, y - 20.0);
        draw_text(
            &format!("File: {}", filename),
            text_x + x_offset,
            text_y + y_offset,
            15.0 * scale,
            WHITE,
        );
    }

    fn draw_scene(&self, scene: &Scene) {
        if let Some(texture) = self.textures.get(&scene.background) {
            self.draw_background(texture);
            self.draw_clickable_areas(scene);
            self.draw_scene_description(scene);
        } else {
            self.draw_loading_message(&scene.background);
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

    fn draw_clickable_areas(&self, scene: &Scene) {
        for area in &scene.clickable_areas {
            self.draw_clickable_area(area);
            self.draw_target_scene_description(area);
        }
    }

    fn draw_clickable_area(&self, area: &ClickableArea) {
        let (x, y) = self.get_scaled_pos(area.x, area.y);
        let width = area.width * self.get_scale();
        let height = area.height * self.get_scale();
        draw_rectangle_lines(x, y, width, height, 2.0, RED);
    }

    fn draw_target_scene_description(&self, area: &ClickableArea) {
        if let Some(target_scene) = self.get_scene(area.target_scene) {
            let (x, y) = self.get_scaled_pos(area.x, area.y);
            let width = area.width * self.get_scale();
            let text = &target_scene.description;
            let font_size = 15.0 * self.get_scale();
            let text_dim = measure_text(text, None, font_size as u16, 1.0);

            let text_x = x + (width - text_dim.width) / 2.0;
            let text_y = y - text_dim.height - 5.0 * self.get_scale();

            draw_rectangle(
                text_x - 5.0 * self.get_scale(),
                text_y - 5.0 * self.get_scale(),
                text_dim.width + 10.0 * self.get_scale(),
                text_dim.height + 10.0 * self.get_scale(),
                Color::new(0.0, 0.0, 0.0, 0.5),
            );

            draw_text(text, text_x, text_y + text_dim.height, font_size, WHITE);
        }
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
