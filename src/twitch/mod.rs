use self::{announcements::Announcement, parse::TwitchMessage, pubsub::MessageData};

pub mod announcements;
pub mod irc;
pub mod message;
pub mod messages;
pub mod parse;
pub mod pubsub;

#[derive(Debug)]
pub enum ChannelMessages {
    MessageData(MessageData),
    Announcement(Announcement),
    TwitchMessage(TwitchMessage),
}

#[cfg(test)]
pub mod fixtures;
