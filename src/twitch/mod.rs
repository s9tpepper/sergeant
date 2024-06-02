use serde::{Deserialize, Serialize};

use crate::eventsub::structs::SubscriptionEvent;

use self::{announcements::Announcement, parse::TwitchMessage, pubsub::MessageData};

pub mod announcements;
pub mod api;
pub mod irc;
pub mod message;
pub mod messages;
pub mod parse;
pub mod pubsub;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ChannelMessages {
    MessageData(MessageData),
    Announcement(Announcement),
    TwitchMessage(TwitchMessage),
    Notifications(Box<SubscriptionEvent>),
}

#[cfg(test)]
pub mod fixtures;
