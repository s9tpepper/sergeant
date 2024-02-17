use hex_rgb::*;
use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use base64::prelude::*;
use irc::client::prelude::Message;
use irc::proto::Command;

use directories::ProjectDirs;

const ESCAPE:&str = "\x1b";
const BELL:&str = "\x07";

type AsyncResult<T> = Result<T, Box<dyn Error>>;

pub struct Emote {
    // id: String,
    start: usize,
    end: usize,
    url: String,
    name: String,
}

pub struct TwitchMessage {
    pub badges: Vec<String>,
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

fn get_data_directory() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(project_directories) = ProjectDirs::from("com", "s9tpepper", "FerrisTwitch") {
        let data_directory = project_directories.data_dir();

        if !data_directory.exists() {
            std::fs::create_dir_all(data_directory)?;
        }

        return Ok(data_directory.to_path_buf())
    }

    Err("Could not get data directory".into())
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
    let mut response = reqwest::Client::new()
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

    let data_dir = get_data_directory()?;

    for badge_item in response.data.iter_mut() {
        let file_name = format!("{}.txt", badge_item.set_id);
        let Some(ref version) = badge_item.versions.pop() else {
            continue
        };

        let badge_path = data_dir.join(file_name);

        if !badge_path.exists() {
            generate_badge_file(badge_path, version).await?;
        }
    }

    Ok(response.data)
}

async fn get_encoded_image(url: &str) -> Result<String, Box<dyn Error>>{
    let file_bytes: Vec<u8> = reqwest::get(url).await?.bytes().await?.to_vec();
    let img_data = image::load_from_memory(&file_bytes)?;

    let mut buffer: Vec<u8> = Vec::new();
    img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
    let base64_emote = BASE64_STANDARD.encode(&buffer);

    Ok(base64_emote)
}

async fn generate_badge_file(badge_path: PathBuf, version: &BadgeVersion) -> Result<(), Box<dyn Error>> {
    if let Ok(encoded_image) = get_encoded_image(&version.image_url_1x).await {
        fs::write(badge_path, encoded_image)?;
    }

    Ok(())
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
}

fn get_bool(value: &str) -> bool {
    value != "0"
}

pub async fn parse(message: Message) -> Result<TwitchMessage, Box<dyn Error>> {
    let mut twitch_message = TwitchMessage {
        badges: vec![],
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
        add_badges(&mut twitch_message).await?;
    }

    Ok(twitch_message)
}

fn get_iterm_encoded_image(base64: String) -> String {
    format!(
        "{}]1337;File=inline=1;height=20px;preserveAspectRatio=1:{}{}",
        ESCAPE,
        base64.as_str(),
        BELL
    )
}

async fn add_badges(twitch_message: &mut TwitchMessage) -> Result<(), Box<dyn Error>> {
    let data_dir = get_data_directory()?;
    for badge in twitch_message.badges.iter() {
        let badge_path = data_dir.join(format!("{}.txt", badge));
        let base64 = fs::read_to_string(badge_path)?;
        let encoded_badge = get_iterm_encoded_image(base64);
        twitch_message.display_name = Some(format!("{} {}", encoded_badge.as_str(), twitch_message.display_name.as_ref().unwrap()));
    }

    Ok(())
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

        let img_data = image::load_from_memory(&file_bytes)?;

        let mut buffer: Vec<u8> = Vec::new();
        img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
        let base64_emote = BASE64_STANDARD.encode(&buffer);

        //ESC]1337;File=size=FILESIZEINBYTES;inline=1:base-64 encoded file contents^G
        // works in iTerm
        let encoded_image = format!(
            "\x1b]1337;File=inline=1;height=20px;preserveAspectRatio=1:{}\x07",
            base64_emote.as_str()
        );

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
                // id: emote_id.to_owned(),
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

        let mut valid_badges: Vec<String> = vec![];
        for badge in badges.into_iter() {
            let mut badge_parts = badge.split('/');
            if let Some(key) = badge_parts.next() {
                let value = badge_parts.next().unwrap_or("0");
                if value == "1" {
                    valid_badges.push(key.to_string());
                }
            }
        }

        twitch_message.badges = valid_badges;
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

    // #[tokio::test]
    // async fn test_parse_emotes_id() -> Result<(), Box<dyn Error>> {
    //     let tag = Tag(
    //         "emotes".to_string(),
    //         Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
    //     );
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(
    //         Some(tags),
    //         Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
    //         "PRIVMSG",
    //         vec!["#s9tpepper_", "This is a message from twitch"],
    //     );
    //
    //     let twitch_message = parse(msg.unwrap()).await?;
    //
    //     assert_eq!("303147449", twitch_message.emotes[0].id);
    //
    //     Ok(())
    // }

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
}
