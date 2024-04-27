use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

use base64::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Block, Widget},
};
use serde::{Deserialize, Serialize};

use crate::{
    tui::{MessageParts, Symbol},
    utils::get_data_directory,
};

use super::pubsub::TwitchApiResponse;

const ESCAPE: &str = "\x1b";
const BELL: &str = "\x07";
const EMOTE_SPACE: u8 = 2;

static EMOTE_CACHE: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

#[derive(Debug, PartialEq, Clone)]
pub struct Text {
    char: String,
    color: Option<(u8, u8, u8)>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Emote {
    emote_id: String,
    start: usize,
    end: usize,
    url: String,
    name: String,
    encoded: Option<String>,
}

impl Emote {
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        if self.encoded.is_some() {
            return Ok(());
        }

        let cache = EMOTE_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        let cache_read = cache.read().unwrap();
        if let Some(encoding) = cache_read.get(&self.emote_id) {
            self.encoded = Some(encoding.to_string());

            return Ok(());
        }
        drop(cache_read);

        let response = ureq::get(&self.url).call()?;
        let length: usize = response.header("content-length").unwrap().parse()?;
        let mut file_bytes: Vec<u8> = vec![0; length];
        response.into_reader().read_exact(&mut file_bytes)?;

        // let things = &file_bytes[..4];
        // panic!("{:?}", things);

        // let img_data = image::load_from_memory(&file_bytes)?;
        // let mut buffer: Vec<u8> = Vec::new();
        // // img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)?;
        // img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Gif)?;
        // let base64_emote = BASE64_STANDARD.encode(&buffer);
        let base64_emote = BASE64_STANDARD.encode(&file_bytes);

        let encoded_image = format!(
            // "{}1337;File=inline=1;height=22px;width=22px;preserveAspectRatio=1;doNotMoveCursor=1:{}{}",
            "{}1337;File=inline=1;height=22px;width=22px;doNotMoveCursor=1:{}{}",
            get_emote_prefix(),
            base64_emote.as_str(),
            get_emote_suffix()
        );

        self.encoded = Some(encoded_image.clone());
        let mut cache_write = cache.write().unwrap();
        cache_write.insert(self.emote_id.clone(), encoded_image);

        Ok(())
    }
}

