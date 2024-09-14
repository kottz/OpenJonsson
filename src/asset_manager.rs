use macroquad::audio::{load_sound, Sound};
use macroquad::prelude::*;
use std::collections::HashMap;

pub struct AssetManager {
    textures: HashMap<String, Texture2D>,
    pub sounds: HashMap<String, Sound>,
    loading_textures: Vec<String>,
    fonts: HashMap<String, Font>,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager {
            textures: HashMap::new(),
            sounds: HashMap::new(),
            loading_textures: Vec::new(),
            fonts: HashMap::new(),
        }
    }

    pub async fn load_texture(&mut self, path: &str) -> Result<(), String> {
        if self.textures.contains_key(path) || self.loading_textures.contains(&path.to_string()) {
            return Ok(());
        }

        self.loading_textures.push(path.to_string());
        let full_path = format!("static/resources/{}", path);
        match load_texture(&full_path).await {
            Ok(texture) => {
                self.textures.insert(path.to_string(), texture);
                self.loading_textures.retain(|x| x != path);
                Ok(())
            }
            Err(e) => {
                self.loading_textures.retain(|x| x != path);
                Err(format!("Failed to load texture {}: {}", path, e))
            }
        }
    }

    pub async fn load_sound(&mut self, path: &str) -> Result<(), String> {
        if self.sounds.contains_key(path) {
            return Ok(());
        }
        let full_path = format!("static/resources/{}", path);
        match load_sound(&full_path).await {
            Ok(sound) => {
                self.sounds.insert(path.to_string(), sound);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load sound {}: {}", path, e)),
        }
    }

    pub fn get_texture(&self, path: &str) -> Option<&Texture2D> {
        self.textures.get(path)
    }

    pub fn get_sound(&self, path: &str) -> Option<&Sound> {
        self.sounds.get(path)
    }

    pub async fn load_textures(&mut self, paths: &[String]) {
        for path in paths {
            if let Err(e) = self.load_texture(path).await {
                eprintln!("{}", e);
            }
        }
    }

    pub async fn load_font(&mut self, name: &str, path: &str) -> Result<(), String> {
        match load_ttf_font(path).await {
            Ok(font) => {
                self.fonts.insert(name.to_string(), font);
                Ok(())
            }
            Err(e) => Err(format!("Failed to load font {} from {}: {}", name, path, e)),
        }
    }

    pub fn get_font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name)
    }
}
