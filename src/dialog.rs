use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Dialog {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub description: String,
    pub open_audio: Option<String>,
    pub tree: Vec<DialogNode>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DialogNode {
    pub level: u32,
    pub options: Vec<DialogOption>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DialogOption {
    #[serde(rename = "option_id")]
    pub id: u32,
    pub text: String,
    pub response_audio: Vec<String>,
    pub target: u32,
}

pub struct DialogMenu {
    pub open: bool,
    pub current_dialog_id: Option<u32>,
    pub current_level: usize,
    pub hovered_option: Option<usize>,
}

impl DialogMenu {
    pub fn new() -> Self {
        DialogMenu {
            open: false,
            current_dialog_id: None,
            current_level: 0,
            hovered_option: None,
        }
    }
}
