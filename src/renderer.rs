use crate::asset_manager::AssetManager;
use crate::config::character;
use crate::{ClickableArea, Game, OverlayAsset, Scene};
use macroquad::prelude::*;

pub struct Renderer {
    window_size: Vec2,
    game_rect: Rect,
}

impl Renderer {
    pub fn new(window_size: Vec2) -> Self {
        let game_rect = Self::calculate_game_rect(window_size);
        Self {
            window_size,
            game_rect,
        }
    }

    pub fn update_window_size(&mut self, window_size: Vec2) {
        self.window_size = window_size;
        self.game_rect = Self::calculate_game_rect(self.window_size);
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

    pub fn get_scale(&self) -> f32 {
        self.game_rect.w / 1920.0
    }

    pub fn get_scaled_pos(&self, x: f32, y: f32) -> (f32, f32) {
        let scale = self.get_scale();
        (self.game_rect.x + x * scale, self.game_rect.y + y * scale)
    }

    pub fn get_game_coordinates(&self, mouse_pos: Vec2) -> Vec2 {
        let scale = self.get_scale();
        Vec2::new(
            (mouse_pos.x - self.game_rect.x) / scale,
            (mouse_pos.y - self.game_rect.y) / scale,
        )
    }

    pub fn is_in_game_area(&self, game_pos: Vec2) -> bool {
        self.game_rect
            .contains(self.get_scaled_pos(game_pos.x, game_pos.y).into())
    }

    pub fn draw(&self, game: &Game, asset_manager: &AssetManager) {
        clear_background(BLACK);

        if let Some(current_scene) = game.get_current_scene() {
            self.draw_scene(game, current_scene, asset_manager);
        } else {
            self.draw_error_message("Scene not found");
        }

        self.draw_inventory(game, asset_manager);
        self.draw_debug(game);
        self.draw_ui(game, asset_manager);
    }

    fn draw_scene(&self, game: &Game, scene: &Scene, asset_manager: &AssetManager) {
        if let Some(texture) = asset_manager.get_texture(&scene.background) {
            self.draw_background(&texture);

            self.draw_world_items(game, asset_manager);

            let scale = self.get_scale();

            for i in 0..game.characters.count {
                let is_active = game.active_character == Some(i);
                self.draw_character(game, i, scale, is_active, asset_manager);
            }

            for overlay in &scene.overlay_assets {
                self.draw_overlay_asset(overlay, asset_manager);
            }

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

    fn draw_character(
        &self,
        game: &Game,
        index: usize,
        scale: f32,
        is_active: bool,
        asset_manager: &AssetManager,
    ) {
        // In order for characters to line up on the grid
        // we need to offset them up.
        let x_offset = character::X_OFFSET * scale;
        let y_offset = character::Y_OFFSET * scale;

        let (x, y) = self.get_scaled_pos(
            game.characters.positions[index].x,
            game.characters.positions[index].y,
        );

        let cycle = if game.characters.animation_indices[index] < 4 {
            0
        } else {
            7
        };
        let frame = game.characters.animation_indices[index] % 4;

        let filename = format!(
            "{}{}{}{}.png",
            game.characters.data[index].name, game.characters.directions[index] as u8, frame, cycle
        );
        let texture_path = format!("berlin/Gubbar/{}", filename);

        if let Some(texture) = asset_manager.get_texture(&texture_path) {
            let xt = texture.width() / 2.0 * scale;
            let yt = texture.height() / 2.0 * scale;
            draw_texture_ex(
                &texture,
                (x + x_offset) - xt,
                (y + y_offset) - yt,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(texture.width() * scale, texture.height() * scale)),
                    ..Default::default()
                },
            );
        } else {
            println!("Texture not found for filename: {}", filename);
            let rect_size = 50.0 * scale;
            draw_rectangle(
                (x + x_offset) - rect_size / 2.0,
                (y + y_offset) - rect_size / 2.0,
                rect_size,
                rect_size,
                RED,
            );
        }

        if is_active {
            let indicator_size = 10.0 * scale;
            draw_circle(
                x + x_offset,
                y + y_offset - 40.0 * scale,
                indicator_size,
                GREEN,
            );
        }
    }

    fn draw_overlay_asset(&self, overlay: &OverlayAsset, asset_manager: &AssetManager) {
        if let Some(texture) = asset_manager.get_texture(&overlay.texture_path) {
            let (ox, oy) = (overlay.x * 3.0, overlay.y * 3.0);
            let (x, y) = self.get_scaled_pos(ox, oy);
            let scale = self.get_scale();
            draw_texture_ex(
                &texture,
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

    fn draw_world_items(&self, game: &Game, asset_manager: &AssetManager) {
        let current_scene_items = &game.world_items[game.current_scene as usize];
        let mouse_pos = Vec2::from(mouse_position());
        let game_pos = self.get_game_coordinates(mouse_pos);

        for item_instance in current_scene_items {
            let item = game
                .items
                .iter()
                .find(|i| i.id == item_instance.item_id)
                .unwrap();
            let texture_path = if game.is_mouse_over_item(game_pos, item_instance) {
                &item.textures.mouse_over
            } else {
                &item.textures.in_world
            };

            if let Some(texture) = asset_manager.get_texture(texture_path) {
                let (x, y) = self.get_scaled_pos(item_instance.x, item_instance.y);
                let scale = self.get_scale();
                draw_texture_ex(
                    &texture,
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

    fn draw_inventory(&self, game: &Game, asset_manager: &AssetManager) {
        let inventory_start_x = 20.0;
        let inventory_start_y = self.game_rect.h - 100.0;
        let item_size = 60.0;
        let item_spacing = 10.0;

        for (index, item_id) in game.inventory.iter().enumerate() {
            if let Some(item) = game.items.iter().find(|i| i.id == *item_id) {
                if let Some(texture) = asset_manager.get_texture(&item.textures.in_inventory) {
                    let x = inventory_start_x + (item_size + item_spacing) * index as f32;
                    let (draw_x, draw_y) = self.get_scaled_pos(x, inventory_start_y);
                    let scale = self.get_scale();
                    draw_texture_ex(
                        &texture,
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

    fn draw_ui(&self, game: &Game, asset_manager: &AssetManager) {
        for menu_item in &game.ui.menu_items {
            if let Some(texture) = asset_manager.get_texture(&menu_item.texture) {
                let (x, y) = self.get_scaled_pos(menu_item.position[0], menu_item.position[1]);
                let scale = self.get_scale();
                draw_texture_ex(
                    &texture,
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

        if let Some(cursor_texture) = asset_manager.get_texture(
            &game
                .ui
                .cursors
                .iter()
                .find(|c| c.cursor_type == game.current_cursor)
                .map(|c| &c.texture)
                .unwrap_or(&String::new()),
        ) {
            let cursor_pos = mouse_position();
            if let Some(cursor) = game
                .ui
                .cursors
                .iter()
                .find(|c| c.cursor_type == game.current_cursor)
            {
                let scale = self.get_scale();
                draw_texture_ex(
                    &cursor_texture,
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

    fn draw_debug(&self, game: &Game) {
        if game.debug_tools.active {
            if game.debug_tools.draw_grid {
                self.draw_debug_grid(game);
            }
            self.draw_debug_info(game);
        }
    }

    fn draw_debug_info(&self, game: &Game) {
        let (text_x, text_y) = self.get_scaled_pos(20.0, 60.0);
        draw_text(
            &format!("Characters: {}", game.characters.count),
            text_x,
            text_y,
            20.0 * self.get_scale(),
            WHITE,
        );

        if game.characters.count > 0 {
            let pos = format!("Position: ({:.2}, {:.2})", game.characters.positions[0].x, game.characters.positions[0].y);
            let anim_speed = format!("Animation Speed: {:.2}", game.characters.animation_speeds[0]);
            for (i, text) in [pos, anim_speed].iter().enumerate() {
                let (x, y) = self.get_scaled_pos(20.0, 90.0 + 30.0 * i as f32);
                draw_text(text, x, y, 20.0 * self.get_scale(), WHITE);
            }
        }

        if game.debug_tools.bounding_box_mode {
            self.draw_bounding_box_info(game);
        }

        self.draw_scene_transitions(game);
    }

    fn draw_scene_transitions(&self, game: &Game) {
        if let Some(current_scene) = game.get_current_scene() {
            for transition in &current_scene.scene_transitions {
                let (x, y) = self.get_scaled_pos(transition.x, transition.y);
                let width = transition.width * self.get_scale();
                let height = transition.height * self.get_scale();

                draw_rectangle_lines(x, y, width, height, 2.0, BLUE);
                let text = format!("#{}", transition.target_scene);
                draw_text(&text, x, y, 40.0 * self.get_scale(), WHITE);
            }
        }
    }

    fn draw_debug_grid(&self, game: &Game) {
        let grid_color = Color::new(0.0, 1.0, 0.0, 0.5);
        let scale = self.get_scale();

        let grid_width = 41;
        let grid_height = 41;

        let x_delta =
            (game.grid.get_coord_from_grid(1, 0) - game.grid.get_coord_from_grid(0, 0)) / 2.0;
        for x in 0..=grid_width {
            let start = game.grid.get_coord_from_grid(x, 0) - x_delta;
            let end = game.grid.get_coord_from_grid(x, grid_height) - x_delta;
            let start = self.get_scaled_pos(start.x, start.y);
            let end = self.get_scaled_pos(end.x, end.y);
            draw_line(start.0, start.1, end.0, end.1, 2.0, grid_color);
        }

        let y_delta =
            (game.grid.get_coord_from_grid(0, 1) - game.grid.get_coord_from_grid(0, 0)) / 2.0;
        for y in 0..=grid_height {
            let start = game.grid.get_coord_from_grid(0, y) - y_delta;
            let end = game.grid.get_coord_from_grid(grid_width, y) - y_delta;
            let start = self.get_scaled_pos(start.x, start.y);
            let end = self.get_scaled_pos(end.x, end.y);
            draw_line(start.0, start.1, end.0, end.1, 2.0, grid_color);
        }

        let font_size = 20.0 * scale;
        let (x_offset, y_offset) = (0.0, 0.0);
        for x in 0..=grid_width {
            for y in 0..=grid_height {
                let pos = game.grid.get_coord_from_grid(x, y);
                let (draw_x, draw_y) = self.get_scaled_pos(pos.x, pos.y);

                // Draw black circle for blocked nodes
                if game.grid.blocked_nodes.contains(&(x, y)) {
                    let circle_radius = 5.0 * scale;
                    draw_circle(draw_x, draw_y, circle_radius, BLACK);
                }

                let color = if game.grid.is_node_walkable((x, y)) {
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

    fn draw_bounding_box_info(&self, game: &Game) {
        let (text_x, text_y) = self.get_scaled_pos(20.0, self.game_rect.h - 40.0);
        draw_text(
            "Bounding Box Mode: ON",
            text_x,
            text_y,
            20.0 * self.get_scale(),
            GREEN,
        );

        if let Some(rect) = game.debug_tools.current_bounding_box {
            let (x, y) = self.get_scaled_pos(rect.x, rect.y);
            let width = rect.w * self.get_scale();
            let height = rect.h * self.get_scale();
            draw_rectangle_lines(x, y, width, height, 2.0, GREEN);
        }
    }
}
