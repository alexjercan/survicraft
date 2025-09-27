use std::fmt::Debug;

use bevy::prelude::*;
use lightyear::{connection::host::HostClient, prelude::*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod prelude {
    pub use super::AppEventExt;
    pub use super::{FromClient, ToClient};
}

#[derive(Serialize, Deserialize)]
struct ServerMessageEvent<E> {
    message: E,
}

#[derive(Serialize, Deserialize)]
struct ClientMessageEvent<E> {
    message: E,
}

/// Event sent from server to a specific client or all clients. When you write an event of this
/// type, it will send a message from the server to the specified client(s).
#[derive(Event, Debug, Clone)]
pub struct ToClient<E: Event> {
    pub target: NetworkTarget,
    pub event: E,
}

/// Event sent from a specific client to the server. When you write an event of this type, it will
/// send a message from the client to the server.
#[derive(Event, Debug, Clone)]
pub struct FromClient<E: Event> {
    pub owner: Entity,
    pub peer: PeerId,
    pub event: E,
}

pub trait AppEventExt {
    /// Register a new app event, with replication from client to server
    fn add_client_event<E: Event + Serialize + DeserializeOwned + Clone + Debug, C: Channel>(
        &mut self,
    ) -> &mut Self;

    /// Register a new app event, with replication from server to client
    fn add_server_event<E: Event + Serialize + DeserializeOwned + Clone + Debug, C: Channel>(
        &mut self,
    ) -> &mut Self;
}

impl AppEventExt for App {
    fn add_client_event<E: Event + Serialize + DeserializeOwned + Clone + Debug, C: Channel>(
        &mut self,
    ) -> &mut Self {
        self.add_event::<E>();
        self.add_event::<FromClient<E>>();
        self.add_message::<ClientMessageEvent<E>>()
            .add_direction(NetworkDirection::ClientToServer);
        self.add_systems(Update, send_event_to_server::<E, C>);
        self.add_systems(Update, receive_event_from_client::<E>);
        self
    }

    fn add_server_event<E: Event + Serialize + DeserializeOwned + Clone + Debug, C: Channel>(
        &mut self,
    ) -> &mut Self {
        self.add_event::<E>();
        self.add_event::<ToClient<E>>();
        self.add_message::<ServerMessageEvent<E>>()
            .add_direction(NetworkDirection::ServerToClient);
        self.add_systems(Update, send_event_to_client::<E, C>);
        self.add_systems(Update, receive_event_from_server::<E>);
        self
    }
}

fn send_event_to_client<E: Event + Clone + Debug, C: Channel>(
    mut ev_server: EventReader<ToClient<E>>,
    mut sender: ServerMultiMessageSender,
    server: Single<&Server>,
) -> Result {
    for ToClient { target, event } in ev_server.read() {
        trace!("Sending event to client ({:?}): {:?}", target, event);

        sender.send::<_, C>(
            &ServerMessageEvent {
                message: event.clone(),
            },
            server.clone(),
            target,
        )?;
    }

    Ok(())
}

fn receive_event_from_server<E: Event + Debug>(
    receiver: Single<
        &mut MessageReceiver<ServerMessageEvent<E>>,
        Or<(With<Client>, With<HostClient>)>,
    >,
    mut ev_client: EventWriter<E>,
) {
    let mut receiver = receiver.into_inner();

    for ServerMessageEvent { message } in receiver.receive() {
        trace!("Received event from server: {:?}", message);

        ev_client.write(message);
    }
}

fn send_event_to_server<E: Event + Clone + Debug, C: Channel>(
    mut ev_client: EventReader<E>,
    sender: Single<
        (&RemoteId, &mut MessageSender<ClientMessageEvent<E>>),
        Or<(With<Client>, With<HostClient>)>,
    >,
) -> Result {
    let (RemoteId(_), mut sender) = sender.into_inner();

    for event in ev_client.read() {
        trace!("Sending event to server: {:?}", event);

        sender.send::<C>(ClientMessageEvent {
            message: event.clone(),
        });
    }

    Ok(())
}

fn receive_event_from_client<E: Event + Debug>(
    mut q_receiver: Query<(
        Entity,
        &RemoteId,
        &mut MessageReceiver<ClientMessageEvent<E>>,
    )>,
    mut ev_server: EventWriter<FromClient<E>>,
    _: Single<&Server>,
) {
    for (entity, RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for ClientMessageEvent { message } in receiver.receive() {
            trace!("Received event from client {:?}: {:?}", peer, message);

            ev_server.write(FromClient {
                owner: entity,
                peer: *peer,
                event: message,
            });
        }
    }
}
