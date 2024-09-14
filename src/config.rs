pub mod character {
    // Character rendering and interaction offsets
    pub const X_OFFSET: f32 = -4.0;
    pub const Y_OFFSET: f32 = -90.0;

    // Character dimensions for hitbox calculation
    pub const WIDTH: f32 = 55.0;
    pub const HEIGHT: f32 = 120.0;
}

pub mod inventory {
    pub const START_X: f32 = 120.0;
    pub const START_Y: f32 = 1280.0;
    pub const SLOT_SIZE: f32 = 150.0;
    pub const SLOT_SPACING: f32 = 0.0;
    pub const SLOT_COUNT: usize = 9;
    pub const INVENTORY_SIZE: usize = 30;
    pub const LEFT_ARROW_OFFSET_X: f32 = -60.0; // Offset from the left edge of the first slot
    pub const RIGHT_ARROW_OFFSET_X: f32 = 10.0; // Offset from the right edge of the last slot
    pub const ARROW_OFFSET_Y: f32 = 20.0; // Vertical offset from the slots (0 means aligned with slots)
    pub const ARROW_SIZE: f32 = 50.0; // Size of the arrow buttons
}

pub mod dialog {
    use macroquad::prelude::Color;
    use macroquad::prelude::{GREEN, DARKGRAY, GRAY, WHITE, YELLOW, RED};

    pub const WIDTH: f32 = 1920.0;
    pub const HEIGHT: f32 = 258.0;
    pub const START_Y: f32 = 1440.0 - HEIGHT;
    pub const TEXT_PADDING_X: f32 = 20.0;
    pub const TEXT_PADDING_Y: f32 = 20.0;
    pub const FONT_SIZE: f32 = 40.0;

    // New configuration options
    pub const OPTION_START_X: f32 = 45.0;
    pub const OPTION_START_Y: f32 = 30.0;
    pub const OPTION_SPACING: f32 = 55.0;
    pub const OPTION_BOX_WIDTH: f32 = 1280.0;
    pub const OPTION_BOX_HEIGHT: f32 = 50.0;
    pub const OPTION_TEXT_COLOR: Color = WHITE;
    pub const OPTION_HOVER_TEXT_COLOR: Color = YELLOW;
    pub const OPTION_BOX_COLOR: Color = GREEN;
    pub const OPTION_HOVER_BOX_COLOR: Color = RED;
}
