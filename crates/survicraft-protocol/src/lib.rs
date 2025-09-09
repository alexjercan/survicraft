use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, time::Duration};

use bevy::prelude::*;

pub mod component;
pub mod input;
pub mod message;

pub const FIXED_TIMESTEP_HZ: f64 = 64.0;

pub const SERVER_REPLICATION_INTERVAL: Duration = Duration::from_millis(100);

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        component::register_components(app);
        message::register_messages(app);
        input::register_input(app);
    }
}
