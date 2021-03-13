use crate::components::Actor;
use crate::components::ActorActions;
use crate::components::Weapon;
use crate::resources::{EntityMap, DebugTimer};
use crate::resources::GameTask;
use crate::resources::GameTaskResource;
use crate::resources::Message;
use crate::resources::MouseInput;
use crate::resources::NetResource;
use crate::states::ui::HomeState;
use crate::systems::net::ConnectionUpdateSystem;
use crate::systems::net::InputSendSystem;
use crate::systems::net::InterpolationSystem;
use crate::systems::net::MessageReceiveSystem;
use crate::systems::net::PositionUpdateSendSystem;
use crate::systems::net::PositionUpdateSystem;
use crate::systems::ActorSystem;
use crate::systems::AiSystem;
use crate::systems::CameraSystem;
use crate::systems::CollisionSystem;
use crate::systems::PlayerSystem;
use crate::systems::ProjectileSystem;
use crate::systems::TerrainSystem;
use crate::systems::WeaponSystem;
use crate::utils;
use crate::utils::TakeContent;
use amethyst::controls::HideCursor;
use amethyst::core::transform::Transform;
use amethyst::core::ArcThreadPool;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::World;
use amethyst::ecs::Dispatcher;
use amethyst::ecs::DispatcherBuilder;
use amethyst::ecs::Entity;
use amethyst::input::is_key_down;
use amethyst::prelude::*;
use amethyst::renderer::debug_drawing::DebugLines;
use amethyst::winit::DeviceEvent;
use amethyst::winit::ElementState;
use amethyst::winit::Event;
use amethyst::winit::MouseButton;
use amethyst::winit::VirtualKeyCode;
use amethyst::winit::WindowEvent;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct GameState<'a, 'b> {
    game_type: GameType,
    root: Option<Entity>,
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

pub enum GameType {
    Host(u16),
    Join(SocketAddr),
}

impl GameState<'_, '_> {
    pub fn new(game_type: GameType) -> Self {
        return Self {
            game_type,
            root: None,
            dispatcher: None,
        };
    }

    fn init_dispatcher(&mut self, world: &mut World) {
        let mut builder = DispatcherBuilder::new();

        match self.game_type {
            GameType::Host(..) => {
                builder.add(AiSystem::new(), "Ai", &[]);
                builder.add(PlayerSystem, "Player", &[]);
                builder.add(ActorSystem, "Actor", &["Ai", "Player"]);
                builder.add(CollisionSystem::new(), "Collision", &["Actor"]);
                builder.add(WeaponSystem::new(), "Weapon", &["Collision"]);
                builder.add(ProjectileSystem, "Projectile", &["Collision"]);
                builder.add(PositionUpdateSendSystem::new(), "PositionUpdateSend", &["Collision"]);
                builder.add(ConnectionUpdateSystem, "ConnectionUpdate", &[]);
                builder.add(CameraSystem::new(), "Camera", &[]);
                builder.add(TerrainSystem, "Terrain", &[]);
                builder.add(MessageReceiveSystem::new(true), "MessageReceive", &[]);
            }
            GameType::Join(..) => {
                builder.add(InterpolationSystem, "Interpolation", &[]);
                builder.add(PlayerSystem, "Player", &[]);
                builder.add(ActorSystem, "Actor", &["Player", "Interpolation"]);
                builder.add(InputSendSystem::new(), "InputSend", &["Player", "Actor"]);
                builder.add(MessageReceiveSystem::new(false), "MessageReceive", &[]);
                builder.add(PositionUpdateSystem, "PositionUpdate", &["MessageReceive", "Actor"]);
                builder.add(ProjectileSystem, "Projectile", &["Actor"]);
                builder.add(ConnectionUpdateSystem, "ConnectionUpdate", &[]);
                builder.add(CameraSystem::new(), "Camera", &[]);
                builder.add(TerrainSystem, "Terrain", &[]);
            }
        }

        let mut dispatcher = builder
            .with_pool(Arc::clone(&world.read_resource::<ArcThreadPool>()))
            .build();

        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);
    }

    #[allow(clippy::unused_self)]
    fn init_resources(&self, world: &mut World) {
        world.register::<Weapon>();
        world.insert(DebugLines::new());
        world.insert(EntityMap::new());
        world.insert(GameTaskResource::new());

        #[allow(clippy::unwrap_used)] // TODO: Resolve
        match self.game_type {
            GameType::Host(port) => {
                world.insert(NetResource::new_as_server(port).unwrap());
            }
            GameType::Join(address) => {
                world.insert(NetResource::new_as_client(address).unwrap());
            }
        }
    }

    fn init_world_entities(&mut self, world: &mut World) {
        let root = world.create_entity().build();
        self.root.replace(root);

        if self.is_host() {
            let mut tasks = world.write_resource::<GameTaskResource>();
            let external_id = world.write_resource::<EntityMap>().generate_external_id();

            tasks.push(GameTask::ActorSpawn {
                external_id,
                x: 0.0,
                y: 0.0,
                direction: 0.0,
            });

            tasks.push(GameTask::ActorGrant { external_id });

            for i in 0..2 {
                let external_id = world.write_resource::<EntityMap>().generate_external_id();

                tasks.push(GameTask::ActorSpawn {
                    external_id,
                    x: 5.0 * (0.5 - i as f32),
                    y: 0.0,
                    direction: 0.0,
                });

                tasks.push(GameTask::ActorAiSet { external_id });
            }
        }

        utils::world::create_terrain(world, root);
        utils::world_decorations::create_decorations(world, root);
    }

    #[allow(clippy::unused_self)]
    fn reset_input(&self, world: &World) {
        let mut mouse_input = world.write_resource::<MouseInput>();
        mouse_input.delta_x = 0.0;
        mouse_input.delta_y = 0.0;
    }

    fn on_task(&mut self, world: &mut World, task: GameTask) {
        match task {
            GameTask::ClientGreet(address) => {
                self.on_task_client_greet(world, address);
            }
            GameTask::ActorSpawn {
                external_id,
                x,
                y,
                direction,
            } => {
                self.on_task_actor_spawn(world, external_id, x, y, direction);
            }
            GameTask::ActorGrant { external_id } => {
                self.on_task_actor_grant(world, external_id);
            }
            GameTask::ActorAiSet { external_id } => {
                self.on_task_actor_ai_set(world, external_id);
            }
            GameTask::ActorAction {
                external_id,
                actions,
                direction,
            } => {
                self.on_task_actor_action(world, external_id, Some(actions), direction);
            }
            GameTask::ActorTurn {
                external_id,
                direction,
            } => {
                self.on_task_actor_action(world, external_id, None, direction);
            }
            GameTask::ProjectileSpawn {
                x,
                y,
                velocity_x,
                velocity_y,
                acceleration_factor,
                shooter_id,
            } => {
                self.on_task_projectile_spawn(
                    world,
                    x,
                    y,
                    velocity_x,
                    velocity_y,
                    acceleration_factor,
                    shooter_id,
                );
            }
            GameTask::ProjectileHit {
                entity,
                force_x,
                force_y,
            } => {
                self.on_task_projectile_hit(world, entity, force_x, force_y);
            }
            GameTask::MessageSent {
                message,
                address_filter,
            } => {
                self.on_task_message_send(world, message, address_filter);
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn on_task_client_greet(&self, world: &mut World, address: SocketAddr) {
        let mut entity_map = world.write_resource::<EntityMap>();
        let mut net = world.write_resource::<NetResource>();

        for (entity, _, transform) in (
            &world.entities(),
            &world.read_storage::<Actor>(),
            &world.read_storage::<Transform>(),
        )
            .join()
        {
            if let Some(external_id) = entity_map.get_external_id(entity) {
                net.send_to(
                    &address,
                    Message::ActorSpawn {
                        id: 0,
                        external_id,
                        x: transform.translation().x,
                        y: transform.translation().y,
                        direction: transform.euler_angles().2,
                    },
                );
            }
        }

        let external_id = entity_map.generate_external_id();
        let mut tasks = world.write_resource::<GameTaskResource>();

        tasks.push(GameTask::ActorSpawn {
            external_id,
            x: 0.0,
            y: 0.0,
            direction: 0.0,
        });

        tasks.push(GameTask::MessageSent {
            message: Message::ActorGrant { id: 0, external_id },
            address_filter: Some(address),
        });

        net.attach_external_id(&address, external_id);
    }

    fn on_task_actor_spawn(
        &self,
        world: &mut World,
        external_id: u16,
        x: f32,
        y: f32,
        direction: f32,
    ) {
        if let Some(root) = self.root {
            utils::world::create_actor(
                world,
                root,
                Some(external_id),
                x,
                y,
                direction,
                false,
                &self.game_type,
            );

            if let GameType::Host(..) = self.game_type {
                world
                    .write_resource::<NetResource>()
                    .send_to_all(Message::ActorSpawn {
                        id: 0,
                        external_id,
                        x,
                        y,
                        direction,
                    });
            }
        }
    }

    fn on_task_actor_grant(&mut self, world: &mut World, external_id: u16) {
        if let Some(root) = self.root {
            if let Some(actor) = utils::world::get_entity(world, external_id) {
                utils::world::grant_played_actor(world, root, actor, &self.game_type);
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn on_task_actor_ai_set(&self, world: &World, external_id: u16) {
        if let Some(actor) = utils::world::get_entity(world, external_id) {
            utils::world::set_actor_ai(world, actor);
        }
    }

    #[allow(clippy::unused_self)]
    fn on_task_actor_action(
        &self,
        world: &World,
        external_id: u16,
        actions: Option<ActorActions>,
        direction: f32,
    ) {
        if let Some(entity) = utils::world::get_entity(world, external_id) {
            if let Some(actions) = actions {
                if let Some(actor) = world.write_storage::<Actor>().get_mut(entity) {
                    actor.actions = actions;
                }
            }

            if let Some(transform) = world.write_storage::<Transform>().get_mut(entity) {
                transform.set_rotation_2d(direction);
            }
        }
    }

    fn on_task_projectile_spawn(
        &self,
        world: &mut World,
        x: f32,
        y: f32,
        velocity_x: f32,
        velocity_y: f32,
        acceleration_factor: f32,
        shooter_id: Option<u16>,
    ) {
        if let Some(root) = self.root {
            if let GameType::Host(..) = self.game_type {
                world
                    .write_resource::<NetResource>()
                    .send_to_all(Message::ProjectileSpawn {
                        id: 0,
                        x,
                        y,
                        velocity_x,
                        velocity_y,
                        acceleration_factor,
                        shooter_id,
                    });
            }

            utils::world::create_projectile(
                world,
                root,
                x,
                y,
                velocity_x,
                velocity_y,
                acceleration_factor,
                shooter_id,
            );
        }
    }

    #[allow(clippy::unused_self)]
    fn on_task_projectile_hit(
        &self,
        world: &mut World,
        entity: Entity,
        force_x: f32,
        force_y: f32,
    ) {
        if let Some(transform) = world.write_storage::<Transform>().get_mut(entity) {
            let translation = transform.translation_mut();
            translation.x += force_x;
            translation.y += force_y;
        }
    }

    #[allow(clippy::unused_self)]
    fn on_task_message_send(
        &self,
        world: &World,
        message: Message,
        address_filter: Option<SocketAddr>,
    ) {
        let mut net = world.write_resource::<NetResource>();

        if let Some(address) = address_filter {
            net.send_to(&address, message);
        } else {
            net.send_to_all(message);
        }
    }

    fn is_host(&self) -> bool {
        return match self.game_type {
            GameType::Host(..) => true,
            GameType::Join(..) => false,
        };
    }
}

impl<'a, 'b> SimpleState for GameState<'a, 'b> {
    fn on_start(&mut self, mut data: StateData<GameData>) {
        self.init_dispatcher(&mut data.world);
        self.init_resources(&mut data.world);
        self.init_world_entities(&mut data.world);
        self.reset_input(&data.world);
        utils::ui::set_cursor_visibility(&data.world, false);
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(root) = self.root.take() {
            if let Err(error) = data.world.delete_entity(root) {
                log::error!("Failed to delete the root entity. Details: {}", error);
            }
        }
    }

    fn on_resume(&mut self, data: StateData<GameData>) {
        utils::ui::set_cursor_visibility(&data.world, false);
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        if let Some(dispatcher) = self.dispatcher.as_mut() {
            dispatcher.dispatch(data.world);
        }

        loop {
            let tasks = data.world.fetch_mut::<GameTaskResource>().take_content();

            if tasks.is_empty() {
                break;
            }

            for task in tasks {
                self.on_task(&mut data.world, task);
            }
        }

        self.reset_input(&data.world);

        return Trans::None;
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            let mut cursor = data.world.write_resource::<HideCursor>();

            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if cursor.hide {
                        let mut mouse_input = data.world.write_resource::<MouseInput>();

                        #[allow(clippy::cast_possible_truncation)]
                        {
                            mouse_input.delta_x += delta.0 as f32;
                            mouse_input.delta_y += delta.1 as f32;
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    cursor.hide = true;
                }
                _ => {}
            }

            if is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Push(Box::new(HomeState::new(false)));
            }

            if cursor.hide && is_key_down(&event, VirtualKeyCode::Tab) {
                cursor.hide = false;
            }

            // 74 FPS, 19 FPS
            if is_key_down(&event, VirtualKeyCode::Key1) {
                let mut tasks = data.world.write_resource::<GameTaskResource>();
                let mut entity_map = data.world.write_resource::<EntityMap>();

                for _ in 0..2000 {
                    tasks.push(GameTask::ActorSpawn {
                        external_id: entity_map.generate_external_id(),
                        position: Position::default(),
                    });
                }
            }

            if is_key_down(&event, VirtualKeyCode::Key2) {
                let mut tasks = data.world.write_resource::<GameTaskResource>();
                let mut entity_map = data.world.write_resource::<EntityMap>();

                for _ in 0..500 {
                    tasks.push(GameTask::ActorSpawn {
                        external_id: entity_map.generate_external_id(),
                        position: Position::default(),
                    });
                }
            }
        }

        return Trans::None;
    }
}
