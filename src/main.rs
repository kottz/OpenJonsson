mod asset_manager;
mod audio;
mod config;
mod dialog;
mod grid;
mod renderer;

use crate::config::{character, inventory};
use crate::dialog::{Dialog, DialogMenu};
use crate::grid::Grid;
use asset_manager::AssetManager;
use audio::{AudioCategory, AudioSystem};
use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use macroquad::time::get_fps;
use renderer::Renderer;
use serde::Deserialize;
use std::collections::HashMap;

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
    pub name: String,
    pub description: String,
    pub background: String,
    #[serde(rename = "sceneTransitions")]
    pub scene_transitions: Vec<SceneTransition>,
    pub overlay_assets: Vec<OverlayAsset>,
    pub items: Vec<ItemInstance>,
    #[serde(skip)]
    pub blocked_nodes: Vec<(i32, i32)>,
    pub dialogs: Vec<Dialog>,
    pub background_music: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterData {
    pub name: String,
    pub speed: f32,
    pub run_speed: f32,
    pub select_audio: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OverlayAsset {
    pub texture_path: String,
    pub x: f32,
    pub y: f32,
    pub height: usize,
    pub width: usize,
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
    pub in_inventory_text: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub textures: ItemTextures,
    pub allowed_characters: Vec<String>,
    pub pickup_audio: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ItemInstance {
    pub item_id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug)]
pub struct InventoryData {
    pub open: bool,
    pub animation_frame: usize,
    pub animation_timer: f32,
    pub button_rect: Rect,
    pub items: Vec<Option<u32>>,
    pub scroll_offset: usize,
    pub hovered_slot: Option<usize>,
    pub left_arrow_rect: Rect,
    pub right_arrow_rect: Rect,
    pub hovered_left_arrow: bool,
    pub hovered_right_arrow: bool,
}

impl InventoryData {
    pub fn new() -> Self {
        let inventory_width = inventory::SLOT_SIZE * inventory::SLOT_COUNT as f32
            + inventory::SLOT_SPACING * (inventory::SLOT_COUNT - 1) as f32;

        let left_arrow_x = inventory::START_X + inventory::LEFT_ARROW_OFFSET_X;
        let right_arrow_x = inventory::START_X + inventory_width + inventory::RIGHT_ARROW_OFFSET_X;

        InventoryData {
            open: false,
            animation_frame: 0,
            animation_timer: 0.0,
            button_rect: Rect::new(1800.0, 1340.0, 100.0, 100.0),
            items: vec![None; inventory::INVENTORY_SIZE],
            scroll_offset: 0,
            hovered_slot: None,
            left_arrow_rect: Rect::new(
                left_arrow_x,
                inventory::START_Y + inventory::ARROW_OFFSET_Y,
                inventory::ARROW_SIZE,
                inventory::ARROW_SIZE,
            ),
            right_arrow_rect: Rect::new(
                right_arrow_x,
                inventory::START_Y + inventory::ARROW_OFFSET_Y,
                inventory::ARROW_SIZE,
                inventory::ARROW_SIZE,
            ),
            hovered_left_arrow: false,
            hovered_right_arrow: false,
        }
    }
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
    #[serde(rename = "generalTextures")]
    pub general_textures: GeneralTextures,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeneralTextures {
    #[serde(rename = "dialogBackground")]
    pub dialog_background: String,
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
    last_click_times: Vec<f64>,
    is_running: Vec<bool>,
}

struct Scenes {
    data: Vec<Scene>,
    count: usize,
}

struct Game {
    characters: Characters,
    levels: Vec<Level>,
    scenes: Scenes,
    current_level: u32,
    current_scene: u32,
    window_size: Vec2,
    active_character: Option<usize>,
    grid: Grid,
    current_cursor: CursorType,
    ui: UI,
    debug_tools: DebugTools,
    debug_instant_move: bool,
    debug_level_switch_mode: bool,
    items: Vec<Item>,
    world_items: Vec<Vec<ItemInstance>>,
    renderer: Renderer,
    asset_manager: AssetManager,
    inventory: InventoryData,
    dialog_menu: DialogMenu,
    audio_system: AudioSystem,
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
            last_click_times: vec![0.0; game_data.characters.len()],
            is_running: vec![false; game_data.characters.len()],
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
        let renderer = Renderer::new(window_size);
        let asset_manager = AssetManager::new();

        let mut game = Game {
            characters,
            levels: game_data.levels,
            scenes,
            current_level: 0,
            current_scene: 0,
            window_size,
            active_character: Some(0),
            grid: Grid::new(),
            current_cursor: CursorType::Normal,
            ui: game_data.ui,
            debug_tools: DebugTools::new(),
            debug_instant_move: false,
            debug_level_switch_mode: false,
            items: game_data.items,
            world_items: Vec::new(),
            renderer,
            asset_manager,
            inventory: InventoryData::new(),
            dialog_menu: DialogMenu::new(),
            audio_system: AudioSystem::new(),
        };

        game.load_level_scenes(game.current_level);
        game.load_audio_assets().await?;
        game.load_current_and_adjacent_scenes().await;
        game.load_characters().await;
        game.load_debug_textures().await;
        game.load_ui_textures().await;
        game.load_fonts().await?;
        game.load_inventory_textures().await;
        game.load_item_textures().await;

        Ok(game)
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
        self.asset_manager.load_textures(&textures_to_load).await;
    }

    async fn load_audio_assets(&mut self) -> Result<(), String> {
        let mut audio_files = std::collections::HashSet::new();
        for level in &self.levels {
            for scene in &level.scenes {
                if let Some(music) = &scene.background_music {
                    audio_files.insert(music.clone());
                }
                // Add dialog audio files if needed
                for dialog in &scene.dialogs {
                    if let Some(open_audio) = &dialog.open_audio {
                        let audio_path =
                            format!("voice/{}/{}_{}.wav", scene.name, scene.name, open_audio);
                        audio_files.insert(audio_path);
                    }
                    for level in &dialog.tree {
                        for option in &level.options {
                            for audio in &option.response_audio {
                                let audio_path =
                                    format!("voice/{}/{}_{}.wav", scene.name, scene.name, audio);
                                audio_files.insert(audio_path);
                            }
                        }
                    }
                }
            }
        }

        // Add item pickup audio files
        for item in &self.items {
            for audio_list in item.pickup_audio.values() {
                for audio_file in audio_list {
                    audio_files.insert(audio_file.clone());
                }
            }
        }

        // Add character audio files
        for c in &self.characters.data {
            for audio_file in &c.select_audio {
                audio_files.insert(audio_file.clone());
            }
        }

        for audio_file in audio_files {
            self.asset_manager.load_sound(&audio_file).await?;
        }

        // UI sounds
        self.asset_manager
            .load_sound("Huvudmeny/ljudfx/oppna.wav")
            .await?;
        self.asset_manager
            .load_sound("Huvudmeny/ljudfx/stanga.wav")
            .await?;
        Ok(())
    }

    // Update this method to work with the new AudioSystem
    fn update_scene_audio(&mut self) {
        let music_to_play = self
            .get_current_scene()
            .and_then(|scene| scene.background_music.clone());

        match music_to_play {
            Some(music) => {
                // Check if the music is already playing
                if self
                    .audio_system
                    .currently_playing
                    .get(&AudioCategory::Music)
                    != Some(&Some(music.clone()))
                {
                    self.audio_system.play_music(&self.asset_manager, &music);
                }
            }
            None => {
                // Stop the music if there's no background music for this scene
                self.audio_system.stop_music(&self.asset_manager);
            }
        }
    }

    async fn load_fonts(&mut self) -> Result<(), String> {
        self.asset_manager
            .load_font("dialog", "static/fonts/LiberationSans-Regular.ttf")
            .await?;
        Ok(())
    }

    async fn load_characters(&mut self) {
        for index in 0..self.characters.count {
            let character_data = &self.characters.data[index];
            for dir in 1..=8 {
                for frame in 0..=7 {
                    for state in [0, 7] {
                        let filename =
                            format!("{}{}{}{}.png", character_data.name, dir, frame, state);
                        let path = format!("Huvudmeny/Gubbar/{}", filename);
                        if let Err(e) = self.asset_manager.load_texture(&path).await {
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }
    }

    async fn load_debug_textures(&mut self) {
        if let Err(e) = self
            .asset_manager
            .load_texture("berlin/Internal/13.png")
            .await
        {
            eprintln!("{}", e);
        }
    }

    async fn load_ui_textures(&mut self) {
        for cursor in &self.ui.cursors {
            if let Err(e) = self.asset_manager.load_texture(&cursor.texture).await {
                eprintln!("{}", e);
            }
        }

        for menu_item in &self.ui.menu_items {
            if let Err(e) = self.asset_manager.load_texture(&menu_item.texture).await {
                eprintln!("{}", e);
            }
        }

        if let Err(e) = self
            .asset_manager
            .load_texture(&self.ui.general_textures.dialog_background)
            .await
        {
            eprintln!("{}", e);
        }
    }

    async fn load_item_textures(&mut self) {
        let mut textures_to_load = Vec::new();

        for item in &self.items {
            textures_to_load.push(item.textures.in_world.clone());
            textures_to_load.push(item.textures.mouse_over.clone());
            textures_to_load.push(item.textures.in_inventory.clone());
            textures_to_load.push(item.textures.in_inventory_text.clone());
        }

        self.asset_manager.load_textures(&textures_to_load).await;
    }

    async fn load_inventory_textures(&mut self) {
        for i in 1..=13 {
            let path = format!("Huvudmeny/inventory/väska{}.png", i);
            if let Err(e) = self.asset_manager.load_texture(&path).await {
                eprintln!("{}", e);
            }
        }

        // Load arrow textures
        let arrow_paths = [
            "Huvudmeny/inventory/pilv-271.png",
            "Huvudmeny/inventory/pilh-272.png",
        ];
        for path in arrow_paths.iter() {
            if let Err(e) = self.asset_manager.load_texture(path).await {
                eprintln!("{}", e);
            }
        }
    }

    fn load_level_scenes(&mut self, level_id: u32) {
        if let Some(level) = self.levels.iter().find(|l| l.id == level_id) {
            self.scenes = Scenes {
                data: level.scenes.clone(),
                count: level.scenes.len(),
            };
            //self.world_items = level.scenes.iter().map(|s| s.items.clone()).collect();
            self.world_items = level
                .scenes
                .iter()
                .map(|s| {
                    s.items
                        .iter()
                        .map(|item| {
                            let mut new_item = item.clone();
                            new_item.x *= 3.0;
                            new_item.y *= 3.0;
                            //new_item.width *= 3.0;
                            //new_item.height *= 3.0;
                            new_item
                        })
                        .collect::<Vec<ItemInstance>>()
                })
                .collect();
            //self.current_scene = 0; // Reset to the first scene of the new level

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

    fn get_game_coordinates(&self, mouse_pos: Vec2) -> Vec2 {
        self.renderer.get_game_coordinates(mouse_pos)
    }

    fn determine_cursor(&self, game_pos: Vec2) -> CursorType {
        // Check for items first
        let current_scene_items = &self.world_items[self.current_scene as usize];
        for item in current_scene_items {
            if self.is_mouse_over_item(game_pos, item) && self.is_item_in_range(item) {
                return CursorType::Take;
            }
        }

        // Then check for clickable areas
        if let Some(current_scene) = self.get_current_scene() {
            // Check for dialog regions
            for dialog in &current_scene.dialogs {
                if game_pos.x >= dialog.x
                    && game_pos.x <= dialog.x + dialog.width
                    && game_pos.y >= dialog.y
                    && game_pos.y <= dialog.y + dialog.height
                {
                    return CursorType::Talk;
                }
            }

            // Check for scene transitions
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
            _ => Direction::South,
        }
    }

    fn update_window_size(&mut self) {
        let new_window_size = Vec2::new(screen_width(), screen_height());
        if new_window_size != self.window_size {
            self.window_size = new_window_size;
            self.renderer.update_window_size(new_window_size);
        }
    }

    fn get_clicked_item(&self, game_pos: Vec2) -> Option<(usize, &ItemInstance)> {
        let current_scene = self.current_scene as usize;
        self.world_items[current_scene]
            .iter()
            .enumerate()
            .find(|(_, item)| self.is_mouse_over_item(game_pos, item))
    }

    fn find_closest_accessible_position(&self, target_pos: Vec2) -> Option<(i32, i32)> {
        let target_grid = self.grid.get_grid_from_coord(target_pos);
        let search_radius = 5; // Adjust this value as needed

        let mut closest_pos = None;
        let mut min_distance = f32::MAX;

        for dx in -search_radius..=search_radius {
            for dy in -search_radius..=search_radius {
                let grid_pos = (target_grid.0 + dx, target_grid.1 + dy);
                if self.grid.is_node_walkable(grid_pos) {
                    let pos = self.grid.get_coord_from_grid(grid_pos.0, grid_pos.1);
                    let distance = Vec2::distance(target_pos, pos);
                    if distance < min_distance {
                        min_distance = distance;
                        closest_pos = Some(grid_pos);
                    }
                }
            }
        }

        closest_pos
    }

    fn is_item_in_range(&self, item: &ItemInstance) -> bool {
        if let Some(active_character_index) = self.active_character {
            let character_pos = self.characters.positions[active_character_index];
            let item_pos = Vec2::new(item.x, item.y);
            let distance = Vec2::distance(item_pos, character_pos);
            distance <= character::INTERACTION_RANGE
        } else {
            false
        }
    }

    fn handle_item_click(&mut self, game_pos: Vec2) {
        let current_scene = self.current_scene as usize;
        let active_character_name = self
            .active_character
            .and_then(|index| self.characters.data.get(index))
            .map(|character| character.name.clone());

        if let Some(active_character_name) = active_character_name {
            let pickup_info = self.world_items[current_scene]
                .iter()
                .enumerate()
                .find(|(_, item)| self.is_mouse_over_item(game_pos, item))
                .and_then(|(index, world_item)| {
                    self.items
                        .iter()
                        .find(|i| i.id == world_item.item_id)
                        .map(|item| {
                            (
                                index,
                                item.id,
                                world_item,
                                item.allowed_characters.contains(&active_character_name),
                                item.pickup_audio.get(&active_character_name).cloned(),
                            )
                        })
                });

            if let Some((item_index, item_id, world_item, is_allowed, maybe_audio)) = pickup_info {
                if !self.is_item_in_range(&world_item) {
                    return;
                }

                if is_allowed {
                    if self.add_item_to_inventory(item_id) {
                        println!("Item added to inventory");
                        self.world_items[current_scene].remove(item_index);
                    } else {
                        println!("Inventory is full!");
                    }
                } else {
                    println!("{} cannot pick up this item!", active_character_name);
                }

                // Play pickup audio
                if let Some(audio_files) = maybe_audio {
                    if let Some(audio_file) = audio_files.choose() {
                        self.audio_system.play_audio(
                            &self.asset_manager,
                            audio_file,
                            AudioCategory::SoundEffect,
                        );
                    }
                }
            }
        }
    }

    fn is_item_in_inventory(&self, item_id: u32) -> bool {
        self.inventory
            .items
            .iter()
            .any(|&item| item == Some(item_id))
    }

    fn update_inventory_animation(&mut self, delta_time: f32) {
        const ANIMATION_SPEED: f32 = 0.03;
        const TOTAL_FRAMES: usize = 13;

        self.inventory.animation_timer += delta_time;

        if self.inventory.animation_timer >= ANIMATION_SPEED {
            self.inventory.animation_timer -= ANIMATION_SPEED;

            if self.inventory.open {
                if self.inventory.animation_frame < TOTAL_FRAMES - 1 {
                    self.inventory.animation_frame += 1;
                }
            } else {
                if self.inventory.animation_frame > 0 {
                    self.inventory.animation_frame -= 1;
                }
            }
        }
    }

    fn update_inventory(&mut self, mouse_pos: Vec2) {
        if self.inventory.open {
            self.inventory.hovered_slot = None;
            self.inventory.hovered_left_arrow = false;
            self.inventory.hovered_right_arrow = false;

            for i in 0..inventory::SLOT_COUNT {
                let slot_x = inventory::START_X
                    + (inventory::SLOT_SIZE + inventory::SLOT_SPACING) * i as f32;
                let slot_rect = Rect::new(
                    slot_x,
                    inventory::START_Y,
                    inventory::SLOT_SIZE,
                    inventory::SLOT_SIZE,
                );

                if slot_rect.contains(mouse_pos) {
                    self.inventory.hovered_slot = Some(i);
                    break;
                }
            }

            // Check for arrow hovering
            if self.inventory.left_arrow_rect.contains(mouse_pos) {
                self.inventory.hovered_left_arrow = true;
            } else if self.inventory.right_arrow_rect.contains(mouse_pos) {
                self.inventory.hovered_right_arrow = true;
            }
        }
    }

    fn is_double_click(&mut self, character_index: usize) -> bool {
        let current_time = get_time();
        let last_click_time = &mut self.characters.last_click_times[character_index];
        let is_double = current_time - *last_click_time < 0.3; // 300ms threshold for double-click
        *last_click_time = current_time;
        is_double
    }

    fn scroll_inventory(&mut self, direction: i32) {
        let items_count = self
            .inventory
            .items
            .iter()
            .filter(|&item| item.is_some())
            .count();
        let max_scroll = items_count.saturating_sub(inventory::SLOT_COUNT);

        let new_scroll_offset = (self.inventory.scroll_offset as i32 + direction)
            .max(0)
            .min(max_scroll as i32) as usize;

        // Only update if the scroll actually changed
        if new_scroll_offset != self.inventory.scroll_offset {
            self.inventory.scroll_offset = new_scroll_offset;
        }
    }

    fn add_item_to_inventory(&mut self, item_id: u32) -> bool {
        if let Some(empty_slot) = self.inventory.items.iter_mut().find(|slot| slot.is_none()) {
            *empty_slot = Some(item_id);
            true
        } else {
            false // Inventory is full
        }
    }

    fn toggle_inventory(&mut self) {
        self.inventory.open = !self.inventory.open;

        let audio_path = if self.inventory.open {
            "Huvudmeny/ljudfx/oppna.wav"
        } else {
            "Huvudmeny/ljudfx/stanga.wav"
        };

        self.audio_system
            .play_audio(&self.asset_manager, audio_path, AudioCategory::SoundEffect);
    }

    async fn handle_mouse_click(&mut self, game_pos: Vec2) {
        if !self.renderer.is_in_game_area(game_pos) {
            return;
        }

        if self.inventory.button_rect.contains(game_pos) {
            self.toggle_inventory();
            return;
        }

        // Handle inventory interaction
        if self.inventory.open {
            let inventory_top = inventory::START_Y - 59.0;
            // Check if click is inside or below the inventory area
            if game_pos.y >= inventory_top {
                // Handle left arrow click
                if self.inventory.left_arrow_rect.contains(game_pos) {
                    self.scroll_inventory(-1);
                    return;
                }
                // Handle right arrow click
                if self.inventory.right_arrow_rect.contains(game_pos) {
                    self.scroll_inventory(1);
                    return;
                }
                // If we've reached here, the click was inside or below the inventory area
                // so we keep it open and do nothing
                return;
            }
            // If the click is above the inventory, close it
            if game_pos.y < inventory_top {
                self.toggle_inventory();
                return;
            }
        }

        // Check if the dialog is open and the click is within the dialog area
        if self.dialog_menu.open {
            let in_dialog_area = game_pos.y >= config::dialog::START_Y && game_pos.y <= 1440.0;
            if in_dialog_area {
                if let Some(selected_option) = self.get_clicked_dialog_option(game_pos) {
                    self.handle_dialog_option_selection(selected_option);
                }
                return;
            } else {
                // Close the dialog if clicked outside
                self.close_dialog_menu();
                return;
            }
        }

        // Check if a character was clicked
        if let Some(index) =
            (0..self.characters.count).find(|&i| self.is_point_in_character(game_pos, i))
        {
            if Some(index) != self.active_character {
                self.active_character = Some(index);

                // Play select character audio
                if let Some(audio_file) = self.characters.data[index].select_audio.choose() {
                    self.audio_system.play_audio(
                        &self.asset_manager,
                        audio_file,
                        AudioCategory::SoundEffect,
                    );
                }
                return;
            }
        }

        // Check for dialog interactions
        let dialog_clicked = self
            .get_current_scene()
            .map(|current_scene| {
                current_scene.dialogs.iter().any(|dialog| {
                    game_pos.x >= dialog.x
                        && game_pos.x <= dialog.x + dialog.width
                        && game_pos.y >= dialog.y
                        && game_pos.y <= dialog.y + dialog.height
                })
            })
            .unwrap_or(false);

        if dialog_clicked {
            self.open_dialog_menu(game_pos);
            return;
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

        // Handle double-clicks and pathfinding
        if let Some(active_index) = self.active_character {
            let is_running = self.is_double_click(active_index);
            self.characters.is_running[active_index] = is_running;
        }
        self.handle_pathfinding(game_pos).await;
    }

    fn handle_right_click(&mut self, game_pos: Vec2) {
        if self.debug_tools.bounding_box_mode {
            self.debug_tools.handle_bounding_box_creation(game_pos);
        }
    }

    fn open_dialog_menu(&mut self, game_pos: Vec2) {
        let dialog_id = self.get_current_scene().and_then(|current_scene| {
            current_scene
                .dialogs
                .iter()
                .find(|dialog| {
                    game_pos.x >= dialog.x
                        && game_pos.x <= dialog.x + dialog.width
                        && game_pos.y >= dialog.y
                        && game_pos.y <= dialog.y + dialog.height
                })
                .map(|dialog| dialog.id)
        });

        if let Some(id) = dialog_id {
            self.dialog_menu.open = true;
            self.dialog_menu.current_dialog_id = Some(id);
            self.play_open_dialog_sound(id);
        }
    }

    fn play_open_dialog_sound(&mut self, dialog_id: u32) {
        if let Some(current_scene) = self.get_current_scene() {
            if let Some(dialog) = current_scene.dialogs.iter().find(|d| d.id == dialog_id) {
                if let Some(audio) = &dialog.open_audio {
                    let audio_to_play = format!(
                        "voice/{}/{}_{}.wav",
                        current_scene.name, current_scene.name, audio
                    );
                    self.audio_system.play_audio(
                        &self.asset_manager,
                        &audio_to_play,
                        AudioCategory::Dialog,
                    );
                }
            }
        }
    }

    fn close_dialog_menu(&mut self) {
        self.dialog_menu.open = false;
        self.dialog_menu.current_dialog_id = None;
        self.dialog_menu.current_level = 0;
    }

    fn get_clicked_dialog_option(&self, game_pos: Vec2) -> Option<usize> {
        if let Some(current_scene) = self.get_current_scene() {
            if let Some(dialog_id) = self.dialog_menu.current_dialog_id {
                if let Some(dialog) = current_scene.dialogs.iter().find(|d| d.id == dialog_id) {
                    if let Some(level) = dialog.tree.get(self.dialog_menu.current_level) {
                        // Calculate the relative mouse position within the dialog area
                        let relative_pos = Vec2::new(
                            game_pos.x - config::dialog::OPTION_START_X,
                            game_pos.y - config::dialog::START_Y - config::dialog::OPTION_START_Y,
                        );

                        for (i, _) in level.options.iter().enumerate() {
                            let option_y = i as f32 * config::dialog::OPTION_SPACING;
                            let option_rect = Rect::new(
                                0.0,
                                option_y,
                                config::dialog::OPTION_BOX_WIDTH,
                                config::dialog::OPTION_BOX_HEIGHT,
                            );

                            if option_rect.contains(relative_pos) {
                                return Some(i);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn update_dialog_hover(&mut self, mouse_pos: Vec2) {
        if self.dialog_menu.open {
            self.dialog_menu.hovered_option = self.get_clicked_dialog_option(mouse_pos);
        }
    }

    fn handle_dialog_option_selection(&mut self, selected_option: usize) {
        let mut audio_to_play = None;
        let mut next_level = None;

        if let Some(current_scene) = self.get_current_scene() {
            if let Some(dialog_id) = self.dialog_menu.current_dialog_id {
                if let Some(dialog) = current_scene.dialogs.iter().find(|d| d.id == dialog_id) {
                    if let Some(level) = dialog.tree.get(self.dialog_menu.current_level) {
                        if let Some(option) = level.options.get(selected_option) {
                            println!("Selected option: {}", option.text);
                            if let Some(audio) = option.response_audio.choose() {
                                audio_to_play = Some(format!(
                                    "voice/{}/{}_{}.wav",
                                    current_scene.name, current_scene.name, audio
                                ));
                            }
                            next_level = Some(option.target as usize);
                        }
                    }
                }
            }
        }

        // Now that we've gathered all the information, we can modify the state
        if let Some(audio_path) = audio_to_play {
            self.audio_system
                .play_audio(&self.asset_manager, &audio_path, AudioCategory::Dialog);
        }

        if let Some(level) = next_level {
            self.dialog_menu.current_level = level;
        }

        // Use 100 as indication that the dialog should be closed
        if next_level == Some(100) {
            self.close_dialog_menu();
        }
    }

    fn is_point_in_character(&self, point: Vec2, character_index: usize) -> bool {
        let character_pos = self.characters.positions[character_index];

        point.x >= character_pos.x + character::X_OFFSET - character::WIDTH / 2.0
            && point.x <= character_pos.x + character::X_OFFSET + character::WIDTH / 2.0
            && point.y >= character_pos.y + character::Y_OFFSET - character::HEIGHT / 2.0
            && point.y <= character_pos.y + character::Y_OFFSET + character::HEIGHT / 2.0
    }

    fn is_active_character_in_transition_area(&self, transition: &SceneTransition) -> bool {
        if let Some(active_index) = self.active_character {
            let character_pos = self.characters.positions[active_index];
            let in_area = character_pos.x >= transition.x
                && character_pos.x <= transition.x + transition.width
                && character_pos.y >= transition.y
                && character_pos.y <= transition.y + transition.height;

            if in_area {
                return true;
            }

            // Check if the character is at the closest possible position
            self.is_character_at_closest_position(active_index, transition)
        } else {
            false
        }
    }

    fn is_character_at_closest_position(
        &self,
        character_index: usize,
        transition: &SceneTransition,
    ) -> bool {
        let character_grid_pos = self
            .grid
            .get_grid_from_coord(self.characters.positions[character_index]);
        let closest_node = self.find_closest_walkable_node(character_grid_pos, transition);

        if let Some(closest_node) = closest_node {
            character_grid_pos == closest_node
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

            self.grid.update_blocked_nodes(blocked_nodes);

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
        }

        self.load_current_and_adjacent_scenes().await;
        self.update_scene_audio();
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

        let spawn_positions = self.find_n_closest_walkable_grids(center, count).unwrap();

        for i in 0..count {
            let pos = self
                .grid
                .get_coord_from_grid(spawn_positions[i].0, spawn_positions[i].1);
            positions.push(pos);
        }

        positions
    }

    async fn handle_pathfinding(&mut self, target_pos: Vec2) {
        if let Some(active_index) = self.active_character {
            let target_grid = self.grid.get_grid_from_coord(target_pos);
            let mut final_target = target_grid;

            let grid_pos_player = self
                .grid
                .get_grid_from_coord(self.characters.positions[active_index]);

            if let Some(_clicked_item) = self.get_clicked_item(target_pos) {
                if let Some(closest_grid) = self.find_closest_accessible_position(target_pos) {
                    final_target = closest_grid;
                }
            } else if let Some(transition) = self.find_clicked_transition(target_pos) {
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

            // Check if the clicked position is the same as the current target
            if let Some(current_target) = self.characters.targets[active_index] {
                if current_target == final_target {
                    return;
                }
            }

            // Don't move if the player is already at the target
            if grid_pos_player == final_target {
                return;
            }

            let start_grid = self
                .grid
                .get_grid_from_coord(self.characters.positions[active_index]);

            if let Some(path) = self.grid.pathfind(start_grid, final_target) {
                self.characters.paths[active_index] = Some(path);
                self.characters.targets[active_index] = Some(final_target);
            } else {
                // If no path is found, stop the character
                self.stop_character(active_index);
            }
        }
    }

    fn find_n_closest_walkable_grids(&self, pixel_pos: Vec2, n: usize) -> Option<Vec<(i32, i32)>> {
        let target_grid = self.grid.get_grid_from_coord(pixel_pos);
        let mut walkable_grids = Vec::new();

        let search_radius = 10;

        for dx in -search_radius..=search_radius {
            for dy in -search_radius..=search_radius {
                let grid_pos = (target_grid.0 + dx, target_grid.1 + dy);

                if self.grid.is_node_walkable(grid_pos) {
                    let distance = (dx * dx + dy * dy) as f32;
                    walkable_grids.push((grid_pos, distance));
                }
            }
        }

        if walkable_grids.is_empty() {
            return None;
        }

        walkable_grids.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        Some(
            walkable_grids
                .into_iter()
                .take(n)
                .map(|(pos, _)| pos)
                .collect(),
        )
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

        if is_key_pressed(KeyCode::D) {
            self.debug_tools.active = !self.debug_tools.active;
        }
        if is_key_pressed(KeyCode::G) {
            if self.debug_tools.active {
                self.debug_tools.draw_grid = !self.debug_tools.draw_grid;
            }
        }

        if is_key_pressed(KeyCode::M) && self.debug_tools.active {
            if self.audio_system.is_muted() {
                println!("Unmuting audio");
            } else {
                println!("Muting audio");
            }
            self.audio_system.toggle_mute(&self.asset_manager);
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

        if self.debug_tools.active {
            if is_key_pressed(KeyCode::L) {
                self.debug_level_switch_mode = !self.debug_level_switch_mode;
            }

            if self.debug_level_switch_mode {
                for i in 0..10 {
                    // Support up to 10 levels (0-9)
                    if is_key_pressed(match i {
                        0 => KeyCode::Key0,
                        1 => KeyCode::Key1,
                        2 => KeyCode::Key2,
                        3 => KeyCode::Key3,
                        4 => KeyCode::Key4,
                        5 => KeyCode::Key5,
                        6 => KeyCode::Key6,
                        7 => KeyCode::Key7,
                        8 => KeyCode::Key8,
                        9 => KeyCode::Key9,
                        _ => continue,
                    }) {
                        self.switch_to_level(i as u32).await;
                        break;
                    }
                }
            }
        }
        // Update cursor based on game position
        let new_cursor_type = self.determine_cursor(game_pos);
        if new_cursor_type != self.current_cursor {
            self.current_cursor = new_cursor_type;
        }

        self.update_dialog_hover(game_pos);

        let delta_time = get_frame_time();
        self.update_characters(delta_time);
        self.update_inventory_animation(delta_time);
        self.update_inventory(game_pos);
    }

    async fn switch_to_level(&mut self, level_index: u32) {
        if level_index < self.levels.len() as u32 {
            self.current_level = level_index;
            self.load_level_scenes(self.current_level);
            self.current_scene = 0; // Reset to the first scene of the new level
            self.load_current_and_adjacent_scenes().await;

            let spawn_position = Vec2::new(1000.0, 800.0); // Default spawn position
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

            println!(
                "Switched to level: {}",
                self.levels[level_index as usize].name
            );
        } else {
            println!("Invalid level index: {}", level_index);
        }
    }

    fn update_characters(&mut self, delta_time: f32) {
        for i in 0..self.characters.count {
            if let Some(path) = &mut self.characters.paths[i] {
                if !path.is_empty() {
                    let target = self.grid.get_coord_from_grid(path[0].0, path[0].1);
                    let direction = (target - self.characters.positions[i]).normalize_or_zero();

                    let speed = if self.characters.is_running[i] {
                        self.characters.data[i].run_speed
                    } else {
                        self.characters.data[i].speed
                    };

                    let new_position =
                        self.characters.positions[i] + direction * speed * delta_time;
                    self.characters.positions[i] = new_position;

                    // Update direction only if we're actually moving
                    if direction != Vec2::ZERO {
                        self.characters.directions[i] = Self::vec_to_direction(direction);
                    }

                    // Update animation
                    self.characters.animation_timers[i] += delta_time;
                    if self.characters.animation_timers[i] >= self.characters.animation_speeds[i] {
                        self.characters.animation_timers[i] -= self.characters.animation_speeds[i];
                        self.characters.animation_indices[i] =
                            (self.characters.animation_indices[i] + 1) % 8;
                    }

                    // Check if character has reached the current path node
                    if (self.characters.positions[i] - target).length_squared() < 25.0 {
                        path.remove(0);
                        if path.is_empty() {
                            self.stop_character(i);
                        }
                    }
                } else {
                    self.stop_character(i);
                }
            } else {
                self.reset_character_animation(i);
            }
        }
    }

    fn stop_character(&mut self, index: usize) {
        self.characters.paths[index] = None;
        self.characters.targets[index] = None;
        self.characters.is_running[index] = false;
        self.reset_character_animation(index);
    }

    fn reset_character_animation(&mut self, index: usize) {
        self.characters.animation_indices[index] = 0;
        self.characters.animation_timers[index] = 0.0;
    }

    fn draw(&self) {
        self.renderer.draw(self, &self.asset_manager);
        draw_text(&format!("FPS: {}", get_fps()), 0., 16., 32., crate::WHITE);
    }

    fn is_mouse_over_item(&self, game_pos: Vec2, item: &ItemInstance) -> bool {
        game_pos.x >= item.x
            && game_pos.x <= item.x + item.width
            && game_pos.y >= item.y
            && game_pos.y <= item.y + item.height
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
