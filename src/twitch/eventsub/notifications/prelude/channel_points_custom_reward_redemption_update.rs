use std::sync::{mpsc::Sender, Arc};

use crate::{channel::ChannelMessages, twitch::eventsub::deserialization::NotificationPayload};

pub fn channel_points_custom_reward_redemption_update(
    _payload: &NotificationPayload,
    _tui_tx: &Sender<ChannelMessages>,
    _websocket_tx: &Sender<ChannelMessages>,
    _oauth_token: &Arc<String>,
    _client_id: &Arc<String>,
) {
    // TODO: figure out how to handle the update event
}
