use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use survicraft_protocol::{PROTOCOL_ID, SERVER_ADDR, SERVER_REPLICATION_INTERVAL};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_observer(handle_new_client);
    }
}

fn startup(mut commands: Commands) {
    let server = commands
        .spawn((
            NetcodeServer::new(NetcodeConfig::default().with_protocol_id(PROTOCOL_ID)),
            LocalAddr(SERVER_ADDR),
            ServerUdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Start, server);
}

fn handle_new_client(trigger: Trigger<OnAdd, Connected>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .insert(ReplicationSender::new(
            SERVER_REPLICATION_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}
