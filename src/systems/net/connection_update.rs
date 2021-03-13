use crate::resources::NetResource;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::System;
use amethyst::ecs::prelude::SystemData;
use amethyst::shred::WriteExpect;
use amethyst::ecs::prelude::Write;
use crate::resources::DebugTimer;

#[derive(SystemDesc)]
pub struct ConnectionUpdateSystem;

impl<'a> System<'a> for ConnectionUpdateSystem {
    type SystemData = (WriteExpect<'a, NetResource>, Write<'a, DebugTimer>);

    fn run(&mut self, (mut net, mut dt): Self::SystemData) {
        let s = dt.start();
        net.update_connections();
        dt.end("ConnectionUpdate", s);
    }
}
