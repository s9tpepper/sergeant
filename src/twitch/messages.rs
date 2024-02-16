use hex_rgb::*;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::io::Cursor;

use base64::prelude::*;
use irc::client::prelude::Message;
use irc::proto::Command;

type AsyncResult<T> = Result<T, Box<dyn Error>>;

pub struct Badges {
    broadcaster: bool,
    premium: bool,
    no_audio: bool,
}

pub struct Emote {
    id: String,
    start: usize,
    end: usize,
    url: String,
    name: String,
}

pub struct TwitchMessage {
    pub badges: Badges,
    pub emotes: Vec<Emote>,
    pub nickname: Option<String>,
    pub display_name: Option<String>,
    pub first_msg: Option<bool>,
    pub returning_chatter: Option<bool>,
    pub subscriber: Option<bool>,
    pub moderator: Option<bool>,
    pub message: Option<String>,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BadgeVersion {
    // id: String,
    // title: String,
    // description: String,
    // click_action: String,
    // click_url: String,
    image_url_1x: String,
    // image_url_2x: String,
    // image_url_4x: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BadgeItem {
    set_id: String,
    versions: Vec<BadgeVersion>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TwitchApiResponse<T> {
    pub data: T,
}

pub async fn get_badges(token: &str, client_id: &String) -> AsyncResult<Vec<BadgeItem>> {
    // Global badges: https://api.twitch.tv/helix/chat/badges/global
    // oauth:141241241241241
    //
    // scopes:
    // chat:read+chat:edit+channel:moderate+channel:read:redemptions+channel:bot+user:write:chat
    // base64: encoded app title
    // https://twitchtokengenerator.com/api/create
    //
    let response = reqwest::Client::new()
        .get("https://api.twitch.tv/helix/chat/badges/global")
        .header(
            "Authorization",
            format!("Bearer {}", token.replace("oauth:", "")),
        )
        .header("Client-Id", client_id)
        .send()
        .await?
        .json::<TwitchApiResponse<Vec<BadgeItem>>>()
        .await?;

    Ok(response.data)
}

impl TwitchMessage {
    pub fn get_nickname_color(&self) -> (u8, u8, u8) {
        let hex_color = self.color.clone().unwrap_or("#FFFFFF".to_string());
        if hex_color.is_empty() {
            return (167, 23, 124);
        }
        let color = convert_hexcode_to_rgb(hex_color).unwrap();

        (color.red, color.green, color.blue)
    }

    fn set_field_bool(&mut self, field: &str, tag_value: Option<String>) {
        if let Some(value) = tag_value {
            match field {
                "subscriber" => self.subscriber = Some(get_bool(&value)),
                "first_msg" => self.first_msg = Some(get_bool(&value)),
                "returning_chatter" => self.returning_chatter = Some(get_bool(&value)),
                "moderator" => self.moderator = Some(get_bool(&value)),
                _ => panic!(), // ... at the disco!
            }
        }
    }

    fn set_badge_value(&mut self, badge: &str) {
        let mut badge_parts = badge.split('/');
        if let Some(key) = badge_parts.next() {
            let value = badge_parts.next().unwrap_or("0");
            match key {
                "broadcaster" => self.badges.broadcaster = get_bool(value),
                "premium" => self.badges.premium = get_bool(value),
                "no_audio" => self.badges.no_audio = get_bool(value),
                _other => {
                    // println!("{}", other);
                }
            }
        }
    }
}

fn get_bool(value: &str) -> bool {
    value != "0"
}

pub async fn parse(message: Message) -> Result<TwitchMessage, Box<dyn Error>> {
    let mut twitch_message = TwitchMessage {
        badges: Badges {
            no_audio: false,
            broadcaster: false,
            premium: false,
        },
        emotes: vec![],
        nickname: None,
        display_name: None,
        first_msg: None,
        returning_chatter: None,
        subscriber: None,
        message: None,
        moderator: None,
        color: None,
    };

    let nickname: String = message.source_nickname().unwrap_or("").to_owned();
    twitch_message.nickname = Some(nickname);

    if let Some(tags) = message.tags {
        for tag in tags {
            match tag.0.as_str() {
                "badges" => set_badges(tag.1, &mut twitch_message),
                "color" => set_color(tag.1, &mut twitch_message),
                "display-name" => set_display_name(tag.1, &mut twitch_message),
                "first-msg" => twitch_message.set_field_bool("first_msg", tag.1),
                "subscriber" => twitch_message.set_field_bool("subscriber", tag.1),
                "returning-chatter" => twitch_message.set_field_bool("returning_chatter", tag.1),
                "mod" => twitch_message.set_field_bool("moderator", tag.1),
                "emotes" => process_emotes(tag.1, &mut twitch_message),
                _other => {}
            }
        }
    }

    if let Command::PRIVMSG(ref _message_sender, ref message) = message.command {
        twitch_message.message = Some(message.to_string());
        add_emotes(&mut twitch_message).await?;
    }

    Ok(twitch_message)
}

async fn add_emotes(twitch_message: &mut TwitchMessage) -> Result<(), Box<dyn Error>> {
    for emote in twitch_message.emotes.iter_mut() {
        let range = std::ops::Range {
            start: emote.start,
            end: emote.end + 1,
        };
        let temp_msg = twitch_message.message.clone().expect("no message found");
        let emote_name = temp_msg.get(range);
        emote.name = emote_name.unwrap_or("").to_string();
    }

    for emote in twitch_message.emotes.iter() {
        let file_bytes: Vec<u8> = reqwest::get(&emote.url).await?.bytes().await?.to_vec();
        let size = file_bytes.len();

        let img_data = image::load_from_memory(&file_bytes)?;

        let mut buffer: Vec<u8> = Vec::new();
        img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
        let base64_emote = BASE64_STANDARD.encode(&buffer);

        //ESC]1337;File=size=FILESIZEINBYTES;inline=1:base-64 encoded file contents^G
        // works in iTerm
        let encoded_image = format!(
            "\x1b]1337;File=size={};inline=1;height=20px;preserveAspectRatio=1:{}\x07",
            size,
            base64_emote.as_str()
        );
        // let encoded_image = format!("\x1bPtmux;\x1b\x1b]1337;File=size={};inline=1;height=20px;preserveAspectRatio=1:{}\x07\x1b\\", size, base64_emote.as_str());

        // TODO: Figure out the right encoding to make emotes work in tmux
        // let encoded_image = format!(" \x1b]tmux;\x1b]\x1b]1337;File=size={};inline=1;preserveAspectRatio=1:{}\x07", size, base64_emote.as_str());

        let mut msg = twitch_message.message.clone().unwrap();
        msg = msg.replace(&emote.name, encoded_image.as_str());
        twitch_message.message = Some(msg);
    }

    Ok(())
}

// 303147449:0-13
// id: text-position-for-emote
// https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0
fn process_emotes(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
    if let Some(value) = tag_value {
        let emotes: Vec<&str> = value.split('/').collect();
        if emotes.is_empty() {
            return;
        }

        for emote_data in emotes.into_iter() {
            let mut emote_parts = emote_data.split(':');
            let emote_id = emote_parts.next();
            let Some(emote_id) = emote_id else {
                continue;
            };

            let positions = emote_parts.next();
            let Some(emote_position_data) = positions else {
                continue;
            };
            let mut emote_position_data = emote_position_data.split('-');
            let start = emote_position_data
                .next()
                .unwrap()
                .to_string()
                .parse::<usize>()
                .unwrap();
            let end = emote_position_data
                .next()
                .unwrap()
                .to_string()
                .parse::<usize>()
                .unwrap();

            let url = format!(
                "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0",
                emote_id
            );
            let name = "".to_string();

            let emote = Emote {
                id: emote_id.to_owned(),
                start,
                end,
                url,
                name,
            };

            twitch_message.emotes.push(emote);
        }
    }
}

fn set_display_name(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
    if let Some(value) = tag_value {
        twitch_message.display_name = Some(value);
    }
}

fn set_color(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
    if let Some(value) = tag_value {
        twitch_message.color = Some(value);
    }
}

fn set_badges(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
    if let Some(value) = tag_value {
        let badges: Vec<&str> = value.split(',').collect();
        for badge in badges.into_iter() {
            twitch_message.set_badge_value(badge);
        }
    }
}

#[cfg(test)]
mod tests {
    use irc::proto::message::Tag;
    use irc::proto::Message;

    use crate::twitch::fixtures::TEST_MESSAGE_WITH_EMOTES;

    use super::parse;

    use std::error::Error;

    #[tokio::test]
    async fn test_parse_emotes_attaching() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "emotes".to_string(),
            Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags),
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG",
            vec!["#s9tpepper_", "This is a message from twitch"],
        );

