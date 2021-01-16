mod confirm;
mod home;
mod new_game;

pub use self::confirm::*;
pub use self::home::*;
pub use self::new_game::*;
use crate::resources::Wallpaper;
use crate::resources::WallpaperResource;
use crate::utils;
use amethyst::core::ecs::Join;
use amethyst::core::Parent;
use amethyst::ecs::prelude::Entity;
use amethyst::ecs::prelude::World;
use amethyst::ecs::prelude::WorldExt;
use amethyst::ui::UiImage;
use amethyst::ui::UiTransform;
use amethyst::ui::UiWidget;
use amethyst::ui::ToNativeWidget;
use amethyst::ui::UiTransformData;
use amethyst::ui::UiButtonData;
use amethyst::ui::UiButton;
use amethyst::ui::Anchor;
use amethyst::ui::Stretch;
use amethyst::ui::UiTextData;
use serde::Deserialize;
use serde::Serialize;
use crate::input::CustomBindingTypes;

// TODO: Cleanup

const WALLPAPER_ID: &str = "wallpaper";

pub trait UiState {
    fn set_visibility(&self, world: &World, is_visibility: bool) {
        if let Some(root) = self.get_root() {
            utils::set_entity_visibility(world, root, is_visibility);
        }

        if is_visibility {
            utils::ui::set_cursor_visibility(world, true);
        }
    }

    fn set_wallpaper(&self, world: &World, wallpaper: Wallpaper) {
        if let Some(root) = self.get_root() {
            if let Some(image) = world.read_resource::<WallpaperResource>().get(wallpaper) {
                for (entity, parent, transform) in (
                    &world.entities(),
                    &world.read_storage::<Parent>(),
                    &world.read_storage::<UiTransform>(),
                )
                    .join()
                {
                    if parent.entity == root && transform.id == WALLPAPER_ID {
                        if let Err(error) = world.write_storage::<UiImage>().insert(entity, image) {
                            log::error!(
                                "Failed to set {:?} for Entity({}). Details: {}",
                                wallpaper,
                                entity.id(),
                                error,
                            )
                        }

                        break;
                    }
                }
            }
        }
    }

    fn get_root(&self) -> Option<Entity>;
}

#[derive(Clone, Deserialize)]
pub enum CustomUi {
    Button {
        id: String,
        text: String,
        layout: Layout,
    },
    Space { // TODO: Make as container
        layout: Layout,
    },
    Label {
        text: String,
        layout: Layout,
        style: LabelStyle,
    },
    Root {
        id: String,
        children: Vec<UiWidget<CustomUi>>,
    },
}

#[derive(Deserialize, Serialize, Clone)]
pub enum Layout {
    Absolute {
        x: f32,
        y: f32,
        size_x: f32,
        size_y: f32,
    },
    Fill(f32),
}

#[derive(Deserialize, Serialize, Clone)]
pub enum LabelStyle {
    Big,
    Small,
}

impl CustomUi {
    pub fn get_fill(&self) -> Option<f32> {
        match self {
            Self::Button { layout: Layout::Fill(fill), .. } => {
                return Some(*fill);
            }
            Self::Space { layout: Layout::Fill(fill), .. } => {
                return Some(*fill);
            }
            Self::Label { layout: Layout::Fill(fill), .. } => {
                return Some(*fill);
            }
            _ => {
                return None;
            }
        }
    }

    pub fn set_layout(&mut self, new_layout: Layout) {
        match self {
            Self::Button { layout, .. } => {
                *layout = new_layout;
            }
            Self::Space { layout, .. } => {
                *layout = new_layout;
            }
            Self::Label { layout, .. } => {
                *layout = new_layout;
            }
            Self::Root { .. } => {}
        }
    }
}

impl ToNativeWidget for CustomUi {
    type PrefabData = ();

