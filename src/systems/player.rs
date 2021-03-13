use crate::components::Actor;
use crate::components::ActorActions;
use crate::components::Player;
use crate::input::ActionBinding;
use crate::input::AxisBinding;
use crate::input::CustomBindingTypes;
use crate::resources::MouseInput;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::Join;
use amethyst::ecs::prelude::Read;
use amethyst::ecs::prelude::ReadStorage;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::ecs::prelude::WriteStorage;
use amethyst::input::InputHandler;
use std::f32::consts::TAU;
use amethyst::ecs::prelude::Write;
use crate::resources::DebugTimer;

const ROTATION_SENSITIVITY: f32 = 0.01;

#[derive(SystemDesc)]
pub struct PlayerSystem;

impl<'a> System<'a> for PlayerSystem {
    type SystemData = (
        Read<'a, InputHandler<CustomBindingTypes>>,
        Read<'a, MouseInput>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Actor>,
        Write<'a, DebugTimer>,
    );

    fn run(&mut self, (input, input_mouse, players, mut actors, mut dt): Self::SystemData) {
        let s = dt.start();
        let rotation = (input_mouse.delta_x * ROTATION_SENSITIVITY) % TAU;

        for (actor, _) in (&mut actors, &players).join() {
            actor.rotation = rotation;

            apply_movement_input(
                &mut actor.actions,
                ActorActions::MOVEMENT_FORWARD,
                ActorActions::MOVEMENT_BACKWARD,
                input
                    .axis_value(&AxisBinding::MovementForward)
                    .unwrap_or(0.0),
            );

            apply_movement_input(
                &mut actor.actions,
                ActorActions::MOVEMENT_RIGHTWARD,
                ActorActions::MOVEMENT_LEFTWARD,
                input.axis_value(&AxisBinding::MovementAside).unwrap_or(0.0),
            );

            actor.actions.set(
                ActorActions::ATTACK,
                input
                    .action_is_down(&ActionBinding::Attack)
                    .unwrap_or(false),
            );
        }
        dt.end("Player", s);
    }
}

fn apply_movement_input(actions: &mut ActorActions, a: ActorActions, b: ActorActions, ratio: f32) {
    if ratio > 0.0 {
        *actions |= a;
        *actions -= b;
    } else if ratio < 0.0 {
        *actions -= a;
        *actions |= b;
    } else {
        *actions -= a;
        *actions -= b;
    }
}
