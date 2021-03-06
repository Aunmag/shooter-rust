use crate::data::VIEW_DISTANCE;
use crate::utils;
use amethyst::core::transform::Transform;
use amethyst::ecs::Join;
use amethyst::ecs::ReadExpect;
use amethyst::ecs::System;
use amethyst::ecs::WriteStorage;
use amethyst::renderer::Camera;
use amethyst::window::ScreenDimensions;

const OFFSET_RATIO: f32 = 0.25;

pub struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        ReadExpect<'a, ScreenDimensions>,
        WriteStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (screen, mut cameras, mut transforms): Self::SystemData) {
        let screen_size_x = screen.width();
        let screen_size_y = screen.height();
        let scale = VIEW_DISTANCE / utils::math::length(screen_size_x, screen_size_y);
        let view_x = 2.0 / (screen_size_x * scale);
        let view_y = -2.0 / (screen_size_y * scale);
        let offset = screen_size_y * scale * OFFSET_RATIO;

        #[allow(clippy::indexing_slicing)]
        for (camera, transform) in (&mut cameras, &mut transforms).join() {
            // Keep in sync with `Camera::standard_2d` sources
            camera.matrix[(0, 0)] = view_x;
            camera.matrix[(1, 1)] = view_y;
            camera.matrix[(0, 3)] = 0.0;
            camera.matrix[(1, 3)] = 0.0;
            transform.set_translation_y(offset);
        }
    }
}
