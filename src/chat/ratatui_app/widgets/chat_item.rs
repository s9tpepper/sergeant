use std::{env, str::FromStr};

use base64::{prelude::BASE64_STANDARD, Engine};
use ratatui::{
    layout::Position,
    prelude::{Buffer, Rect},
    style::Color,
    widgets::Widget,
};

use crate::{
    chat::ratatui_app::ChatItem,
    twitch::eventsub::deserialization::{Emote, Fragment, FragmentType},
};

const ESCAPE: &str = "\x1b";
const BELL: &str = "\x07";

fn get_color(color: &str) -> anyhow::Result<Color> {
    Ok(Color::from_str(color)?)
}

/*
Badge { set_id: "broadcaster", id: "1", info: "" }
Badge { set_id: "subscriber", id: "0", info: "14" }
Badge { set_id: "share-the-love", id: "1", info: "" }
*/
// TODO: Finish badges, must integrate API calls for badge list
fn write_user_badges(chat_item: &ChatItem, cursor: &mut Position, buffer: &mut Buffer) {
    chat_item.badges.iter().for_each(|badge| {});
}

// Loading badge info from API
/*
    let response = ureq::get("https://api.twitch.tv/helix/chat/badges/global")
        .set("Authorization", &format!("Bearer {}", token.replace("oauth:", "")))
        .set("Client-Id", client_id)
        .call()?;

    let mut response: TwitchApiResponse<Vec<BadgeItem>> = serde_json::from_reader(response.into_reader())?;

    let data_dir = get_data_directory(Some("badges"))?;

    for badge_item in response.data.iter_mut() {
        for version in badge_item.versions.iter_mut() {
            let file_name = format!("{}_{}.txt", badge_item.set_id, version.id);
            let badge_path = data_dir.join(file_name);
            if !badge_path.exists() {
                generate_badge_file(badge_path, version)?;
            }
        }
    }
*/

// Emote caching from V1
/*
        let cache = EMOTE_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        let cache_read = cache.read().unwrap();
        if let Some(encoding) = cache_read.get(&self.emote_id) {
            self.encoded = Some(encoding.to_string());

            return Ok(());
        }
        drop(cache_read);

        let mut cache_write = cache.write().unwrap();
        cache_write.insert(self.emote_id.clone(), encoded_image);

*/

fn write_user_name(chat_item: &ChatItem, style: &mut Style, cursor: &mut Position, buf: &mut Buffer) {
    chat_item.chatter_user_name.chars().for_each(|char| {
        write_symbol(&char.to_string(), style, cursor, buf);
    });

    style.fg = Color::White;
    write_symbol(":", style, cursor, buf);
    write_symbol(" ", style, cursor, buf);
}
// TODO: Add an emote cache for encoded emotes so that we dont keep downloading them from the web
// TODO: Investigate whether we can query the terminal for iTerm/Kitty/Sixel Image Protocol support
fn write_emote(emote: &Emote, cursor: &mut Position, buf: &mut Buffer) -> anyhow::Result<()> {
    let url = format!(
        "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0",
        emote.id
    );

    let response = ureq::get(&url).call()?;
    let length: usize = response.header("content-length").unwrap().parse()?;
    let mut file_bytes: Vec<u8> = vec![0; length];
    response.into_reader().read_exact(&mut file_bytes)?;

    let base64_emote = BASE64_STANDARD.encode(&file_bytes);
    let encoded_image = format!(
        // "{}1337;File=inline=1;height=22px;width=22px;preserveAspectRatio=1;doNotMoveCursor=1:{}{}",
        "{}]1337;File=inline=1;height=22px;width=22px;doNotMoveCursor=1:{}{}",
        ESCAPE,
        base64_emote.as_str(),
        BELL
    );

    let Some(cell) = buf.cell_mut(*cursor) else {
        return Ok(());
    };

    cell.reset();
    cell.set_symbol(&encoded_image);

    buf.cell_mut((cursor.x + 1, cursor.y)).map(|cell| cell.set_skip(true));

    cursor.x += 2;

    Ok(())
}

