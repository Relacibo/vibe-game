// This file handles network communication for the multiplayer game.
// It includes functions for sending and receiving player events over the network.

use bevy::prelude::*;
use bincode::{
    config::standard,
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
    Decode, Encode,
};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};

use super::player::Player;

#[derive(Event, Serialize, Deserialize)]
pub struct PlayerEvent {
    pub player_id: u32,
    pub position: Vec2,
}

// Manuelle Implementierung für bincode v2
impl Encode for PlayerEvent {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.player_id.encode(encoder)?;
        self.position.x.encode(encoder)?;
        self.position.y.encode(encoder)?;
        Ok(())
    }
}

impl<C> Decode<C> for PlayerEvent {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let player_id = u32::decode(decoder)?;
        let x = f32::decode(decoder)?;
        let y = f32::decode(decoder)?;
        Ok(PlayerEvent {
            player_id,
            position: Vec2::new(x, y),
        })
    }
}

#[derive(Resource)]
pub struct MySocket(pub UdpSocket);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MySocket(
            UdpSocket::bind("127.0.0.1:34254").expect("Could not bind socket"),
        ))
        .add_event::<PlayerEvent>()
        .add_systems(Startup, setup_network)
        .add_systems(Update, (send_player_events, receive_player_events));
    }
}

fn setup_network(_socket: Res<MySocket>) {
    // Setup code for the network can be added here
}

fn send_player_events(socket: Res<MySocket>, player_query: Query<(&Player, &Transform, Entity)>) {
    for (_player, transform, entity) in player_query.iter() {
        let event = PlayerEvent {
            player_id: entity.index(),
            position: Vec2::new(transform.translation.x, transform.translation.y),
        };
        let serialized_event =
            bincode::encode_to_vec(&event, standard()).expect("Failed to encode event");
        socket
            .0
            .send_to(&serialized_event, "127.0.0.1:34255")
            .expect("Failed to send event");
    }
}

fn receive_player_events(mut socket: ResMut<MySocket>, mut event_writer: EventWriter<PlayerEvent>) {
    let mut buf = [0; 1024];
    if let Ok((size, _)) = socket.0.recv_from(&mut buf) {
        if let Ok((event, _)) = bincode::decode_from_slice(&buf[..size], standard()) {
            event_writer.write(event);
        }
    }
}
