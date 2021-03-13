#![warn(clippy::all, clippy::cargo, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::as_conversions,
    clippy::blanket_clippy_restriction_lints,
    clippy::cargo_common_metadata,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::default_trait_access,
    clippy::else_if_without_else,
    clippy::explicit_iter_loop,
    clippy::float_arithmetic,
    clippy::implicit_return, // TODO: Enable later excepting closures
    clippy::integer_arithmetic,
    clippy::match_wildcard_for_single_variants,
    clippy::missing_docs_in_private_items,
    clippy::module_name_repetitions,
    clippy::modulo_arithmetic,
    clippy::multiple_crate_versions,
    clippy::needless_return,
    clippy::redundant_else,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::wildcard_enum_match_arm,
)]

mod components;
mod data;
mod input;
mod resources;
mod states;
mod systems;
mod utils;

use crate::components::Terrain;
use crate::input::CustomBindingTypes;
use crate::states::StartupState;
use crate::systems::UiResizeSystem;
use crate::systems::UiTaskSystem;
use amethyst::controls::CursorHideSystemDesc;
use amethyst::controls::MouseFocusUpdateSystemDesc;
use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::transform::TransformBundle;
use amethyst::core::HideHierarchySystemDesc;
use amethyst::input::InputBundle;
use amethyst::prelude::*;
use amethyst::renderer::plugins::RenderFlat2D;
use amethyst::renderer::plugins::RenderToWindow;
use amethyst::renderer::types::DefaultBackend;
use amethyst::renderer::RenderDebugLines;
use amethyst::renderer::RenderingBundle;
use amethyst::tiles::MortonEncoder;
use amethyst::tiles::RenderTiles2D;
use amethyst::ui::RenderUi;
use amethyst::ui::UiBundle;
use amethyst::utils::application_root_dir;

const FRAME_RATE: u32 = 144;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    log::debug!("Actor: {}", std::mem::size_of::<crate::components::Actor>());
    log::debug!("Ai: {}", std::mem::size_of::<crate::components::Ai>());
    log::debug!("Collision: {}", std::mem::size_of::<crate::components::Collision>());
    log::debug!("Interpolation: {}", std::mem::size_of::<crate::components::Interpolation>());
    log::debug!("Player: {}", std::mem::size_of::<crate::components::Player>());
    log::debug!("Projectile: {}", std::mem::size_of::<crate::components::Projectile>());
    log::debug!("Weapon: {}", std::mem::size_of::<crate::components::Weapon>());

    let root = application_root_dir()?;
    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<CustomBindingTypes>::new()
                .with_bindings_from_file(root.join("config/input.ron"))?,
        )?
        .with_system_desc(MouseFocusUpdateSystemDesc::default(), "mouse_focus", &[])
        .with_system_desc(CursorHideSystemDesc::default(), "", &["mouse_focus"])
        .with_system_desc(UiResizeSystem::new(), "", &[])
        .with_system_desc(UiTaskSystem, "", &[])
        .with_system_desc(HideHierarchySystemDesc::default(), "", &[]) // TODO: Maybe this system depends on something?
        .with_bundle(UiBundle::<CustomBindingTypes>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(RenderToWindow::from_config_path(
                    root.join("config/display.ron"),
                )?)
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default())
                .with_plugin(RenderTiles2D::<Terrain, MortonEncoder>::default())
                .with_plugin(RenderDebugLines::default()),
        )?;

    Application::build(root.join("assets/"), StartupState::new())?
        .with_frame_limit(FrameRateLimitStrategy::Yield, FRAME_RATE)
        .build(game_data)?
        .run();

    return Ok(());
}
