use crate::asset_manager::AssetManager;
use crate::config::{character, dialog, inventory};
use crate::{ClickableArea, Game, OverlayAsset, Scene};
use macroquad::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

struct DrawableItem<'a> {
    y_position: i32,
    item: DrawableType<'a>,
}

enum DrawableType<'a> {
    Character(usize),
    OverlayAsset(&'a OverlayAsset),
}

impl<'a> DrawableItem<'a> {
    fn new_character(index: usize, y: f32) -> Self {
        DrawableItem {
            y_position: ((y + character::HEIGHT) * 1000.0) as i32,
            item: DrawableType::Character(index),
        }
    }

    fn new_overlay(overlay: &'a OverlayAsset) -> Self {
        DrawableItem {
            y_position: ((overlay.y + overlay.height as f32) * 1000.0) as i32,
            item: DrawableType::OverlayAsset(overlay),
        }
    }
}

impl<'a> Ord for DrawableItem<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for max-heap behavior
        other.y_position.cmp(&self.y_position)
    }
}

impl<'a> PartialOrd for DrawableItem<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Eq for DrawableItem<'a> {}

impl<'a> PartialEq for DrawableItem<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.y_position == other.y_position
    }
}

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
        self.draw_dialog_menu(game, asset_manager);
        self.draw_debug(game);
        self.draw_ui(game, asset_manager);
    }

    fn draw_scene(&self, game: &Game, scene: &Scene, asset_manager: &AssetManager) {
        let Some(texture) = asset_manager.get_texture(&scene.background) else {
            self.draw_loading_message(&scene.background);
            return;
        };

        self.draw_background(&texture);
        self.draw_world_items(game, asset_manager);
        let scale = self.get_scale();

        let mut heap = BinaryHeap::new();
        let mut top_overlays = Vec::new();

        for (i, pos) in game.characters.positions.iter().enumerate() {
            heap.push(DrawableItem::new_character(i, pos.y));
        }

        for overlay in &scene.overlay_assets {
            match overlay.z_value {
                0 => self.draw_overlay_asset(overlay, asset_manager),
                4 => top_overlays.push(overlay),
                _ => heap.push(DrawableItem::new_overlay(overlay)),
            }
        }

        // Draw items in correct z-order
        while let Some(item) = heap.pop() {
            match item.item {
                DrawableType::Character(index) => {
                    self.draw_character(
                        game,
                        index,
                        scale,
                        game.active_character == Some(index),
                        asset_manager,
                    );
                }
                DrawableType::OverlayAsset(overlay) => {
                    self.draw_overlay_asset(overlay, asset_manager);
                }
            }
        }

        // Draw overlays with z_value=4 last
        for overlay in top_overlays {
            self.draw_overlay_asset(overlay, asset_manager);
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
        let texture_path = format!("Huvudmeny/Gubbar/{}", filename);

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
        let button_texture_path = format!(
            "Huvudmeny/inventory/väska{}.png",
            game.inventory.animation_frame + 1
        );
        if let Some(texture) = asset_manager.get_texture(&button_texture_path) {
            let scale = self.get_scale();

            let scaled_width = texture.width() * scale;
            let scaled_height = texture.height() * scale;

            let game_x = 1920.0 - texture.width();
            let game_y = 1440.0 - texture.height();

            let (screen_x, screen_y) = self.get_scaled_pos(game_x, game_y);

            let animation_progress = game.inventory.animation_frame as f32 / 12.0;

            // Draw the background texture
            draw_texture_ex(
                &texture,
                screen_x,
                screen_y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(scaled_width, scaled_height)),
                    ..Default::default()
                },
            );

            if game.inventory.animation_frame > 6 {
                let visible_width = animation_progress * texture.width();
                let slots_start_x = game_x + inventory::START_X;

                for i in 0..inventory::SLOT_COUNT {
                    let item_index = i + game.inventory.scroll_offset;
                    let slot = if item_index < inventory::INVENTORY_SIZE {
                        game.inventory.items[item_index]
                    } else {
                        None
                    };

                    let slot_x =
                        slots_start_x + (inventory::SLOT_SIZE + inventory::SLOT_SPACING) * i as f32;
                    let slot_y = game_y + (texture.height() - inventory::SLOT_SIZE) / 2.0;
                    let (screen_x, screen_y) = self.get_scaled_pos(slot_x, slot_y);
                    let scaled_slot_size = inventory::SLOT_SIZE * scale;

                    let slot_visible_width = (visible_width - (slot_x - game_x))
                        .max(0.0)
                        .min(inventory::SLOT_SIZE);

                    if slot_visible_width > 0.0 {
                        let slot_color = if Some(i) == game.inventory.hovered_slot {
                            BLUE
                        } else {
                            GREEN
                        };

                        draw_rectangle_lines(
                            screen_x,
                            screen_y,
                            scaled_slot_size * (slot_visible_width / inventory::SLOT_SIZE),
                            scaled_slot_size,
                            2.0,
                            slot_color,
                        );

                        // Draw item in slot if it exists
                        if let Some(item_id) = slot {
                            if let Some(item) = game.items.iter().find(|i| i.id == item_id) {
                                if let Some(mut item_texture) =
                                    asset_manager.get_texture(&item.textures.in_inventory)
                                {
                                    if let Some(item_texture_text) =
                                        asset_manager.get_texture(&item.textures.in_inventory_text)
                                    {
                                        item_texture = if Some(i) == game.inventory.hovered_slot {
                                            item_texture_text
                                        } else {
                                            item_texture
                                        };
                                    }

                                    // text asset is wider than the item asset
                                    // TODO: clean up as this does nothing when
                                    // item_texture is changed above
                                    // still works though
                                    let max_width = item_texture.width().max(
                                        asset_manager
                                            .get_texture(&item.textures.in_inventory_text)
                                            .map_or(0.0, |t| t.width()),
                                    );
                                    let max_height = item_texture.height();

                                    // Calculate scaling factors
                                    let scale_x = inventory::SLOT_SIZE / max_width;
                                    let scale_y = inventory::SLOT_SIZE / max_height;
                                    let item_scale = scale_x.min(scale_y);

                                    let scaled_item_width =
                                        item_texture.width() * item_scale * scale;
                                    let scaled_item_height =
                                        item_texture.height() * item_scale * scale;

                                    // Center the item in the slot
                                    let item_x =
                                        screen_x + (scaled_slot_size - scaled_item_width) / 2.0;
                                    let item_y =
                                        screen_y + (scaled_slot_size - scaled_item_height) / 2.0;

                                    // Calculate the visible portion of the item
                                    let visible_item_width = (slot_visible_width
                                        / inventory::SLOT_SIZE
                                        * scaled_item_width)
                                        .min(scaled_item_width);

                                    // Adjust the source rectangle to account for the item's original dimensions
                                    let source_rect = Rect::new(
                                        0.0,
                                        0.0,
                                        visible_item_width / (item_scale * scale),
                                        item_texture.height(),
                                    );

                                    draw_texture_ex(
                                        item_texture,
                                        item_x,
                                        item_y,
                                        WHITE,
                                        DrawTextureParams {
                                            dest_size: Some(Vec2::new(
                                                visible_item_width,
                                                scaled_item_height,
                                            )),
                                            source: Some(source_rect),
                                            ..Default::default()
                                        },
                                    );
                                }
                            }
                        }
                    }
                }

                if game.inventory.animation_frame > 11 {
                    // Draw arrow buttons
                    self.draw_inventory_arrow(game, asset_manager, true); // Left arrow
                    self.draw_inventory_arrow(game, asset_manager, false); // Right arrow
                }
            }
        } else {
            println!(
                "Inventory button texture not found: {}",
                button_texture_path
            );
        }
    }

    fn draw_inventory_arrow(&self, game: &Game, asset_manager: &AssetManager, is_left: bool) {
        let arrow_rect = if is_left {
            game.inventory.left_arrow_rect
        } else {
            game.inventory.right_arrow_rect
        };

        let texture_path = if is_left {
            "Huvudmeny/inventory/pilv-271.png"
        } else {
            "Huvudmeny/inventory/pilh-272.png"
        };

        if let Some(texture) = asset_manager.get_texture(texture_path) {
            let (x, y) = self.get_scaled_pos(arrow_rect.x, arrow_rect.y);
            let scale = self.get_scale();

            // Draw the arrow texture
            draw_texture_ex(
                texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(arrow_rect.w * scale, arrow_rect.h * scale)),
                    ..Default::default()
                },
            );

            // Draw the border
            let border_color = if (is_left && game.inventory.hovered_left_arrow)
                || (!is_left && game.inventory.hovered_right_arrow)
            {
                BLUE
            } else {
                GREEN
            };

            draw_rectangle_lines(
                x,
                y,
                arrow_rect.w * scale,
                arrow_rect.h * scale,
                2.0,
                border_color,
            );
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
            format!("#{} - {} - {}", scene.id, scene.name, scene.description).as_str(),
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
            if game.debug_level_switch_mode {
                self.draw_level_list(game);
            }
            self.draw_debug_info(game);
            self.draw_dialog_boxes(game);
        }
    }

    fn draw_level_list(&self, game: &Game) {
        let (text_x, text_y) = self.get_scaled_pos(20.0, 200.0);
        let font_size = 35.0 * self.get_scale();
        let line_height = font_size * 0.8;
        let mut y = text_y;

        for (i, level) in game.levels.iter().enumerate() {
            let text = format!("{} - {}", i, level.name);
            draw_text(&text, text_x, y, font_size, WHITE);
            y += line_height;
        }
    }

    fn draw_debug_info(&self, game: &Game) {
        self.draw_scene_description(&game.scenes.data[game.current_scene as usize]);
        let (text_x, text_y) = self.get_scaled_pos(20.0, 60.0);
        draw_text(
            &format!("Characters: {}", game.characters.count),
            text_x,
            text_y,
            20.0 * self.get_scale(),
            WHITE,
        );

        if game.characters.count > 0 {
            let pos = format!(
                "Position: ({:.2}, {:.2})",
                game.characters.positions[0].x, game.characters.positions[0].y
            );
            let anim_speed = format!(
                "Animation Speed: {:.2}",
                game.characters.animation_speeds[0]
            );
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

    fn draw_dialog_boxes(&self, game: &Game) {
        if let Some(current_scene) = game.get_current_scene() {
            for dialog in &current_scene.dialogs {
                let (x, y) = self.get_scaled_pos(dialog.x, dialog.y);
                let width = dialog.width * self.get_scale();
                let height = dialog.height * self.get_scale();

                draw_rectangle_lines(x, y, width, height, 2.0, MAGENTA);
                let text = format!("#{}", dialog.description);
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

    fn draw_dialog_menu(&self, game: &Game, asset_manager: &AssetManager) {
        if game.dialog_menu.open {
            if let Some(dialog_background) =
                asset_manager.get_texture(&game.ui.general_textures.dialog_background)
            {
                let scale = self.get_scale();

                // Draw dialog background
                let (scaled_x, scaled_y) = self.get_scaled_pos(0.0, dialog::START_Y);
                let scaled_width = dialog::WIDTH * scale;
                let scaled_height = dialog::HEIGHT * scale;

                draw_texture_ex(
                    dialog_background,
                    scaled_x,
                    scaled_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(scaled_width, scaled_height)),
                        ..Default::default()
                    },
                );

                if let Some(dialog_id) = game.dialog_menu.current_dialog_id {
                    if let Some(current_scene) = game.get_current_scene() {
                        if let Some(dialog) =
                            current_scene.dialogs.iter().find(|d| d.id == dialog_id)
                        {
                            if let Some(level) = dialog.tree.get(game.dialog_menu.current_level) {
                                // Get the font outside the loop
                                let dialog_font = asset_manager.get_font("dialog");

                                for (i, option) in level.options.iter().enumerate() {
                                    let option_x = dialog::OPTION_START_X * scale + scaled_x;
                                    let option_y = (dialog::OPTION_START_Y
                                        + i as f32 * dialog::OPTION_SPACING)
                                        * scale
                                        + scaled_y;
                                    let option_width = dialog::OPTION_BOX_WIDTH * scale;
                                    let option_height = dialog::OPTION_BOX_HEIGHT * scale;

                                    let is_hovered = game.dialog_menu.hovered_option == Some(i);
                                    let (box_color, text_color) = if is_hovered {
                                        (
                                            dialog::OPTION_HOVER_BOX_COLOR,
                                            dialog::OPTION_HOVER_TEXT_COLOR,
                                        )
                                    } else {
                                        (dialog::OPTION_BOX_COLOR, dialog::OPTION_TEXT_COLOR)
                                    };

                                    if game.debug_tools.active {
                                        draw_rectangle_lines(
                                            option_x,
                                            option_y,
                                            option_width,
                                            option_height,
                                            2.0,
                                            box_color,
                                        );
                                    }
                                    // Draw option text with custom font
                                    let font_size = dialog::FONT_SIZE * scale;
                                    let text_params = TextParams {
                                        font: dialog_font,
                                        font_size: font_size as u16,
                                        color: text_color,
                                        ..Default::default()
                                    };

                                    draw_text_ex(
                                        &option.text,
                                        option_x + dialog::TEXT_PADDING_X * scale,
                                        option_y + option_height / 2.0 + font_size / 2.0,
                                        text_params,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
