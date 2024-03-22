use std::{error::Error, fs, io::Cursor};

use base64::prelude::*;
use serde::{Deserialize, Serialize};

use crate::utils::get_data_directory;

const ESCAPE: &str = "\x1b";
const BELL: &str = "\x07";

#[derive(Debug)]
pub struct Emote {
    start: usize,
    end: usize,
    url: String,
    name: String,
}

#[derive(Debug)]
pub struct ChatMessage {
    pub badges: Vec<String>,
    pub emotes: Vec<Emote>,
    pub nickname: String,
    pub first_msg: bool,
    pub returning_chatter: bool,
    pub subscriber: bool,
    pub moderator: bool,
    pub message: String,
    pub color: String,
    pub channel: String,
    pub raw: String,
}

#[derive(Debug)]
pub enum TwitchMessage {
    RaidMessage { user_id: String, raid_notice: String },
    PrivMessage { message: ChatMessage },
    UnknownMessage { message: String },
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

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
struct IrcMessage<'a> {
    tags: Vec<(&'a str, &'a str)>,
    r#type: &'a str,
    sender: &'a str,
    raw: &'a str,
    // message: Option<&'a str>,
}

/// Tmux sucks.
fn is_tmux() -> bool {
    let term = std::env::var("TERM").unwrap();
    term.contains("tmux") || term.contains("screen")
}

// @badge-info=;badges=;client-nonce=b0377556cf50be6ca38957b8ca735aa8;color=#FF4500;display-name=vei_bean;emotes=;first-msg=0;flags=;id=e4c10c03-a606-47f0-b0ab-2d83d415af46;mod=0;returning-chatter=0;room-id=961536166;subscriber=0;tmi-sent-ts=1708304710278;turbo=0;user-id=624578741;user-type= :vei_bean!vei_bean@vei_bean.tmi.twitch.tv PRIVMSG #s9tpepper_ hello
//
//
// @badge-info=;badges=;color=#FF4500;display-name=vei_bean;emotes=;flags=;id=4c33fcb0-9337-4e68-b7d0-3a3049ad7cfd;login=vei_bean;mod=0;msg-id=raid;msg-param-displayName=vei_bean;msg-param-login=vei_bean;msg-param-profileImageURL=https://static-cdn.jtvnw.net/jtv_user_pictures/618358c1-993a-4a2d-b0b9-a51d1827c659-profile_image-%s.png;msg-param-viewerCount=1;room-id=961536166;subscriber=0;system-msg=1\sraiders\sfrom\svei_bean\shave\sjoined!;tmi-sent-ts=1708304703515;user-id=624578741;user-type=;vip=0 :tmi.twitch.tv USERNOTICE #s9tpepper_

pub fn parse(message: String) -> Result<TwitchMessage, Box<dyn Error>> {
    let raw = message.as_str();
    let Some((tags_str, msg)) = message.split_once(' ') else {
        return Err("Could not parse message".into());
    };

    let mut tags: Vec<(&str, &str)> = vec![];
    for tag_pair in tags_str.split(';') {
        let Some(pair) = tag_pair.split_once('=') else {
            continue;
        };
        tags.push(pair);
    }

    let mut message_parts = msg.split(' ');

    let (mut sender, _) = message_parts.next().unwrap_or("").split_once('!').unwrap_or(("", ""));
    sender = sender.trim_start_matches(':');

    if let Some((_, display_name)) = tags.iter().find(|(tag, _)| *tag == "display-name") {
        sender = display_name;
    }

    let r#type = message_parts.next().unwrap_or("");

    let irc_message = IrcMessage {
        tags,
        sender,
        r#type,
        raw,
    };

    match r#type {
        "PRIVMSG" => {
            let channel = message_parts.next().expect("Missing channel in message");
            let mut message_text: String = message_parts.collect::<Vec<&str>>().join(" ");
            if message_text.starts_with(':') {
                message_text = message_text[1..].to_string();
            }

            Ok(parse_privmsg(irc_message, message_text, channel))
        }
        "USERNOTICE" => {
            let channel = message_parts.next().expect("Missing channel in message");
            Ok(parse_usernotice(irc_message, channel))
        }
        _ => Err("Unknown message type".into()),
    }
}

/// A message tag as defined by [IRCv3.2](http://ircv3.net/specs/core/message-tags-3.2.html).
/// It consists of a tag key, and an optional value for the tag. Each message can contain a number
/// of tags (in the string format, they are separated by semicolons). Tags are used to add extended
/// information to a message under IRCv3.
#[derive(Clone, PartialEq, Debug)]
pub struct Tag<'a>(pub &'a str, pub &'a str);

fn set_badges(tag_value: &str, valid_badges: &mut Vec<String>) {
    for badge in tag_value.split(',') {
        let mut badge_parts = badge.split('/');
        if let Some(key) = badge_parts.next() {
            let value = badge_parts.next().unwrap_or("0");
            if value == "1" {
                valid_badges.push(key.to_string());
            }
        }
    }
}

fn get_bool(value: &str) -> bool {
    value != "0"
}

