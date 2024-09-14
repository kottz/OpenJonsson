use crate::asset_manager::AssetManager;
use macroquad::audio::{play_sound, set_sound_volume, stop_sound};

pub struct AudioSystem {
    current_music: Option<String>,
    master_volume: f32,
}

impl AudioSystem {
    pub fn new() -> Self {
        AudioSystem {
            current_music: None,
            master_volume: 1.0,
        }
    }

    pub fn play_music(&mut self, asset_manager: &AssetManager, name: &str) {
        if let Some(current_music) = &self.current_music {
            if current_music == name {
                return; // The requested music is already playing
            }
            self.stop_music(asset_manager);
        }

        if let Some(sound) = asset_manager.get_sound(name) {
            play_sound(
                sound,
                macroquad::audio::PlaySoundParams {
                    looped: true,
                    volume: self.master_volume,
                },
            );
            self.current_music = Some(name.to_string());
        }
    }

    pub fn stop_music(&mut self, asset_manager: &AssetManager) {
        if let Some(current_music) = &self.current_music {
            if let Some(sound) = asset_manager.get_sound(current_music) {
                stop_sound(sound);
            }
            self.current_music = None;
        }
    }

    pub fn set_master_volume(&mut self, asset_manager: &AssetManager, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        for sound in asset_manager.sounds.values() {
            set_sound_volume(sound, self.master_volume);
        }
    }
}
