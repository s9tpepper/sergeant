use std::sync::mpsc::Sender;

use crate::{
    channel::ChannelMessages,
    twitch::eventsub::deserialization::{NotificationEvent, NotificationPayload},
};

use super::send_to_channels;

pub fn channel_chat_clear_user_messages(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
) -> anyhow::Result<()> {
    let NotificationEvent::ChannelChatClearUserMessages { target_user_name, .. } = &*payload.event else {
        return Ok(());
    };

    let channel_message = ChannelMessages::ClearMessagesByUser {
        target_user_name: target_user_name.clone(),
    };

    send_to_channels(
        channel_message,
        tui_tx,
        websocket_tx,
        "channel_chat_clear_user_messages",
    )
}