impl Clone for Emote {
    fn clone(&self) -> Self {
        Self {
            emote_id: self.emote_id.clone(),
            start: self.start,
            end: self.end,
            url: self.url.clone(),
            name: self.name.clone(),
            encoded: self.encoded.clone(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub badges: Vec<Emote>,
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
    #[serde(skip)]
    pub area: Option<Rect>,
}

// Place all characters and emote base64s in a vector
pub fn get_message_symbols(message: &str, emotes: &mut [Emote], color: Option<(u8, u8, u8)>) -> Vec<Symbol> {
    // Load the base64 encoded emotes
    emotes.iter_mut().for_each(|e| {
        e.load().unwrap();
    });

    // Place all characters and emote base64s in a vector
    let mut symbols: Vec<Symbol> = vec![];
    let msg_length = message.len();

    let mut cursor = 0;
    'outer: for i in 0..msg_length {
        if i < cursor {
            continue;
        }

        for emote in emotes.iter_mut() {
            if emote.start == i {
                let emote_length = emote.end - emote.start;
                symbols.push(Symbol::Emote(emote.clone()));
                cursor += emote_length + 1;
                continue 'outer;
            }
        }

        let temp = message.chars().nth(i).unwrap_or(' ').to_string();
        let c: &str = temp.as_str();
        symbols.push(Symbol::Text(Text {
            char: c.to_string(),
            color,
        }));

        cursor += 1;
    }

    symbols
}

#[test]
fn test_get_message_symbols() {
    let emote = Emote {
        emote_id: "12345".to_string(),
        start: 0,
        end: 13,
        url: "https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/light/1.0".to_string(),
        name: "primeagenEmacs".to_string(),
        encoded: None,
    };

    let mut emotes: Vec<Emote> = vec![emote];
    let message = "primeagenEmacs Hello";
    let symbols = get_message_symbols(message, &mut emotes, None);
    assert_eq!(symbols, vec![]);
}

// #[test]
// fn test_get_message_words() {
//     let emote = Emote {
//         start: 0,
//         end: 13,
//         url: "https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/light/1.0".to_string(),
//         name: "primeagenEmacs".to_string(),
//         encoded: None,
//     };
//
//     let mut emotes: Vec<Emote> = vec![emote];
//     let message = "primeagenEmacs Hello";
//     let symbols = get_message_symbols(message, &mut emotes);
//
//     let message_parts = get_message_parts(&symbols);
//     assert_eq!(message_parts, vec![]);
// }

fn get_message_parts(symbols: &[Symbol]) -> Vec<MessageParts> {
    let mut message_to_render: Vec<MessageParts> = vec![];
    let mut word: Vec<Symbol> = vec![];
    symbols.iter().enumerate().for_each(|(index, s)| match s {
        Symbol::Text(character) => {
            let previous_index = if index == 0 { index } else { index - 1 };
            let max_index = symbols.len() as u16 - 1;
            let next_index = if index as u16 == max_index {
                max_index
            } else {
                index as u16 + 1
            };

            let previous = &symbols[previous_index];
            let next = &symbols[next_index as usize];

            let mut previous_is_emote = false;
            if let Symbol::Emote(_) = previous {
                previous_is_emote = true;
            };

            let mut next_is_emote = false;
            if let Symbol::Emote(_) = next {
                next_is_emote = true;
            };

            if character.char == " " && !word.is_empty() && !previous_is_emote {
                message_to_render.push(MessageParts::Text(word.clone()));

                word.clear();
                message_to_render.push(MessageParts::Text(vec![Symbol::Text(Text {
                    char: " ".to_string(),
                    color: None,
                })]));
            } else if character.char == " " && previous_is_emote && next_is_emote {
                // Don't do anything, skip adding spaces between emotes
            } else {
                word.push(s.clone());
            }
        }
        Symbol::Emote(emote) => {
            if !word.is_empty() {
                message_to_render.push(MessageParts::Text(word.clone()));
                word.clear();
                message_to_render.push(MessageParts::Text(vec![Symbol::Text(Text {
                    char: " ".to_string(),
                    color: None,
                })]));
            }

            message_to_render.push(MessageParts::Emote(emote.clone()));
        }
    });

    // Collect the last word
    if !word.is_empty() {
        message_to_render.push(MessageParts::Text(word.clone()));
    }
    word.clear();

    message_to_render
}

fn get_nickname_color(color: &str) -> (u8, u8, u8) {
    let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);

    (r, g, b)
}

#[derive(Debug)]
pub struct RenderCursor {
    pub x: u16,
    pub y: u16,
}

pub fn get_lines(symbols: &[Symbol], area: &Rect) -> Vec<Vec<MessageParts>> {
    let message_parts = get_message_parts(symbols);
    let mut lines: Vec<Vec<MessageParts>> = vec![];
    let mut line: Vec<MessageParts> = vec![];
    let mut line_length = 0;

    message_parts.iter().enumerate().for_each(|(ndx, part)| {
        let section_length = match part {
            MessageParts::Text(word) => word.len(),
            MessageParts::Emote(_) => EMOTE_SPACE as usize,
        };

        let next_line_length = line_length + section_length;
        if next_line_length > (area.width - 1) as usize {
            lines.push(line.clone());
            line.clear();

            line_length = section_length;
        } else {
            line_length += section_length;
        }

        // Check that section_length isn't wider than the area
        // if it is, split the section into multiple lines
        if let MessageParts::Text(word) = part {
            if section_length >= area.width.into() {
                let chunks = word.chunks((area.width - 2).into());
                let last_index = chunks.len() - 1;
                chunks.enumerate().for_each(|(index, chunk)| {
                    let mut symbols: Vec<Symbol> = vec![];
                    chunk.iter().for_each(|c| {
                        symbols.push(c.clone());
                    });

                    if index != last_index {
                        symbols.push(Symbol::Text(Text {
                            char: "-".to_string(),
                            color: None,
                        }));
                        line.push(MessageParts::Text(symbols));
                        lines.push(line.clone());
                        line.clear();
                    } else {
                        line.push(MessageParts::Text(symbols));
                    }
                });
            } else {
                line.push(part.clone());
            }
        } else {
            line.push(part.clone());
        }

        if ndx == message_parts.len() - 1 {
            // Gather the last line
            if !line.is_empty() {
                lines.push(line.clone());
            }
        }
    });

    // Remove spaces if they appear at the beginning of a line
    lines.iter_mut().for_each(|line| {
        let first_word = line.first();
        if let Some(MessageParts::Text(first_word)) = first_word {
            if first_word.len() == 1 {
                if let Some(Symbol::Text(symbol)) = first_word.first() {
                    if symbol.char == *" ".to_string() {
                        line.remove(0);
                    }
                }
            }
        }
    });

    lines
}

pub fn get_screen_lines(lines: &mut [Vec<MessageParts>], area: &Rect) -> Vec<Vec<MessageParts>> {
    if lines.len() > area.height.into() {
        let line_limit = area.height.saturating_sub(1);

        let start = lines.len() - line_limit as usize;
        lines[start..].to_vec()
    } else {
        lines[..].to_vec()
    }
}

pub fn write_to_buffer(lines: &mut [Vec<MessageParts>], buf: &mut Buffer, cursor: &mut RenderCursor) {
    let left = cursor.x;

    lines.iter_mut().for_each(|line| {
        line.iter().for_each(|s| match s {
            MessageParts::Text(word) => {
                word.iter().for_each(|symbol| match symbol {
                    Symbol::Text(character) => {
                        let index = buf.index_of(cursor.x, cursor.y);
                        if index < buf.content.len() {
                            let (r, g, b) = character.color.unwrap_or((255, 255, 255));
                            let rgb = Color::Rgb(r, g, b);
                            buf.get_mut(cursor.x, cursor.y).set_symbol(&character.char).set_fg(rgb);
                            cursor.x += 1;
                        }
                    }
                    Symbol::Emote(_) => {}
                });
            }

            MessageParts::Emote(emote) => {
                let index = buf.index_of(cursor.x, cursor.y);
                if index < buf.content.len() {
                    let encoded = emote.encoded.clone().unwrap_or_default();
                    buf.get_mut(cursor.x, cursor.y).set_symbol(&encoded);
                    cursor.x += EMOTE_SPACE as u16;
                }
            }
        });

        cursor.x = left;
        cursor.y += 1;
    });
}

impl Widget for &mut RaidMessage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut cursor = RenderCursor {
            x: area.left(),
            y: area.bottom(),
        };

