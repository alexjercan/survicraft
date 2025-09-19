use crate::protocol::prelude::*;
use bevy::prelude::*;
use lightyear::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ChatPluginSet;

pub(crate) struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, on_chat_message.in_set(ChatPluginSet));
    }
}

fn on_chat_message(
    mut q_receiver: Query<(&RemoteId, &mut MessageReceiver<ClientChatMessage>)>,
    mut ev_chat: EventWriter<ServerChatMessageEvent>,
) {
    for (RemoteId(peer), mut receiver) in q_receiver.iter_mut() {
        for message in receiver.receive() {
            debug!("Received chat message from {:?}: {}", peer, message.message);

            ev_chat.write(ServerChatMessageEvent {
                sender: *peer,
                message: message.message.clone(),
            });
        }
    }
}
