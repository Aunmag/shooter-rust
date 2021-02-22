use std::time::Duration;

pub const VIEW_DISTANCE: f32 = 15.0;

pub const WORLD_SIZE: f32 = 64.0;
pub const WORLD_SIZE_HALF: f32 = WORLD_SIZE / 2.0;
pub const WORLD_SIZE_VISUAL: f32 = WORLD_SIZE + VIEW_DISTANCE;

pub const LAYER_TERRAIN: f32 = 0.0;
pub const LAYER_BLUFF: f32 = 0.1;
pub const LAYER_PROJECTILE: f32 = 0.2;
pub const LAYER_ACTOR: f32 = 0.3;
pub const LAYER_ACTOR_PLAYER: f32 = 0.4;
pub const LAYER_TREE: f32 = 0.5;
pub const LAYER_CAMERA: f32 = 1.0;

pub const MAX_PING: Duration = Duration::from_millis(250);