        // Render the messages in yellow
        let symbols: Vec<Symbol> = get_message_symbols(&self.raid_notice, &mut [], Some((255, 255, 0)));

        // Shrink horizontal area by 4 to make space for border and scroll bar
        let mut line_area = area;
        line_area.width = area.width - 4;

        let mut lines: Vec<Vec<MessageParts>> = get_lines(&symbols, &area);

        // Move cursor one over to make space for border
        cursor.x = area.left() + 1;
        cursor.y = cursor.y.saturating_sub(lines.len() as u16).saturating_sub(1);

        let mut screen_lines = get_screen_lines(&mut lines, &area);

        write_to_buffer(&mut screen_lines, buf, &mut cursor);

        let block_area = Rect {
            x: 0,
            y: cursor.y.saturating_sub(2),
            width: area.width.saturating_sub(2),
            height: screen_lines.len() as u16 + 2,
        };

        Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::reset().fg(Color::LightYellow))
            .title(format!("🪂 {} Raid", self.display_name))
            .render(block_area, buf);

        self.area = Some(Rect {
            x: 0,
            y: cursor.y,
            width: area.width,
            height: screen_lines.len() as u16,
        });
    }
}

impl Widget for &mut RedeemMessage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut cursor = RenderCursor {
            x: area.left(),
            y: area.bottom(),
        };

        // Render the messages in green
        let symbols: Vec<Symbol> = get_message_symbols(&self.message, &mut [], Some((0, 255, 0)));
        let mut lines: Vec<Vec<MessageParts>> = get_lines(&symbols, &area);

        cursor.x = area.left();
        cursor.y = cursor.y.saturating_sub(lines.len() as u16);

        let mut screen_lines = get_screen_lines(&mut lines, &area);

        write_to_buffer(&mut screen_lines, buf, &mut cursor);

        self.area = Some(Rect {
            x: 0,
            y: cursor.y,
            width: area.width,
            height: lines.len() as u16,
        });
    }
}

impl ChatMessage {
    fn get_symbols(&mut self) -> Vec<Symbol> {
        let mut symbols: Vec<Symbol> = get_message_symbols(&self.message, &mut self.emotes, None);

        // add space after nickname
        symbols.insert(
            0,
            Symbol::Text(Text {
                char: " ".to_string(),
                color: None,
            }),
        );

        // add colon for nickname
        symbols.insert(
            0,
            Symbol::Text(Text {
                char: ":".to_string(),
                color: None,
            }),
        );

        // add nickname to front of message
        let color = get_nickname_color(&self.color);
        self.nickname.chars().rev().for_each(|char| {
            symbols.insert(
                0,
                Symbol::Text(Text {
                    char: char.to_string(),
                    color: Some(color),
                }),
            )
        });

        // add badges to front of message
        self.badges.iter().for_each(|badge| {
            symbols.insert(0, Symbol::Emote(badge.clone()));
        });

        symbols
    }

