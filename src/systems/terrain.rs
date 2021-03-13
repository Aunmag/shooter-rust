use crate::components::Terrain;
use amethyst::core::transform::Transform;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::WriteStorage;
use amethyst::renderer::Camera;
use amethyst::tiles::MortonEncoder;
use amethyst::tiles::TileMap;
use amethyst::ecs::prelude::Write;
use crate::resources::DebugTimer;

#[derive(SystemDesc)]
pub struct TerrainSystem;

impl<'a> System<'a> for TerrainSystem {
    type SystemData = (
        ReadStorage<'a, Camera>,
        ReadStorage<'a, TileMap<Terrain, MortonEncoder>>,
        WriteStorage<'a, Transform>,
        Write<'a, DebugTimer>,
    );

    fn run(&mut self, (cameras, tiles, mut transforms, mut dt): Self::SystemData) {
        let s = dt.start();
        let mut x = 0.0;
        let mut y = 0.0;

        #[allow(clippy::never_loop)]
        for (_, transform) in (&cameras, &transforms).join() {
            // TODO: Use `global_translation` in future amethyst version
            // TODO: Maybe there's a way to avoid global matrix calculation
            let translation = transform.global_matrix().column(3).xyz();
            x = align(translation.x, Terrain::SIZE as f32) + Terrain::SIZE_HALF;
            y = align(translation.y, Terrain::SIZE as f32) - Terrain::SIZE_HALF;
            break;
        }

        // TODO: Cache, do not run every time
        for (_, transform) in (&tiles, &mut transforms).join() {
            transform.set_translation_x(x);
            transform.set_translation_y(y);
        }

        dt.end("Terrain", s);
    }
}

fn align(n: f32, reminder: f32) -> f32 {
    return (n / reminder).round() * reminder;
}