// 303147449:0-13
// id: text-position-for-emote
// https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0
fn process_emotes(value: &str, emotes: &mut Vec<Emote>) {
    for emote_data in value.split('/') {
        let mut emote_parts = emote_data.split(':');
        let emote_id = emote_parts.next();
        let Some(emote_id) = emote_id else {
            continue;
        };

        let positions = emote_parts.next();
        let Some(mut emote_position_data) = positions else {
            continue;
        };

        if let Some((a, _)) = emote_position_data.split_once(',') {
            emote_position_data = a;
        }

        let (s, e) = emote_position_data.split_once('-').unwrap();
        let start = s.to_string().parse::<usize>().unwrap();
        let end = e.to_string().parse::<usize>().unwrap();

        let url = format!(
            "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0",
            emote_id
        );

        let name = "".to_string();

        let emote = Emote { start, end, url, name };

        emotes.push(emote);
    }
}

fn get_emote_prefix() -> String {
    if is_tmux() {
        return format!("{0}Ptmux;{0}{0}]", ESCAPE);
    }

    format!("{ESCAPE}]")
}

fn get_emote_suffix() -> String {
    if is_tmux() {
        return format!("{}{}\\.", BELL, ESCAPE);
    }

    BELL.to_string()
}

fn add_emotes(message: &mut String, emotes: &mut [Emote]) -> Result<(), Box<dyn Error>> {
    for emote in emotes.iter_mut() {
        let range = emote.start..=emote.end;
        let temp_msg = message.clone();
        let emote_name = temp_msg.get(range);
        emote.name = emote_name.unwrap_or("").to_string();
    }

    for emote in emotes.iter() {
        let response = ureq::get(&emote.url).call()?;
        let length: usize = response.header("content-length").unwrap().parse()?;
        let mut file_bytes: Vec<u8> = vec![0; length];
        response.into_reader().read_exact(&mut file_bytes)?;

        let img_data = image::load_from_memory(&file_bytes)?;

        let mut buffer: Vec<u8> = Vec::new();
        img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
        let base64_emote = BASE64_STANDARD.encode(&buffer);

        let encoded_image = format!(
            // "{}1337;File=inline=1;height=22px;width=22px;preserveAspectRatio=1:{}{}",
            "{}1337;File=inline=1;height=22px;preserveAspectRatio=1:{}{}",
            get_emote_prefix(),
            base64_emote.as_str(),
            get_emote_suffix()
        );

        *message = message.replace(&emote.name, encoded_image.as_str());
    }

    Ok(())
}

fn get_iterm_encoded_image(base64: String) -> String {
    let base64_str = base64.as_str();

    // format!("{ESCAPE}]1337;File=inline=1;height=22px;preserveAspectRatio=1:{base64_str}{BELL}")
    format!("{ESCAPE}]1337;File=inline=1;preserveAspectRatio=1:{base64_str}{BELL}")
}

fn add_badges(badges: &[String]) -> Result<String, Box<dyn Error>> {
    let mut badges_list = String::new();
    let data_dir = get_data_directory(None)?;
    for badge in badges.iter() {
        let badge_path = data_dir.join(format!("{}.txt", badge));
        let base64 = fs::read_to_string(badge_path)?;
        let encoded_badge = get_iterm_encoded_image(base64);

        badges_list.push_str(&encoded_badge);
    }

    Ok(badges_list)
}

fn parse_privmsg(irc_message: IrcMessage, message_text: String, channel: &str) -> TwitchMessage {
    let mut badges: Vec<String> = vec![];
    let mut color = "#FF9912".to_string();
    let mut first_msg = false;
    let mut subscriber = false;
    let mut returning_chatter = false;
    let mut moderator = false;
    let mut emotes: Vec<Emote> = vec![];

    for (tag, value) in irc_message.tags {
        match tag {
            "badges" => set_badges(value, &mut badges),
            "color" => {
                if !value.is_empty() {
                    color = value.to_string();
                }
            }
            "first-msg" => {
                first_msg = get_bool(value);
            }
            "subscriber" => {
                subscriber = get_bool(value);
            }
            "returning-chatter" => {
                returning_chatter = get_bool(value);
            }
            "mod" => {
                moderator = get_bool(value);
            }
            "emotes" => process_emotes(value, &mut emotes),
            _other => {}
        }
    }

    let mut message = message_text.to_string();
    let _ = add_emotes(&mut message, &mut emotes);
    let encoded_badges = add_badges(&badges).unwrap_or("".to_string());
    let nickname = format!("{}{}", encoded_badges, irc_message.sender);

    TwitchMessage::PrivMessage {
        message: ChatMessage {
            badges,
            emotes,
            first_msg,
            returning_chatter,
            subscriber,
            moderator,
            color,
            message,
            nickname,
            channel: channel.to_string(),
            raw: irc_message.raw.to_string(),
        },
    }
}

fn parse_usernotice(message: IrcMessage, _channel: &str) -> TwitchMessage {
    let mut system_msg = String::new();
    let mut is_raid = false;
    let mut user_id = String::new();

    for (tag, value) in message.tags {
        if value == "raid" {
            is_raid = true;
        }

        if tag == "system-msg" {
            system_msg = value.to_string();
        }

        if tag == "user-id" {
            user_id = value.to_string();
        }
    }

    if is_raid && !system_msg.is_empty() {
        return TwitchMessage::RaidMessage {
            raid_notice: system_msg,
            user_id,
        };
    }

    TwitchMessage::UnknownMessage {
        message: message.raw.to_string(),
    }
}
