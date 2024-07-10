use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

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

#[derive(Deserialize, Debug, Clone)]
struct GameData {
    levels: Vec<Level>,
}

struct DebugTools {
    bounding_box_mode: bool,
    bounding_box_start: Option<Vec2>,
    current_bounding_box: Option<Rect>,
}

impl DebugTools {
    fn new() -> Self {
        DebugTools {
            bounding_box_mode: false,
            bounding_box_start: None,
            current_bounding_box: None,
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
            // Second click, create the bounding box
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
            // First click, set the starting point
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
}

impl Game {
    async fn new() -> Result<Self, String> {
        let json_path = Path::new("static/level_data.json");
        let json = std::fs::read_to_string(json_path)
            .map_err(|e| format!("Failed to read JSON file: {}", e))?;
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
        };

        game.load_current_and_adjacent_scenes().await;

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
            // Window is wider, fit to height
            let height = window_size.y;
            let width = height * game_aspect;
            let x = (window_size.x - width) / 2.0;
            Rect::new(x, 0.0, width, height)
        } else {
            // Window is taller, fit to width
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

    fn get_game_coordinates(&self, (mouse_x, mouse_y): (f32, f32)) -> (f32, f32) {
        let scale = self.get_scale();
        (
            (mouse_x - self.game_rect.x) / scale,
            (mouse_y - self.game_rect.y) / scale,
        )
    }

    async fn load_texture(&mut self, bg: &str) -> Result<(), String> {
        if self.textures.contains_key(bg) || self.loading_textures.contains(bg) {
            return Ok(());
        }

        self.loading_textures.insert(bg.to_string());
        let texture_path = Path::new("static/resources").join(bg);
        match load_texture(texture_path.to_str().unwrap()).await {
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

    async fn update(&mut self) {
        self.update_window_size();

        if is_key_pressed(KeyCode::B) {
            if let Some(debug_tools) = &mut self.debug_tools {
                debug_tools.toggle_bounding_box_mode();
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let game_coordinates = self.get_game_coordinates(mouse_position());

            if let Some(debug_tools) = &mut self.debug_tools {
                if debug_tools.bounding_box_mode {
                    debug_tools.handle_bounding_box_creation(game_coordinates);
                } else {
                    self.handle_mouse_click(game_coordinates).await;
                }
            } else {
                self.handle_mouse_click(game_coordinates).await;
            }
        }
    }

    fn update_window_size(&mut self) {
        let new_window_size = Vec2::new(screen_width(), screen_height());
        if new_window_size != self.window_size {
            self.window_size = new_window_size;
            self.game_rect = Game::calculate_game_rect(self.window_size);
        }
    }

    async fn handle_mouse_click(&mut self, (game_x, game_y): (f32, f32)) {
        if let Some(area) = self.find_clicked_area(game_x, game_y) {
            self.current_scene = area.target_scene;
            self.load_current_and_adjacent_scenes().await;
        }
    }

    fn find_clicked_area(&self, game_x: f32, game_y: f32) -> Option<&ClickableArea> {
        self.current_scene().and_then(|scene| {
            scene.clickable_areas.iter().find(|area| {
                game_x >= area.x
                    && game_x <= area.x + area.width
                    && game_y >= area.y
                    && game_y <= area.y + area.height
            })
        })
    }

    fn draw(&self) {
        clear_background(BLACK);

        if let Some(current_scene) = self.current_scene() {
            self.draw_scene(current_scene);
        } else {
            self.draw_error_message("Scene not found");
        }

        if let Some(debug_tools) = &self.debug_tools {
            if debug_tools.bounding_box_mode {
                debug_tools.draw_bounding_box_info(self);
            }
        }
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

#[macroquad::main("Point and Click Adventure")]
async fn main() {
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
