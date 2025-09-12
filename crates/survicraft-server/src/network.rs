//! The network plugin handles the server listener

use bevy::prelude::*;
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        server::{NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};
use survicraft_protocol::{
    PROTOCOL_ID, SERVER_ADDR, SERVER_REPLICATION_INTERVAL,
    message::{MessageChannel, ServerWelcomeMessage},
};

/// Server component to mark the server entity.
/// Add this component to an entity to make it a server.
#[derive(Debug, Clone, Component)]
pub struct ServerListener;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NetworkPluginSet;

pub(crate) struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_server_listener_added,
                on_new_client_connection,
                handle_new_client,
            )
                .in_set(NetworkPluginSet),
        );
    }
}

fn on_server_listener_added(
    mut commands: Commands,
    listener: Single<(Entity, &ServerListener), Added<ServerListener>>,
) {
    info!("Starting server on {}", SERVER_ADDR);
    let (entity, _) = listener.into_inner();

    let server = commands
        .entity(entity)
        .insert((
            Name::new("Server"),
            NetcodeServer::new(NetcodeConfig::default().with_protocol_id(PROTOCOL_ID)),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Start, server);
}

fn on_new_client_connection(
    mut commands: Commands,
    q_connected: Query<Entity, (Added<Connected>, Without<Client>)>,
) {
    for entity in q_connected.iter() {
        info!("New client connected: {:?}", entity);

        commands.entity(entity).insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
    }
}

fn handle_new_client(
    q_connected: Query<(Entity, &RemoteId), (Added<Connected>, Without<Client>)>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    for (_, RemoteId(peer)) in q_connected.iter() {
        sender
            .send::<_, MessageChannel>(
                &ServerWelcomeMessage,
                server.clone(),
                &NetworkTarget::Single(*peer),
            )
            .unwrap_or_else(|e| {
                // TODO: Handle the error properly
                error!("Failed to send message: {:?}", e);
            });
    }
}