    pub fn get_area(&self, area: Rect) -> Rect {
        Rect::new(0, 0, area.width, 2)
    }
}

impl Widget for &mut ChatMessage {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Initialize the cursor position
        let mut cursor = RenderCursor {
            x: area.left(),
            y: area.bottom(),
        };

        let needs_borders = self.first_msg;

        // NOTE: Used to test first time chatter decoration
        // let needs_borders = self.message.len() % 2 > 0;

        let symbols: Vec<Symbol> = self.get_symbols();

        let mut line_area = area;
        line_area.width = if needs_borders { area.width - 4 } else { area.width };
        let mut screen_lines: Vec<Vec<MessageParts>> = get_lines(&symbols, &line_area);

        let y_pos = cursor.y.saturating_sub(screen_lines.len() as u16);
        cursor.x = if needs_borders { area.left() + 1 } else { area.left() };
        cursor.y = if needs_borders { y_pos.saturating_sub(1) } else { y_pos };

        let mut writeable_area = area;
        writeable_area.width = if needs_borders { area.width - 1 } else { area.width };
        writeable_area.height = if needs_borders {
            screen_lines.len() as u16 + 2
        } else {
            screen_lines.len() as u16
        };

        write_to_buffer(&mut screen_lines, buf, &mut cursor);

        // Reset cursor position after writing to buffer
        cursor.x = 0;
        cursor.y = cursor.y.saturating_sub(writeable_area.height) + 1;

