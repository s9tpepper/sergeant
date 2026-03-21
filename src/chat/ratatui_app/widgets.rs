use base64::{Engine, prelude::BASE64_STANDARD};
use kitty_graphics_protocol::{Action, check_protocol_support, get_window_size};
use log::{error, info};
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
};
use std::{
    collections::HashMap,
    env,
    io::{self, Write},
    str::{Chars, FromStr},
    sync::{LazyLock, RwLock},
};

use crate::twitch::eventsub::deserialization::{Emote, Fragment};

pub mod app;
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

        _ => {
            info!("No Image Protocol match for this terminal")
        } // ImageProtocol::Sixel => todo!(),
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
        // info!("[chat_item::handle_text()] Rendering fragment char: {char}");
        // info!(
        //     "[chat_item::handle_text()] x: {}, y: {}, line_width: {line_width:?}",
        //     cursor.x, cursor.y
        // );

        if let Some(width) = line_width
            && cursor.x == width
        {
            cursor.x = 0;
            cursor.y += 1;
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
    let _buffer_area = buffer.area();

    // info!("[chat_item::write_symbol()] symbol: {symbol}, cursor: {cursor} buffer_area: {buffer_area}");

    let Some(cell) = buffer.cell_mut(*cursor) else {
        //error!("Could not get mutable cell to write symbol: {symbol}");
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
fn write_kitty_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {
    let Some(emote) = &fragment.emote else {
        error!("[ratatui_app/widgets.rs] Unable to unwrap fragment.emote");

        return;
    };

    match kitty_emote(emote, cursor, buf) {
        Ok(_) => info!("[ratatui_app/widgets.rs] Kitty emote written successfully"),
        Err(error) => error!("[ratatui_app/widgets.rs] Error writing Kitty Protocol emotes: {error}"),
    }
}

fn write_iterm_emote(fragment: &Fragment, cursor: &mut Position, buf: &mut Buffer) {
    let Some(emote) = &fragment.emote else {
        error!("[ratatui_app/widgets.rs] Unable to unwrap fragment.emote");

        return;
    };

    if let Err(error) = iterm_emote(emote, cursor, buf) {
        error!("[ratatui_app/widgets.rs] Error writing emotes: {error}");
    }
}

static IMAGE_MAP: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn iterm_emote(emote: &Emote, cursor: &mut Position, buf: &mut Buffer) -> anyhow::Result<()> {
    let url = format!(
        "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0",
        emote.id
    );

    let image_map_reader_lock = IMAGE_MAP.read();
    if let Ok(image_map_reader) = &image_map_reader_lock
        && let Some(encoding) = image_map_reader.get(&url)
    {
        set_iterm_encoding_to_cell(buf, cursor, encoding);
    } else {
        drop(image_map_reader_lock);

        let response = ureq::get(&url).call()?;
        let length: usize = response.header("content-length").unwrap().parse()?;
        let mut file_bytes: Vec<u8> = vec![0; length];
        response.into_reader().read_exact(&mut file_bytes)?;

        let base64_emote = BASE64_STANDARD.encode(&file_bytes);
        let encoded_image = get_iterm_encoding(&base64_emote, Some("42"), Some("42"));

        set_iterm_encoding_to_cell(buf, cursor, &encoded_image);

        let mut image_map_write_lock = IMAGE_MAP.write();
        if let Ok(ref mut image_map_writer) = image_map_write_lock {
            image_map_writer.insert(url, encoded_image);
        }

        drop(image_map_write_lock);
    }

    Ok(())
}

// NOTE: This did not work.
pub fn clear_cell(row: u16, column: u16) -> io::Result<()> {
    // let character = ' ';

    // let character = "\x033[X";

    // The full escape sequence: ESC[Y;XH + character
    // Use \x1b for the escape character (ASCII 27 or 0x1B)
    // let sequence = format!("\x1b[{};{}H{}", row, column, character);
    // let sequence1 = format!("\x1b[{};{}H{}", row, column + 1, character);
    // let sequence2 = format!("\x1b[{};{}H{}", row, column + 1, character);

    // let sequence = format!("\x1B[{};{}H\x1B[X", row, column);
    //
    let sequence = format!("\x1B[{};{}H\x1B[2K", row, column);
    // let sequence1 = format!("\x1B[{};{}H\x1B[X", row, column + 1);
    // let sequence2 = format!("\x1B[{};{}H\x1B[X", row, column + 2);

    // Get a handle to stdout
    let mut stdout = io::stdout();

    // Write the escape sequence and character to the terminal
    write!(stdout, "{}", sequence)?;
    // write!(stdout, "{}", sequence1)?;
    // write!(stdout, "{}", sequence2)?;

    // Flush stdout to ensure the output is displayed immediately
    stdout.flush()?;

    // Optional: Move cursor back to a safe location (e.g., home position)
    // write!(stdout, "\x1b[H")?;
    // stdout.flush()?;

    Ok(())
}

fn set_iterm_encoding_to_cell(buf: &mut Buffer, cursor: &mut Position, encoded_image: &str) {
    let Some(cell) = buf.cell_mut(*cursor) else {
        return;
    };

    // cell.reset();
    // cell.set_bg(Color::Black);
    // cell.set_fg(Color::Black);

    let _ = clear_cell(cursor.y, cursor.x);
    let _ = clear_cell(cursor.y, cursor.x + 1);
    let _ = clear_cell(cursor.y, cursor.x + 2);

    cell.set_symbol(encoded_image);

    // #[allow(clippy::option_map_unit_fn)]
    // buf.cell_mut((cursor.x + 1, cursor.y)).map(|cell| {
    //     cell.reset();
    //     cell.set_symbol(" ");
    // });

    cursor.x += 1;
}

fn kitty_emote(emote: &Emote, cursor: &mut Position, buf: &mut Buffer) -> anyhow::Result<()> {
    if check_protocol_support().is_err() {
        error!("Terminal does not support Kitty Image Protocol");
        return Ok(());
    }

    let Ok(terminal_info) = get_window_size() else {
        error!("Could not get terminal info for Kitty Image Protocol terminal");
        return Ok(());
    };

    let url = format!(
        "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0",
        emote.id
    );

    let response = ureq::get(&url).call()?;
    let length: usize = response.header("content-length").unwrap().parse()?;
    let mut file_bytes: Vec<u8> = vec![0; length];
    response.into_reader().read_exact(&mut file_bytes)?;

    let cell_width = terminal_info.cell_width();
    let cell_height = terminal_info.cell_height();

    info!("cell width: {cell_width}, cell_height: {cell_height}");

    let h = (cursor.x + 1) * cell_width;
    let v = cursor.y * cell_height;

    info!("h: {h}, v: {v}");

    let cmd = kitty_graphics_protocol::Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(kitty_graphics_protocol::ImageFormat::Png)
        // .quiet(2)
        .z_index(0)
        .display_area(2, 1)
        .dimensions(40, 40)
        // .source_rect(20, 20, 40, 40)
        // .unicode_placeholder(1, 0)
        //
        // .parent(image_id, placement_id)
        // .relative_offset(100, 0)
        //
        // .display_area(terminal_info.cols.into(), terminal_info.rows.into())
        // .cell_offset(v.into(), h.into())
        // .cell_offset(30, 0)
        .build();
    let chunks: Vec<String> = cmd.serialize_chunked(&file_bytes).unwrap().collect();
    let mut stdout = std::io::stdout().lock();
    for chunk in chunks {
        stdout.write_all(chunk.as_bytes()).unwrap();
    }
    stdout.flush().unwrap();

    // let display = ImageDisplay::new();
    //
    // display.transmit_png(&file_bytes, 123).unwrap();
    //
    // // Display the same image multiple times at different positions
    // display.place_image(123, 10, 5).unwrap(); // column 10, row 5
    //                                           // display.place_image(123, 20, 5).unwrap(); // column 20, row 5

    // Clear images
    // display.clear_all().unwrap();

    info!("Finished Kitty Emote rendering");

    cursor.x += 1;

    Ok(())
}

// let tmux = term_misc::get_wininfo().is_tmux;
//     let prefix = if tmux { "\x1bPtmux;\x1b\x1b" } else { "\x1b" };
//     let suffix = if tmux { "\x1b\x07\x1b\\" } else { "\x07" };
//
//     write!(
//         out,
//         "{prefix}]1337;File=inline=1;size={}:{base64_encoded}{suffix}",
//         base64_encoded.len()
//     )?;
//
//     Ok(())

pub fn get_iterm_encoding(base64: &str, width: Option<&str>, height: Option<&str>) -> String {
    let w = width.unwrap_or("44");
    let h = height.unwrap_or("44");

    format!(
        // "{}1337;File=inline=1;height=22px;width=22px;preserveAspectRatio=1;doNotMoveCursor=1:{}{}",
        //
        "{}]1337;File=inline=1;height={h}px;width={w}px;doNotMoveCursor=1:{}{}",
        //
        // "{}]1337;File=inline=1;size={}px:{}{}",
        ESCAPE,
        // base64.len(),
        base64,
        BELL
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
