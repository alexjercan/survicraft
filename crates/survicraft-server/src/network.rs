use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear::{connection::identity::is_server, prelude::server::*};
use survicraft_protocol::{PROTOCOL_ID, SERVER_ADDR, SERVER_REPLICATION_INTERVAL};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkPluginSet;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            startup.in_set(NetworkPluginSet).run_if(not(is_server)),
        );
        app.add_systems(Update, handle_new_client.in_set(NetworkPluginSet));
    }
}

fn startup(mut commands: Commands) {
    info!("Starting server on {}", SERVER_ADDR);

    let server = commands
        .spawn((
            Name::new("Server"),
            NetcodeServer::new(NetcodeConfig::default().with_protocol_id(PROTOCOL_ID)),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Start, server);
}

fn handle_new_client(
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
