//! The network plugin handles the server listener

use crate::{helpers::prelude::*, protocol::prelude::*};
use bevy::prelude::*;
use lightyear::{
    netcode::NetcodeServer,
    prelude::{
        server::{ClientOf, NetcodeConfig, ServerUdpIo, Start},
        *,
    },
};

/// Server component to mark the server entity.
/// Add this component to an entity to make it a server.
#[derive(Debug, Clone, Component)]
pub struct ServerListener;

pub(crate) struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_server_listener_added);
        app.add_observer(on_new_client);
        app.add_observer(on_new_connection);

        app.add_systems(FixedUpdate, on_client_metadata_message);
    }
}

fn on_server_listener_added(trigger: Trigger<OnAdd, ServerListener>, mut commands: Commands) {
    info!("Starting server on {}", SERVER_ADDR);

    let server = commands
        .entity(trigger.target())
        .insert((
            Name::new("Server"),
            NetcodeServer::new(NetcodeConfig::default().with_protocol_id(PROTOCOL_ID)),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Start, server);
}

fn on_new_client(
    trigger: Trigger<OnAdd, LinkOf>,
    mut commands: Commands,
    _server: Single<&Server>,
) {
    info!("New client connected: {:?}", trigger.target());

    commands
        .entity(trigger.target())
        .insert(Name::new("Client"))
        .insert(ReplicationReceiver::default())
        .insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

fn on_new_connection(
    trigger: Trigger<OnAdd, Connected>,
    q_connected: Query<&RemoteId, With<ClientOf>>,
    mut ev_welcome: EventWriter<ServerWelcomeEvent>,
    world_seed: Res<TerrainGenerationSeed>,
    _: Single<&Server>,
) -> Result {
    info!("New connection established: {:?}", trigger.target());

    let entity = trigger.target();
    let RemoteId(peer) = q_connected.get(entity)?;
    let welcome = ServerWelcomeEvent { peer: *peer, seed: **world_seed };
    debug!("Sending welcome message to {:?}: {:?}", peer, welcome);

    ev_welcome.write(welcome);

    Ok(())
}

fn on_client_metadata_message(
    mut commands: Commands,
    mut q_receiver: Query<(&RemoteId, &mut MessageReceiver<ClientMetaMessage>)>,
) {
    for (RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            debug!("Spawn player metadata for peer {:?}: {:?}", peer, message);

            commands.spawn((
                Name::new("PlayerMetadata"),
                PlayerId(*peer),
                PlayerMetadata {
                    username: message.username.clone(),
                },
                Replicate::to_clients(NetworkTarget::All),
            ));
        }
    }
}