        if needs_borders {
            Block::bordered()
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::reset().fg(Color::Rgb(255, 255, 0)))
                .title("✨First time chatter")
                .render(
                    Rect {
                        x: cursor.x,
                        // y: cursor.y + 1,
                        y: cursor.y,
                        width: area.width - 2,
                        height: screen_lines.len() as u16 + 2,
                    },
                    buf,
                );
        }

        // Update the area this message takes
        self.area = Some(Rect {
            x: 0,
            y: cursor.y,
            width: area.width,
            height: writeable_area.height,
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TwitchMessage {
    ClearMessage { message: ClearMessage },
    RedeemMessage { message: RedeemMessage },
    RaidMessage { message: RaidMessage },
    PrivMessage { message: ChatMessage },
    PingMessage { message: String },
    UnknownMessage { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClearMessage {
    pub display_name: String,
    pub message_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaidMessage {
    pub display_name: String,
    pub user_id: String,
    pub raid_notice: String,
    #[serde(skip)]
    pub area: Option<Rect>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedeemMessage {
    pub message: String,
    #[serde(skip)]
    pub area: Option<Rect>,
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
    parameters: &'a str,
    channel: &'a str,
    tags: Vec<(&'a str, &'a str)>,
    r#type: &'a str,
    sender: &'a str,
    raw: &'a str,
    // message: Option<&'a str>,
}

// TODO: Come back to this later for fixing
//
/// Tmux sucks.
// fn is_tmux() -> bool {
//     let term = std::env::var("TERM").unwrap();
//     term.contains("tmux") || term.contains("screen")
// }

// @badge-info=;badges=;client-nonce=b0377556cf50be6ca38957b8ca735aa8;color=#FF4500;display-name=vei_bean;emotes=;first-msg=0;flags=;id=e4c10c03-a606-47f0-b0ab-2d83d415af46;mod=0;returning-chatter=0;room-id=961536166;subscriber=0;tmi-sent-ts=1708304710278;turbo=0;user-id=624578741;user-type= :vei_bean!vei_bean@vei_bean.tmi.twitch.tv PRIVMSG #s9tpepper_ hello
//
//
// @badge-info=;badges=;color=#FF4500;display-name=vei_bean;emotes=;flags=;id=4c33fcb0-9337-4e68-b7d0-3a3049ad7cfd;login=vei_bean;mod=0;msg-id=raid;msg-param-displayName=vei_bean;msg-param-login=vei_bean;msg-param-profileImageURL=https://static-cdn.jtvnw.net/jtv_user_pictures/618358c1-993a-4a2d-b0b9-a51d1827c659-profile_image-%s.png;msg-param-viewerCount=1;room-id=961536166;subscriber=0;system-msg=1\sraiders\sfrom\svei_bean\shave\sjoined!;tmi-sent-ts=1708304703515;user-id=624578741;user-type=;vip=0 :tmi.twitch.tv USERNOTICE #s9tpepper_
//

fn get_encoded_image(url: &str) -> Result<String, Box<dyn Error>> {
    let response = ureq::get(url).call()?;
    let length: usize = response.header("content-length").unwrap().parse()?;
    let file_bytes: Vec<u8> = vec![0; length];
    // response.into_reader().read_exact(&mut file_bytes)?;
    //
    // let img_data = image::load_from_memory(&file_bytes)?;
    //
    // let mut buffer: Vec<u8> = Vec::new();
    // // img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)?;
    // img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Gif)?;
    let base64_emote = BASE64_STANDARD.encode(file_bytes);

    Ok(base64_emote)
}

fn generate_badge_file(badge_path: PathBuf, version: &BadgeVersion) -> Result<(), Box<dyn Error>> {
    if let Ok(encoded_image) = get_encoded_image(&version.image_url_1x) {
        fs::write(badge_path, encoded_image)?;
    }

    Ok(())
}

type AsyncResult<T> = Result<T, Box<dyn Error>>;
pub fn get_badges(token: &str, client_id: &str) -> AsyncResult<Vec<BadgeItem>> {
    // Global badges: https://api.twitch.tv/helix/chat/badges/global
    // oauth:141241241241241
    //
    // scopes:
    // chat:read+chat:edit+channel:moderate+channel:read:redemptions+channel:bot+user:write:chat
    // base64: encoded app title
    // https://twitchtokengenerator.com/api/create
    //
    let response = ureq::get("https://api.twitch.tv/helix/chat/badges/global")
        .set("Authorization", &format!("Bearer {}", token.replace("oauth:", "")))
        .set("Client-Id", client_id)
        .call()?;

    let mut response: TwitchApiResponse<Vec<BadgeItem>> = serde_json::from_reader(response.into_reader())?;

    let data_dir = get_data_directory(None)?;

    for badge_item in response.data.iter_mut() {
        let file_name = format!("{}.txt", badge_item.set_id);
        let Some(ref version) = badge_item.versions.pop() else {
            continue;
        };

        let badge_path = data_dir.join(file_name);

        if !badge_path.exists() {
            generate_badge_file(badge_path, version)?;
        }
    }

    Ok(response.data)
}

pub fn parse(mut message: &str) -> Result<TwitchMessage, Box<dyn Error>> {
    let raw = message;

    let mut tags = vec![];
    let mut sender: &str = "";
    let channel: &str;
    let parameters: &str;

    // Check if the message contains tags
    if message.starts_with('@') {
        let Some((tags_str, msg)) = message.split_once(' ') else {
            return Err("Could not parse message".into());
        };

        tags = tags_str
            .split(';')
            .filter_map(|tag_pair| tag_pair.split_once('='))
            .collect();

        message = msg;
    }

    if message.starts_with(':') {
        let Some((left, msg)) = message.split_once(' ') else {
            return Err("Could not parse message".into());
        };

        sender = left.trim_start_matches(':');

        if let Some((_, display_name)) = tags.iter().find(|(tag, _)| *tag == "display-name") {
            sender = display_name;
        }

        message = msg;
    }

    let (r#type, rest) = message.split_once(' ').unwrap_or(("", ""));
    if rest.starts_with('#') {
        let (c, p) = rest.split_once(' ').unwrap_or(("", ""));
        channel = c;
        parameters = p.strip_prefix(':').unwrap_or(p);
    } else {
        channel = "";
        parameters = rest.strip_prefix(':').unwrap_or(rest);
    }

    let irc_message = IrcMessage {
        tags,
        sender,
        r#type,
        channel,
        parameters,
        raw,
    };

    match r#type {
        "PRIVMSG" => Ok(parse_privmsg(irc_message)),

        "USERNOTICE" => Ok(parse_usernotice(irc_message)),

        "PING" => {
            let message: String = irc_message.parameters.to_string();
            Ok(TwitchMessage::PingMessage { message })
        }

        "CLEARMSG" => Ok(parse_clearmsg(irc_message)),

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
    // Some("81274:0-5,7-12,14-19/30259:21-27")
    for emote_data in value.split('/') {
        let mut emote_parts = emote_data.split(':');
        let emote_id = emote_parts.next();
        let Some(emote_id) = emote_id else {
            continue;
        };

        let positions = emote_parts.next();
        let Some(emote_position_data) = positions else {
            continue;
        };

        emote_position_data.split(',').for_each(|position| {
            let (s, e) = position.split_once('-').unwrap();
            let start = s.to_string().parse().unwrap();
            let end = e.to_string().parse().unwrap();

            let url = format!(
                "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0",
                emote_id
            );

            let name = "".to_string();
            let encoded = None;

            let emote = Emote {
                emote_id: emote_id.to_string(),
                start,
                end,
                url,
                name,
                encoded,
            };

            emotes.push(emote);
        });
    }
}

fn get_emote_prefix() -> String {
    // if is_tmux() {
    //     return format!("{0}Ptmux;{0}{0}]", ESCAPE);
    // }

    format!("{ESCAPE}]")
}

fn get_emote_suffix() -> String {
    // if is_tmux() {
    //     return format!("{}{}\\", BELL, ESCAPE);
    // }

    BELL.to_string()
}

fn get_iterm_encoded_image(base64: String) -> String {
    let base64_str = base64.as_str();

    // format!("{ESCAPE}]1337;File=inline=1;height=22px;preserveAspectRatio=1:{base64_str}{BELL}")
    format!("{ESCAPE}]1337;File=inline=1;preserveAspectRatio=1:{base64_str}{BELL}")
}

fn get_badges_symbols(badges: &[String]) -> Result<Vec<Emote>, Box<dyn Error>> {
    let mut badges_symbols: Vec<Emote> = vec![];
    let data_dir = get_data_directory(None)?;
    for badge in badges.iter() {
        let badge_path = data_dir.join(format!("{}.txt", badge));
        let base64 = fs::read_to_string(badge_path)?;
        let encoded = get_iterm_encoded_image(base64);

        badges_symbols.push(Emote {
            emote_id: badge.to_string(),
            start: 0,
            end: 0,
            url: "".to_string(),
            name: "".to_string(),
            encoded: Some(encoded),
        });
    }

    Ok(badges_symbols)
}

fn parse_clearmsg(irc_message: IrcMessage) -> TwitchMessage {
    let mut message_id: String = String::new();
    let mut display_name: String = String::new();

    for (tag, value) in irc_message.tags {
        match tag {
            "target-msg-id" => message_id = value.to_string(),
            "login" => display_name = value.to_string(),
            _ => {}
        }
    }

    TwitchMessage::ClearMessage {
        message: ClearMessage {
            display_name,
            message_id,
        },
    }
}

fn parse_privmsg(irc_message: IrcMessage) -> TwitchMessage {
    let mut badges: Vec<String> = vec![];
    let mut color = "#FF9912".to_string();
    let mut first_msg = false;
    let mut subscriber = false;
    let mut returning_chatter = false;
    let mut moderator = false;
    let mut emotes: Vec<Emote> = vec![];
    let mut id = String::new();

    for (tag, value) in irc_message.tags {
        match tag {
            "id" => id = value.to_string(),
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

    // let _ = add_emotes(&mut message, &mut emotes);
    let badges_symbols = get_badges_symbols(&badges);
    // let nickname = format!("{}{}", encoded_badges, irc_message.sender);

    TwitchMessage::PrivMessage {
        message: ChatMessage {
            id,
            emotes,
            first_msg,
            returning_chatter,
            subscriber,
            moderator,
            color,
            badges: badges_symbols.unwrap_or_default(),
            message: irc_message.parameters.to_string(),
            nickname: irc_message.sender.to_string(),
            channel: irc_message.channel.to_string(),
            raw: irc_message.raw.to_string(),
            area: None,
        },
    }
}

fn parse_usernotice(message: IrcMessage) -> TwitchMessage {
    let mut system_msg = String::new();
    let mut is_raid = false;
    let mut user_id = String::new();
    let mut display_name = String::new();

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

        if tag == "msg-param-displayName" {
            display_name = value.to_string();
        }
    }

    if is_raid && !system_msg.is_empty() {
        let message = RaidMessage {
            area: None,
            raid_notice: system_msg,
            user_id,
            display_name,
        };

        return TwitchMessage::RaidMessage { message };
    }

    TwitchMessage::UnknownMessage {
        message: message.raw.to_string(),
    }
}
