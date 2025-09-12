use bevy::prelude::*;
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        server::{NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};
use survicraft_protocol::{
    component::{PlayerId, PlayerName}, message::{
        ClientChatMessage, ClientMetaMessage, MessageChannel, ServerChatMessage,
        ServerWelcomeMessage,
    }, PROTOCOL_ID, SERVER_ADDR, SERVER_REPLICATION_INTERVAL
};

/// Server component to mark the server entity.
/// Add this component to an entity to make it a server.
#[derive(Debug, Clone, Component)]
pub struct ServerListener;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServerPluginSet;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                server_listen,
                handle_new_connection,
                handle_new_client,
                handle_spawn_player,
                handle_chat_message,
            )
                .in_set(ServerPluginSet),
        );
    }
}

fn server_listen(
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

fn handle_new_connection(
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

fn handle_spawn_player(
    mut commands: Commands,
    mut q_receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<ClientMetaMessage>), Without<Client>>,
) {
    for (entity, RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            info!("Client {:?} set their name to {}", peer, message.username);

            commands.spawn((
                Name::new(format!("Player {}", message.username)),
                PlayerName(message.username.clone()),
                PlayerId(*peer),
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::Single(*peer)),
                InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(*peer)),
                ControlledBy {
                    owner: entity,
                    lifetime: Lifetime::default(),
                },
            ));
        }
    }
}

fn handle_chat_message(
    mut q_receiver: Query<&mut MessageReceiver<ClientChatMessage>, Without<Client>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) {
    for mut receiver in q_receiver.iter_mut() {
        for message in receiver.receive() {
            sender
                .send::<_, MessageChannel>(
                    &ServerChatMessage {
                        message: message.message.clone(),
                    },
                    server.clone(),
                    &NetworkTarget::All,
                )
                .unwrap_or_else(|e| {
                    // TODO: Handle the error properly
                    error!("Failed to send message: {:?}", e);
                });
        }
    }
}
