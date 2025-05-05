use base64::{prelude::BASE64_STANDARD, Engine};
use log::{error, info};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
};
use std::{
    env,
    str::{Chars, FromStr},
};

use crate::twitch::eventsub::deserialization::{Emote, Fragment};

mod app;
pub mod chat_event;
pub mod chat_item;
pub mod scroll_view;

const ESCAPE: &str = "\x1b";
const BELL: &str = "\x07";

pub struct Style {
    fg: Color,
    bg: Option<Color>,
}

pub fn get_color(color: &str) -> anyhow::Result<Color> {
    Ok(Color::from_str(color)?)
}

pub fn get_line_count(text: &str, area: &Rect, name_display_space: Option<i32>) -> usize {
    if text.is_empty() {
        return 1;
    }

    // Subtract one to account for the scrollbar rendering
    let width = area.width.saturating_sub(1) as usize;

    let name_display_space = name_display_space.unwrap_or(0) as usize;

    (text.len() + name_display_space).div_ceil(width)
}

pub fn handle_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {
    match supports_images() {
        ImageProtocol::Iterm => write_iterm_emote(fragment, cursor, buf),
        ImageProtocol::Kitty => write_kitty_emote(fragment, cursor, buf),

        _ => {} // ImageProtocol::Sixel => todo!(),
                // ImageProtocol::Kitty => todo!(),
                // ImageProtocol::None => todo!(),
    }
}

pub fn handle_mention(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {
    let Some(mention) = &fragment.mention else {
        return;
    };

    let style = Style {
        fg: Color::White,
        bg: Some(Color::Black),
    };

    write_symbol("@", &style, cursor, buf);
    write_symbols(mention.user_name.chars(), &style, cursor, buf, None);
}

fn write_symbols(chars: Chars, style: &Style, cursor: &mut Position, buf: &mut Buffer, line_width: Option<u16>) {
    chars.for_each(|char| {
        info!("[chat_item::handle_text()] Rendering fragment char: {char}");
        info!(
            "[chat_item::handle_text()] x: {}, y: {}, line_width: {line_width:?}",
            cursor.x, cursor.y
        );

        if let Some(width) = line_width {
            if cursor.x == width {
                cursor.x = 0;
                cursor.y += 1;
            }
        }

        write_symbol(&char.to_string(), style, cursor, buf);
    });
}

pub fn handle_text(line_width: u16, fragment: &Fragment, style: &Style, cursor: &mut Position, buf: &mut Buffer) {
    info!("[chat_item::handle_text()]");
    info!(
        "[chat_item::handle_text()] fragment.text: {}, cursor: {cursor}",
        fragment.text
    );

    write_symbols(fragment.text.chars(), style, cursor, buf, Some(line_width));
}

pub fn write_symbol(symbol: &str, style: &Style, cursor: &mut Position, buffer: &mut Buffer) {
    let buffer_area = buffer.area();

    info!("[chat_item::write_symbol()] symbol: {symbol}, cursor: {cursor} buffer_area: {buffer_area}");

    let Some(cell) = buffer.cell_mut(*cursor) else {
        error!("Could not get mutable cell to write symbol: {symbol}");
        return;
    };

    cell.reset();

    cell.set_symbol(symbol).set_fg(style.fg);

    if let Some(color) = style.bg {
        cell.set_bg(color);
    }

    // info!("Wrote symbol '{symbol} to x: {}, y: {}", cursor.x, cursor.y);

    cursor.x += 1;
}

// TODO: Maybe render these images with ratatui-image
#[allow(unused)]
fn write_kitty_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {}

fn write_iterm_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {
    let Some(emote) = &fragment.emote else {
        return;
    };

    let _ = write_emote(emote, cursor, buf);
}

// TODO: Add an emote cache for encoded emotes so that we dont keep downloading them from the web
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
    let encoded_image = get_iterm_encoding(&base64_emote);

    let Some(cell) = buf.cell_mut(*cursor) else {
        return Ok(());
    };

    cell.reset();

    cell.set_bg(Color::Black);
    cell.set_fg(Color::Black);

    cell.set_symbol(&encoded_image);

    buf.cell_mut((cursor.x + 1, cursor.y)).map(|cell| cell.set_skip(true));

    cursor.x += 2;

    Ok(())
}

pub fn get_iterm_encoding(base64: &str) -> String {
    format!(
        // "{}1337;File=inline=1;height=22px;width=22px;preserveAspectRatio=1;doNotMoveCursor=1:{}{}",
        "{}]1337;File=inline=1;height=44px;width=44px;doNotMoveCursor=1:{}{}",
        ESCAPE, base64, BELL
    )
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

#[test]
fn test_get_line_count() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "aaa aaaaa aaaa";

    let line_count = get_line_count(text, &area, None);

    assert_eq!(line_count, 2);
}

#[test]
fn test_get_line_count2() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "aaa ";

    let line_count = get_line_count(text, &area, None);

    assert_eq!(line_count, 1);
}

#[test]
fn test_get_line_count3() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "";

    let line_count = get_line_count(text, &area, None);

    assert_eq!(line_count, 1);
}

#[test]
fn test_get_line_count4() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "1234567890";

    let line_count = get_line_count(text, &area, None);

    assert_eq!(line_count, 2);
}

#[test]
fn test_get_line_count5() {
    let area = Rect::new(0, 0, 10, 1);
    let text = "123456789";

    let line_count = get_line_count(text, &area, None);

    assert_eq!(line_count, 1);
}

#[test]
fn test_get_line_count6() {
    let area = Rect::new(0, 0, 12, 1);
    let text = "123456789";

    let line_count = get_line_count(text, &area, Some(3));

    assert_eq!(line_count, 2);
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
