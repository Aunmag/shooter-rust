use crate::resources::GameStatus;
use crate::resources::SpriteResource;
use crate::resources::WallpaperResource;
use crate::states::ui::HomeState;
use crate::states::ui::CustomUi;
use amethyst::assets::Completion;
use amethyst::assets::ProgressCounter;
use amethyst::prelude::*;
use amethyst::ui::UiCreator;
use amethyst::window::Window;

pub struct StartupState {
    progress: ProgressCounter,
}

impl StartupState {
    pub fn new() -> Self {
        return Self {
            progress: ProgressCounter::new(),
        };
    }
}

impl SimpleState for StartupState {
    fn on_start(&mut self, data: StateData<GameData>) {
        data.world.exec(|mut creator: UiCreator<'_, CustomUi>| {
            creator.create("ui/home_v2.ron", &mut self.progress);
            // creator.create("ui/confirm.ron", &mut self.progress);
            // creator.create("ui/home.ron", &mut self.progress);
            // creator.create("ui/new_game.ron", &mut self.progress);
        });

        data.world
            .insert(SpriteResource::new(&data.world, &mut self.progress));

        data.world
            .insert(WallpaperResource::new(&data.world, &mut self.progress));
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        match self.progress.complete() {
            Completion::Loading => {
                return Trans::None;
            }
            Completion::Complete => {
                // enable_fullscreen_mode(&data.world);
                complete_startup(&data.world);
                return Trans::Switch(Box::new(HomeState::new(true)));
            }
            Completion::Failed => {
                log::error!("Failed to load assets");
                return Trans::Quit;
            }
        }
    }
}

fn enable_fullscreen_mode(world: &World) {
    let window = world.read_resource::<Window>();

    #[allow(clippy::never_loop)]
    for monitor in window.get_available_monitors() {
        window.set_fullscreen(Some(monitor));
        break;
    }
}

fn complete_startup(world: &World) {
    world
        .read_resource::<SpriteResource>()
        .resize_sprites(world);

    world.write_resource::<GameStatus>().is_loaded = true;
}
