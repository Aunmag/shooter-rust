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
mod models;
mod resources;
mod states;
mod systems;
mod utils;

use crate::components::Terrain;
use crate::input::CustomBindingTypes;
use crate::resources::State;
use crate::states::StartupState;
use crate::systems::net::ConnectionUpdateSystem;
use crate::systems::net::InputSendSystem;
use crate::systems::net::InterpolationSystem;
use crate::systems::net::MessageReceiveSystem;
use crate::systems::net::PositionUpdateSendSystem;
use crate::systems::net::PositionUpdateSystem;
use crate::systems::ActorSystem;
use crate::systems::AiSystem;
use crate::systems::CameraSystem;
use crate::systems::PhysicsSystem;
use crate::systems::PlayerSystem;
use crate::systems::ProjectileSystem;
use crate::systems::TerrainSystem;
use crate::systems::UiResizeSystem;
use crate::systems::WeaponSystem;
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

    let root = application_root_dir()?;
    let game_data = GameDataBuilder::default()
        // Base
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<CustomBindingTypes>::new()
                .with_bindings_from_file(root.join("config/input.ron"))?,
        )?
        // Game
        .with(InterpolationSystem.pausable(State::Client), "interpolation", &[])
        .with(AiSystem::new().pausable(State::Server), "ai", &[])
        .with(PlayerSystem.pausable(State::Any), "player", &["input_system"])
        .with(ActorSystem.pausable(State::Any), "actor", &["ai", "player", "interpolation"])
        .with(PhysicsSystem::new().pausable(State::Any), "physics", &["actor"])
        .with(InputSendSystem::new().pausable(State::Client), "input_send", &["player", "actor"])
        .with(WeaponSystem::new().pausable(State::Server), "weapon", &["physics"])
        .with(ProjectileSystem.pausable(State::Any), "projectile", &["physics"])
        .with(PositionUpdateSendSystem::new().pausable(State::Server), "position_update_send", &["physics"])
        .with(MessageReceiveSystem.pausable(State::Any), "message_receive", &[])
        .with(PositionUpdateSystem.pausable(State::Client), "position_update", &["message_receive", "physics"])
        .with(ConnectionUpdateSystem.pausable(State::Any), "connection_update", &[])
        .with(CameraSystem.pausable(State::Any), "camera", &[])
        .with(TerrainSystem.pausable(State::Any), "terrain", &[])
        // UI
        .with_system_desc(MouseFocusUpdateSystemDesc::default(), "mouse_focus", &[])
        .with_system_desc(CursorHideSystemDesc::default(), "cursor_hide", &["mouse_focus"])
        .with_system_desc(HideHierarchySystemDesc::default(), "hide_hierarchy", &[])
        .with(UiResizeSystem::new(), "ui_resize", &[])
        .with_bundle(UiBundle::<CustomBindingTypes>::new())?
        // Rendering
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