impl Widget for &mut ChatItem {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // NOTE: first_msg is not available in EventSub yet - 03/2025
        // let needs_borders = self.first_msg || is_animated;

        let line_width = area.width.saturating_sub(1);
        let number_of_lines = get_line_count(&self.message.text, &area);
        let mut cursor = Position::new(0, area.height.saturating_sub(number_of_lines as u16));

        let mut style = Style {
            fg: get_color(&self.color).unwrap_or(Color::LightGreen),
            bg: None,
        };

        write_user_badges(self, &mut cursor, buf);
        write_user_name(self, &mut style, &mut cursor, buf);

        self.message
            .fragments
            .iter()
            .for_each(|fragment| match fragment.r#type {
                FragmentType::Text => handle_text(line_width, fragment, &style, &mut cursor, buf),

                // TODO: Implement emotes
                FragmentType::Cheermote => {}

                FragmentType::Emote => handle_emote(fragment, &mut cursor, buf),

                FragmentType::Mention => {}

                FragmentType::Unknown => {
                    unreachable!("We should never have an unknown fragment type");
                }
            });

        self.area = area;
        self.area.height -= number_of_lines as u16;
    }
}

fn handle_text(line_width: u16, fragment: &Fragment, style: &Style, cursor: &mut Position, buf: &mut Buffer) {
    fragment.text.chars().for_each(|char| {
        if cursor.x == line_width {
            cursor.x = 0;
            cursor.y += 1;
        }

        write_symbol(&char.to_string(), style, cursor, buf);
    });
}

#[derive(Debug, PartialEq, Eq)]
enum ImageProtocol {
    Iterm,
    Sixel,
    Kitty,
    None,
}

fn supports_images() -> ImageProtocol {
    match env::var("TERM_PROGRAM") {
        Ok(term_program) => match term_program.as_str() {
            "WezTerm" => ImageProtocol::Iterm,
            "ghostty" => ImageProtocol::Kitty,
            _ => ImageProtocol::None,
        },
        Err(_) => match env::var("TERM") {
            Ok(term) => match term.as_str() {
                "xterm-kitty" => ImageProtocol::Kitty,
                "alacritty" => ImageProtocol::Kitty,
                "xterm" => ImageProtocol::Sixel,
                _ => ImageProtocol::None,
            },
            Err(_) => ImageProtocol::None,
        },
    }
}

fn handle_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {
    match supports_images() {
        ImageProtocol::Iterm => write_iterm_emote(fragment, cursor, buf),
        ImageProtocol::Kitty => write_kitty_emote(fragment, cursor, buf),

        _ => {} // ImageProtocol::Sixel => todo!(),
                // ImageProtocol::Kitty => todo!(),
                // ImageProtocol::None => todo!(),
    }
}

// TODO: Maybe render these images with ratatui-image
fn write_kitty_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {}

fn write_iterm_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {
    let Some(emote) = &fragment.emote else {
        return;
    };

    let _ = write_emote(emote, cursor, buf);
}

struct Style {
    fg: Color,
    bg: Option<Color>,
}

fn write_symbol(symbol: &str, style: &Style, cursor: &mut Position, buffer: &mut Buffer) {
    let Some(cell) = buffer.cell_mut(*cursor) else {
        return;
    };

    cursor.x += 1;
    cell.reset();

    cell.set_symbol(symbol).set_fg(style.fg);

    if let Some(color) = style.bg {
        cell.set_bg(color);
    }
}

fn get_line_count(text: &str, area: &Rect) -> usize {
    if text.is_empty() {
        return 1;
    }

    // Subtract one to account for the scrollbar rendering
    let width = area.width.saturating_sub(1) as usize;

    text.len().div_ceil(width)
}

#[test]
fn test_get_line_count() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "aaa aaaaa aaaa";

    let line_count = get_line_count(text, &area);

    assert_eq!(line_count, 2);
}