    fn to_native_widget(self, _: ()) -> (UiWidget<CustomUi>, Self::PrefabData) {
        match self {
            CustomUi::Button {
                id,
                text,
                layout,
            } => {
                let mut button = UiWidget::Button {
                    transform: UiTransformData::default(),
                    button: UiButtonData {
                        id: None,
                        text,
                        font_size: 26.0, // TODO: To constant
                        font: None, // TODO: Set `Option<AssetPrefab<FontAsset>>`
                        normal_text_color: [0.8, 0.8, 0.8, 1.0], // TODO: Use constant
                        normal_image: None,
                        hover_image: None,
                        hover_text_color: Some([0.6, 0.6, 0.6, 1.0]), // TODO: Use constant
                        press_image: None,
                        press_text_color: None,
                        hover_sound: None,
                        press_sound: None,
                        release_sound: None,
                    },
                };

                if let Some(transform) = button.transform_mut() {
                    transform.id = id;
                    transform.x = 0.35;
                    transform.width = 0.3; // TODO: To constant
                    transform.percent = true;
                    transform.anchor = Anchor::TopLeft;
                    transform.pivot = Anchor::TopLeft;

                    if let Layout::Absolute { y, size_y, ..} = layout {
                        transform.y = y;
                        transform.height = size_y;
                    }
                }

                return (button, ());
            }
            CustomUi::Label {
                text,
                layout,
                style,
            } => {
                let mut label = UiWidget::Label {
                    transform: UiTransformData::default(),
                    text: UiTextData {
                        text,
                        font_size: style.get_font_size(),
                        color: style.get_color(),
                        font: None, // TODO: Set `Option<AssetPrefab<FontAsset>>`
                        password: false,
                        line_mode: None,
                        align: None,
                        editable: None,
                    },
                };

                if let Some(transform) = label.transform_mut() {
                    transform.x = 0.0;
                    transform.width = 1.0;
                    transform.percent = true;
                    transform.opaque = false;
                    transform.anchor = Anchor::TopLeft;
                    transform.pivot = Anchor::TopLeft;

                    if let Layout::Absolute { y, size_y, ..} = layout {
                        transform.y = y;
                        transform.height = size_y;
                    }
                }

                return (label, ());
            }
            CustomUi::Root {
                id,
                mut children,
            } => {
                let mut to_fill = 0.0;

                for child in children.iter() {
                    if let UiWidget::Custom(child) = child {
                        if let Some(fill) = child.get_fill() {
                            to_fill += fill;
                        }
                    }
                }

                log::info!("Root(\"{}\") has {} cells to fill", id, to_fill); // TODO: Use debug

                let mut cell = 0.0; // TODO: Rename

                for child in children.iter_mut() {
                    if let UiWidget::Custom(child) = child {
                        if let Some(fill) = child.get_fill() {
                            let size = fill / to_fill;

                            child.set_layout(Layout::Absolute {
                                x: 0.0,
                                y: -cell,
                                size_x: 1.0,
                                size_y: size,
                            });

                            cell += size;
                        }
                    }
                }

                let mut wallpaper = UiWidget::<CustomUi>::Container {
                    transform: UiTransformData::default(),
                    background: None,
                    children: Vec::new(),
                };

                if let Some(transform) = wallpaper.transform_mut() {
                    transform.id = "wallpaper".to_string(); // TODO: FIx
                    transform.z = 0.5;
                    transform.width = 1.0;
                    transform.height = 1.0;
                    transform.opaque = false;
                }

                let mut list = UiWidget::<CustomUi>::Container {
                    transform: UiTransformData::default(),
                    background: None,
                    children,
                };

                if let Some(transform) = list.transform_mut() {
                    transform.width = 1.0;
                    transform.height = 1.0;
                    transform.opaque = false;
                    transform.stretch = Some(Stretch::XY {
                        x_margin: 0.0,
                        y_margin: 0.0,
                        keep_aspect_ratio: true,
                    });
                }

                let mut container = UiWidget::<CustomUi>::Container {
                    transform: UiTransformData::default(),
                    background: None, // TODO: Set to `Texture(Generate(Srgba(0.2, 0.2, 0.2, 0.8)))`
                    children: vec![list],
                };

                if let Some(transform) = container.transform_mut() {
                    transform.z = 1.0;
                    transform.opaque = false;
                    transform.stretch = Some(Stretch::XY {
                        x_margin: 0.0,
                        y_margin: 0.0,
                        keep_aspect_ratio: false,
                    });
                }

                let mut root = UiWidget::Container {
                    transform: UiTransformData::default(),
                    background: None,
                    children: vec![
                        wallpaper,
                        container,
                    ],
                };

                if let Some(transform) = root.transform_mut() {
                    transform.id = id;
                    transform.hidden = true;
                    transform.opaque = false;
                    transform.stretch = Some(Stretch::XY {
                        x_margin: 0.0,
                        y_margin: 0.0,
                        keep_aspect_ratio: false,
                    });
                }

                return (root, ());
            }
            CustomUi::Space { layout, .. } => {
                let mut spacer = UiWidget::<CustomUi>::Container {
                    transform: UiTransformData::default(),
                    background: None,
                    children: Vec::new(),
                };

                if let Some(transform) = spacer.transform_mut() {
                    // TODO: Make sure it transparent
                    transform.opaque = false;

                    if let Layout::Absolute { y, size_y, ..} = layout {
                        transform.y = y;
                        transform.height = size_y;
                    }
                }

                return (spacer, ());
            }
        }
    }
}

impl LabelStyle {
    pub fn get_font_size(&self) -> f32 {
        match self {
            Self::Big => {
                return 32.0; // TODO: To constant
            }
            Self::Small => {
                return 12.0; // TODO: To constant
            }
        }
    }

    pub fn get_color(&self) -> [f32; 4] {
        match self {
            Self::Big => {
                return [0.8, 0.8, 0.8, 1.0]; // TODO: To constant
            }
            Self::Small => {
                return [0.8, 0.8, 0.8, 0.3]; // TODO: To constant
            }
        }
    }
}
