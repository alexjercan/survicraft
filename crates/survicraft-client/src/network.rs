use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use lightyear::connection::identity::is_client;
use lightyear::netcode::Key;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use survicraft_protocol::{PROTOCOL_ID, SERVER_ADDR, get_client_id};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkPluginSet;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            startup.in_set(NetworkPluginSet).run_if(not(is_client)),
        );
    }
}

fn startup(mut commands: Commands) -> Result {
    // TODO: Change the Server ADDR
    info!("Starting client, connecting to server at {}", SERVER_ADDR);

    let auth = Authentication::Manual {
        server_addr: SERVER_ADDR,
        client_id: get_client_id(),
        private_key: Key::default(),
        protocol_id: PROTOCOL_ID,
    };
    let client = commands
        .spawn((
            Name::new("Client"),
            Client::default(),
            LocalAddr(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)),
            PeerAddr(SERVER_ADDR),
            Link::new(None),
            ReplicationReceiver::default(),
            NetcodeClient::new(auth, NetcodeConfig::default())?,
            UdpIo::default(),
        ))
        .id();

    commands.trigger_targets(Connect, client);

    Ok(())
}