        let twitch_message = parse(msg.unwrap()).await?;

        let parsed_message = twitch_message.message.unwrap();
        println!("{}", parsed_message);

        assert_eq!(TEST_MESSAGE_WITH_EMOTES, parsed_message);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_emotes_length() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "emotes".to_string(),
            Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags),
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG",
            vec!["#s9tpepper_", "This is a message from twitch"],
        );

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(2, twitch_message.emotes.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_emotes_url() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "emotes".to_string(),
            Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags),
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG",
            vec!["#s9tpepper_", "This is a message from twitch"],
        );

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(
            "https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0",
            twitch_message.emotes[0].url
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_emotes_id() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "emotes".to_string(),
            Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags),
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG",
            vec!["#s9tpepper_", "This is a message from twitch"],
        );

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!("303147449", twitch_message.emotes[0].id);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_emotes_position() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "emotes".to_string(),
            Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags),
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG",
            vec!["#s9tpepper_", "This is a message from twitch"],
        );

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(0, twitch_message.emotes[0].start);
        assert_eq!(13, twitch_message.emotes[0].end);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_message() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "badges".to_string(),
            Some("broadcaster/1,premium/1".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags),
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG",
            vec!["#s9tpepper_", "This is a message from twitch"],
        );

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(
            "This is a message from twitch",
            twitch_message.message.unwrap()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_nickname() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "badges".to_string(),
            Some("broadcaster/1,premium/1".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags),
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG",
            vec![],
        );

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!("rayslash", twitch_message.nickname.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_display_name() -> Result<(), Box<dyn Error>> {
        let tag = Tag("display-name".to_string(), Some("s9tpepper_".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!("s9tpepper_", twitch_message.display_name.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_badge_broadcaster_true() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "badges".to_string(),
            Some("broadcaster/1,premium/1".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(true, twitch_message.badges.broadcaster);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_badge_broadcaster_false() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "badges".to_string(),
            Some("broadcaster/0,premium/1".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(false, twitch_message.badges.broadcaster);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_badge_premium_true() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "badges".to_string(),
            Some("broadcaster/1,premium/1".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(true, twitch_message.badges.premium);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_badge_premium_false() -> Result<(), Box<dyn Error>> {
        let tag = Tag(
            "badges".to_string(),
            Some("broadcaster/0,premium/0".to_string()),
        );
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(false, twitch_message.badges.premium);

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_color() -> Result<(), Box<dyn Error>> {
        let tag = Tag("color".to_string(), Some("#8A2BE2".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!("#8A2BE2", twitch_message.color.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_returning_chatter_is_true() -> Result<(), Box<dyn Error>> {
        let tag = Tag("returning-chatter".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(true, twitch_message.returning_chatter.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_returning_chatter_is_false() -> Result<(), Box<dyn Error>> {
        let tag = Tag("returning-chatter".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(false, twitch_message.returning_chatter.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_subscriber_is_true() -> Result<(), Box<dyn Error>> {
        let tag = Tag("subscriber".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(true, twitch_message.subscriber.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_subscriber_is_false() -> Result<(), Box<dyn Error>> {
        let tag = Tag("subscriber".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(false, twitch_message.subscriber.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_moderator_is_true() -> Result<(), Box<dyn Error>> {
        let tag = Tag("mod".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(true, twitch_message.moderator.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_moderator_is_false() -> Result<(), Box<dyn Error>> {
        let tag = Tag("mod".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(false, twitch_message.moderator.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_first_msg_is_true() -> Result<(), Box<dyn Error>> {
        let tag = Tag("first-msg".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(true, twitch_message.first_msg.unwrap());

        Ok(())
    }

    #[tokio::test]
    async fn test_parse_first_msg_is_false() -> Result<(), Box<dyn Error>> {
        let tag = Tag("first-msg".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap()).await?;

        assert_eq!(false, twitch_message.first_msg.unwrap());

        Ok(())
    }

    const MESSAGE_WITH_EMOTES:&str = "\u{1b}]1337;File=size=2195;inline=1;height=20px;preserveAspectRatio=1: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\u{7}a\u{1b}]1337;File=size=2715;inline=1;height=20px;preserveAspectRatio=1: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\u{7}";
}
