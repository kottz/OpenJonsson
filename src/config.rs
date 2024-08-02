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
}
