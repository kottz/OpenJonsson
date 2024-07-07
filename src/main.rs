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

struct Game {
    textures: HashMap<String, Texture2D>,
    current_level: u32,
    current_scene: u32,
    game_data: GameData,
    loading_textures: HashSet<String>,
    window_size: Vec2,
    game_rect: Rect,
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
        };

        game.load_current_and_adjacent_scenes().await;

        Ok(game)
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

        if let Some(current_level) = self
            .game_data
            .levels
            .iter()
            .find(|l| l.id == self.current_level)
        {
            if let Some(current_scene) = current_level
                .scenes
                .iter()
                .find(|s| s.id == self.current_scene)
            {
                backgrounds_to_load.push(current_scene.background.clone());

                for area in &current_scene.clickable_areas {
                    if let Some(target_scene) = current_level
                        .scenes
                        .iter()
                        .find(|s| s.id == area.target_scene)
                    {
                        backgrounds_to_load.push(target_scene.background.clone());
                    }
                }
            }
        }

        self.load_textures(backgrounds_to_load).await;
    }

    async fn update(&mut self) {
        let new_window_size = Vec2::new(screen_width(), screen_height());
        if new_window_size != self.window_size {
            self.window_size = new_window_size;
            self.game_rect = Game::calculate_game_rect(self.window_size);
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            let scale = self.get_scale();

            // Translate mouse position to game coordinates
            let game_x = (mouse_x - self.game_rect.x) / scale;
            let game_y = (mouse_y - self.game_rect.y) / scale;

            if let Some(current_level) = self
                .game_data
                .levels
                .iter()
                .find(|l| l.id == self.current_level)
            {
                if let Some(current_scene) = current_level
                    .scenes
                    .iter()
                    .find(|s| s.id == self.current_scene)
                {
                    for area in &current_scene.clickable_areas {
                        if game_x >= area.x
                            && game_x <= area.x + area.width
                            && game_y >= area.y
                            && game_y <= area.y + area.height
                        {
                            self.current_scene = area.target_scene;
                            self.load_current_and_adjacent_scenes().await;
                            break;
                        }
                    }
                }
            }
        }
    }

    fn draw(&self) {
        clear_background(BLACK);

        if let Some(current_level) = self
            .game_data
            .levels
            .iter()
            .find(|l| l.id == self.current_level)
        {
            if let Some(current_scene) = current_level
                .scenes
                .iter()
                .find(|s| s.id == self.current_scene)
            {
                if let Some(texture) = self.textures.get(&current_scene.background) {
                    // Draw the background texture
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

                    // Debug: Draw clickable areas and target scene descriptions
                    for area in &current_scene.clickable_areas {
                        let (x, y) = self.get_scaled_pos(area.x, area.y);
                        let width = area.width * self.get_scale();
                        let height = area.height * self.get_scale();

                        draw_rectangle_lines(x, y, width, height, 2.0, RED);

                        // Find and draw the target scene description
                        if let Some(target_scene) = current_level
                            .scenes
                            .iter()
                            .find(|s| s.id == area.target_scene)
                        {
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

                    // Draw current scene description in red at the top-left corner
                    let (desc_x, desc_y) = self.get_scaled_pos(20.0, 20.0);
                    draw_text(
                        &current_scene.description,
                        desc_x,
                        desc_y,
                        30.0 * self.get_scale(),
                        RED,
                    );
                } else {
                    let (text_x, text_y) = self.get_scaled_pos(20.0, 20.0);
                    draw_text(
                        &format!("Loading texture: {}", current_scene.background),
                        text_x,
                        text_y,
                        30.0 * self.get_scale(),
                        YELLOW,
                    );
                }
            } else {
                let (text_x, text_y) = self.get_scaled_pos(20.0, 20.0);
                draw_text(
                    &format!("Scene not found: {}", self.current_scene),
                    text_x,
                    text_y,
                    30.0 * self.get_scale(),
                    RED,
                );
            }
        } else {
            let (text_x, text_y) = self.get_scaled_pos(20.0, 20.0);
            draw_text(
                &format!("Level not found: {}", self.current_level),
                text_x,
                text_y,
                30.0 * self.get_scale(),
                RED,
            );
        }
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