#[test]
fn test_get_line_count2() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "aaa ";

    let line_count = get_line_count(text, &area);

    assert_eq!(line_count, 1);
}

#[test]
fn test_get_line_count3() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "";

    let line_count = get_line_count(text, &area);

    assert_eq!(line_count, 1);
}

#[test]
fn test_get_line_count4() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "1234567890";

    let line_count = get_line_count(text, &area);

    assert_eq!(line_count, 2);
}

#[test]
fn test_get_line_count5() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "123456789";

    let line_count = get_line_count(text, &area);

    assert_eq!(line_count, 1);
}

#[test]
fn test_supports_images_wezterm() {
    let original_term_program_var = env::var("TERM_PROGRAM");
    let original_term_var = env::var("TERM");

    env::set_var("TERM_PROGRAM", "WezTerm");

    let images_support = supports_images();

    if let Ok(og_var) = original_term_program_var {
        env::set_var("TERM_PROGRAM", og_var);
    }
    if let Ok(og_var) = original_term_var {
        env::set_var("TERM", og_var);
    }

    assert_eq!(images_support, ImageProtocol::Iterm);
}

#[test]
fn test_supports_images_xterm() {
    let original_term_program_var = env::var("TERM_PROGRAM");
    let original_term_var = env::var("TERM");

    env::remove_var("TERM_PROGRAM");
    env::set_var("TERM", "xterm");

    let images_support = supports_images();

    if let Ok(og_var) = original_term_program_var {
        env::set_var("TERM_PROGRAM", og_var);
    }
    if let Ok(og_var) = original_term_var {
        env::set_var("TERM", og_var);
    }

    assert_eq!(images_support, ImageProtocol::Sixel);
}

#[test]
fn test_supports_images_alacritty() {
    let original_term_program_var = env::var("TERM_PROGRAM");
    let original_term_var = env::var("TERM");

    env::remove_var("TERM_PROGRAM");
    env::set_var("TERM", "alacritty");

    let images_support = supports_images();

    if let Ok(og_var) = original_term_program_var {
        env::set_var("TERM_PROGRAM", og_var);
    }
    if let Ok(og_var) = original_term_var {
        env::set_var("TERM", og_var);
    }

    assert_eq!(images_support, ImageProtocol::Kitty);
}

#[test]
fn test_supports_images_ghostty() {
    let original_term_program_var = env::var("TERM_PROGRAM");
    let original_term_var = env::var("TERM");

    env::set_var("TERM_PROGRAM", "ghostty");

    let images_support = supports_images();

    if let Ok(og_var) = original_term_program_var {
        env::set_var("TERM_PROGRAM", og_var);
    }
    if let Ok(og_var) = original_term_var {
        env::set_var("TERM", og_var);
    }

    assert_eq!(images_support, ImageProtocol::Kitty);
}

#[test]
fn test_supports_images_kitty() {
    let original_term_program_var = env::var("TERM_PROGRAM");
    let original_term_var = env::var("TERM");
    env::remove_var("TERM");
    env::remove_var("TERM_PROGRAM");

    env::set_var("TERM", "xterm-kitty");

    let images_support = supports_images();

    if let Ok(og_var) = original_term_program_var {
        env::set_var("TERM_PROGRAM", og_var);
    }
    if let Ok(og_var) = original_term_var {
        env::set_var("TERM", og_var);
    }

    assert_eq!(images_support, ImageProtocol::Kitty);
}

#[test]
fn test_supports_images_no_term_program() {
    let original_term_program_var = env::var("TERM_PROGRAM");
    let original_term_var = env::var("TERM");
    env::remove_var("TERM");
    env::remove_var("TERM_PROGRAM");

    let images_support = supports_images();

    if let Ok(og_var) = original_term_program_var {
        env::set_var("TERM_PROGRAM", og_var);
    }

    if let Ok(og_var) = original_term_var {
        env::set_var("TERM", og_var);
    }

    assert_eq!(images_support, ImageProtocol::None);
}

