use bincode::Options;
use serde::Deserialize;
use serde::Serialize;
use std::net::SocketAddr;

pub const MESSAGE_SIZE_MAX: usize = std::mem::size_of::<Message>();

pub type MessageResource = Vec<(MessageReceiver, Message)>;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Response {
        message_id: u16,
    },
    Greeting {
        id: u16,
    },
    ClientInput {
        id: u16,
        actions: u8,
        direction: f32,
    },
    ClientInputDirection {
        id: u16,
        direction: f32,
    },
    ActorSpawn {
        id: u16,
        public_id: u16,
        x: f32,
        y: f32,
        direction: f32,
    },
    ActorGrant {
        id: u16,
        public_id: u16,
    },
    TransformSync {
        id: u16,
        public_id: u16,
        x: f32,
        y: f32,
        direction: f32,
    },
}

pub enum MessageReceiver {
    Only(SocketAddr),
    Every,
    Except(SocketAddr),
}

impl Message {
    pub fn encode(&self) -> Vec<u8> {
        // I use unwrap here since I suppose there's nothing to worry about
        #[allow(clippy::unwrap_used)]
        return bincode::DefaultOptions::new()
            .with_varint_encoding()
            .serialize(self)
            .unwrap();
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, bincode::Error> {
        return bincode::DefaultOptions::new()
            .with_varint_encoding()
            .deserialize(encoded);
    }

    pub fn set_id(&mut self, id_new: u16) {
        #[allow(clippy::match_same_arms)]
        match *self {
            Self::Response { .. } => {}
            Self::Greeting { ref mut id } => {
                *id = id_new;
            }
            Self::ClientInput { ref mut id, .. } => {
                *id = id_new;
            }
            Self::ClientInputDirection { ref mut id, .. } => {
                *id = id_new;
            }
            Self::ActorSpawn { ref mut id, .. } => {
                *id = id_new;
            }
            Self::ActorGrant { ref mut id, .. } => {
                *id = id_new;
            }
            Self::TransformSync { ref mut id, .. } => {
                *id = id_new;
            }
        }
    }

    pub fn get_id(&self) -> Option<u16> {
        #[allow(clippy::match_same_arms)]
        return match *self {
            Self::Response { .. } => None,
            Self::Greeting { id } => Some(id),
            Self::ClientInput { id, .. } => Some(id),
            Self::ClientInputDirection { id, .. } => Some(id),
            Self::ActorSpawn { id, .. } => Some(id),
            Self::ActorGrant { id, .. } => Some(id),
            Self::TransformSync { id, .. } => Some(id),
        };
    }

    pub fn has_id(&self) -> bool {
        if let Self::Response { .. } = *self {
            return false;
        } else {
            return true;
        }
    }
}