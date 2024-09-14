use crate::asset_manager::AssetManager;
use macroquad::audio::{play_sound, stop_sound, Sound};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AudioCategory {
    Music,
    Dialog,
    SoundEffect,
}

pub struct AudioSystem {
    volume_levels: HashMap<AudioCategory, f32>,
    pub currently_playing: HashMap<AudioCategory, Option<String>>,
}

impl AudioSystem {
    pub fn new() -> Self {
        let mut volume_levels = HashMap::new();
        volume_levels.insert(AudioCategory::Music, 1.0);
        volume_levels.insert(AudioCategory::Dialog, 1.0);
        volume_levels.insert(AudioCategory::SoundEffect, 1.0);

        AudioSystem {
            volume_levels,
            currently_playing: HashMap::new(),
        }
    }

    pub fn play_audio(
        &mut self,
        asset_manager: &AssetManager,
        name: &str,
        category: AudioCategory,
    ) {
        if let Some(sound) = asset_manager.get_sound(name) {
            let volume = self.volume_levels.get(&category).cloned().unwrap_or(1.0);

            // Stop any currently playing audio in the same category
            if let Some(current_name) = self.currently_playing.get(&category).cloned().flatten() {
                if current_name != name {
                    if let Some(current_sound) = asset_manager.get_sound(&current_name) {
                        stop_sound(current_sound);
                    }
                }
            }

            play_sound(
                sound,
                macroquad::audio::PlaySoundParams {
                    looped: category == AudioCategory::Music,
                    volume,
                },
            );
            self.currently_playing
                .insert(category, Some(name.to_string()));
            println!("Playing audio: {}", name); // Debug print
        } else {
            println!("Audio not found: {}", name); // Debug print
        }
    }

    pub fn stop_audio(&mut self, asset_manager: &AssetManager, category: &AudioCategory) {
        if let Some(Some(current_name)) = self.currently_playing.get(category) {
            if let Some(sound) = asset_manager.get_sound(current_name) {
                stop_sound(sound);
                self.currently_playing.insert(category.clone(), None);
            }
        }
    }

    pub fn set_volume(&mut self, category: AudioCategory, volume: f32) {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.volume_levels.insert(category, clamped_volume);
        // Note: We can't update the volume of currently playing sounds here
        // because we don't have access to the AssetManager
    }

    pub fn get_volume(&self, category: &AudioCategory) -> f32 {
        *self.volume_levels.get(category).unwrap_or(&1.0)
    }

    pub fn play_music(&mut self, asset_manager: &AssetManager, name: &str) {
        self.play_audio(asset_manager, name, AudioCategory::Music);
    }

    pub fn stop_music(&mut self, asset_manager: &AssetManager) {
        self.stop_audio(asset_manager, &AudioCategory::Music);
    }
}