// TODO: Migrate the old render code below to new ratatui
//
// // Initialize the cursor position
//         let mut cursor = RenderCursor {
//             x: area.left(),
//             y: area.bottom(),
//         };
//
//         let is_animated = self.animation_id != *"";
//         let needs_borders = self.first_msg || is_animated;
//
//         // NOTE: Used to test first time chatter decoration
//         // let needs_borders = self.message.len() % 2 > 0;
//
//         let symbols: Vec<Symbol> = self.get_symbols();
//
//         let mut line_area = area;
//         line_area.width = if needs_borders { area.width - 4 } else { area.width };
//         let mut screen_lines: Vec<Vec<MessageParts>> = get_lines(&symbols, &line_area);
//
//         let y_pos = cursor.y.saturating_sub(screen_lines.len() as u16);
//         cursor.x = if needs_borders { area.left() + 1 } else { area.left() };
//         cursor.y = if needs_borders { y_pos.saturating_sub(1) } else { y_pos };
//
//         let mut writeable_area = area;
//         writeable_area.width = if needs_borders { area.width - 1 } else { area.width };
//         writeable_area.height = if needs_borders {
//             screen_lines.len() as u16 + 2
//         } else {
//             screen_lines.len() as u16
//         };
//
//         write_to_buffer(&mut screen_lines, buf, &mut cursor);
//
//         // Reset cursor position after writing to buffer
//         cursor.x = 0;
//         cursor.y = cursor.y.saturating_sub(writeable_area.height) + 1;
//
//         if needs_borders {
//             let icons: Vec<&str> = vec![
//                 "━━━━━━━🔷━━━🦄━━🔴━━━━💜━━🐶━",
//                 "━━🔷━━━🐶━━━━━━💜━━🔴━━━🦄━━━",
//                 "━━━━━━💜━━━━🔴━━━🦄━━🔷━━━🐶━",
//                 "━━🦄━━━━━🔷━━━🐶━━🔴━━━━💜━━━",
//                 "━━━━━🐶━━🔴━━━━━💜━━🔷━━━━━🦄",
//             ];
//
//             let title = if self.first_msg {
//                 "✨First time chatter"
//             } else if self.animation_id == "simmer" {
//                 let now = SystemTime::now();
//                 let time: OffsetDateTime = now.into();
//                 let seconds = time.second();
//                 let index = (seconds as usize) % icons.len();
//
//                 icons.get(index).unwrap()
//             } else {
//                 ""
//             };
//
//             let border_style = if self.first_msg && !is_animated {
//                 Style::reset().fg(Color::Rgb(255, 255, 0))
//             } else if self.animation_id == "rainbow-eclipse" {
//                 if self.direction == 1 {
//                     self.r = self.r.wrapping_add(1);
//                 } else {
//                     self.r = self.r.wrapping_sub(1);
//                 }
//
//                 if self.r == 255 {
//                     self.direction = -1;
//                 } else if self.r == 0 {
//                     self.direction = 1;
//                 }
//                 Style::reset().fg(Color::Rgb(self.r, self.g, self.b))
//             } else {
//                 Style::reset().fg(Color::Rgb(255, 255, 0))
//             };
//
//             let border_type = if is_animated {
//                 symbols::border::THICK
//             } else {
//                 symbols::border::ROUNDED
//             };
//
//             Block::bordered()
//                 .border_set(border_type)
//                 .border_style(border_style)
//                 .title(title)
//                 .render(
//                     Rect {
//                         x: cursor.x,
//                         y: cursor.y,
//                         width: area.width - 2,
//                         height: screen_lines.len() as u16 + 2,
//                     },
//                     buf,
//                 );
//         }
//
//         // Update the area this message takes
//         self.area = Some(Rect {
//             x: 0,
//             y: cursor.y,
//             width: area.width,
//             height: writeable_area.height,
//         });
