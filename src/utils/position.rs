use amethyst::core::transform::Transform;
use serde::Deserialize;
use serde::Serialize;

// TODO: Move to models
#[derive(Copy, Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub direction: f32,
}

impl Position {
    pub const fn new(x: f32, y: f32, direction: f32) -> Self {
        return Self { x, y, direction };
    }
}

impl From<&Transform> for Position {
    fn from(transform: &Transform) -> Self {
        let translation = transform.translation();

        return Self {
            x: translation.x,
            y: translation.y,
            direction: transform.euler_angles().2,
        };
    }
}
