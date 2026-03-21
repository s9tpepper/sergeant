use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use log::info;
use ratatui::{
    layout::Position,
    prelude::{Buffer, Rect},
    style::Color,
    widgets::Widget,
};

use crate::{
    chat::ratatui_app::{
        ChatItem,
        widgets::{chat_event::get_lines, get_color, handle_emote, handle_mention, handle_text},
    },
    twitch::{assets::get_badge_from_disk, eventsub::deserialization::FragmentType},
};

use super::{Style, get_iterm_encoding, write_symbol};

static IMAGE_MAP: LazyLock<RwLock<HashMap<String, String>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn calculate_badges_space(chat_item: &ChatItem) -> i32 {
    let mut space = 0;
    chat_item.badge_items.iter().for_each(|_| {
        space += 2;
    });

    space
}

// TODO: Cache these so we're not reading from disc every time
fn write_user_badges(chat_item: &ChatItem, cursor: &mut Position, buffer: &mut Buffer) {
    for badge_item in chat_item.badge_items.iter() {
        let Some(version) = badge_item.versions.first() else {
            continue;
        };
        let badge_file_name = format!("{}_{}.txt", badge_item.set_id, version.id);

        let Ok(read_lock) = IMAGE_MAP.read() else {
            continue;
        };

        let encoding: &Cow<str> = if read_lock.contains_key(&badge_file_name) {
            let enc = read_lock.get(&badge_file_name).expect("already checked it exists");
            &Cow::Borrowed(enc)
        } else {
            drop(read_lock);
            let Ok(mut write_lock) = IMAGE_MAP.write() else {
                return;
            };

            let badge = get_badge_from_disk(badge_item).unwrap_or_default();
            write_lock.insert(badge_file_name, badge.clone());

            &Cow::Owned(badge)
        };

        if encoding.is_empty() {
            continue;
        }

        let Some(cell) = buffer.cell_mut(*cursor) else {
            return;
        };

        cell.reset();
        cell.set_symbol(&get_iterm_encoding(encoding, None, None));

        buffer
            .cell_mut((cursor.x + 1, cursor.y))
            .map(|cell| cell.set_skip(true));

        cursor.x += 2;
    }
}

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

fn calculate_user_name_space(chat_item: &ChatItem) -> i32 {
    // NOTE: adds two to account for ": " in username display, like:
    // s9tpepper_: Message here
    let username_separator = 2;

    chat_item.chatter_user_name.len() as i32 + username_separator
}

fn write_user_name(chat_item: &ChatItem, style: &mut Style, cursor: &mut Position, buf: &mut Buffer) {
    chat_item.chatter_user_name.chars().for_each(|char| {
        write_symbol(&char.to_string(), style, cursor, buf);
    });

    style.fg = Color::White;
    write_symbol(":", style, cursor, buf);
    write_symbol(" ", style, cursor, buf);
}

impl Widget for &mut ChatItem {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // info!("chat_item:render()");

        // NOTE: first_msg is not available in EventSub yet - 03/2025
        // let needs_borders = self.first_msg || is_animated;
        let mut style = Style {
            fg: get_color(&self.color).unwrap_or(Color::LightGreen),
            bg: None,
        };

        // TODO: Refactor write_user_badges/write_user_name to get a count of how many columns
        // these are going to take so that column count can be used to take into account when
        // calling get_line_count() so that the line count is accurate and includes the user's
        // badges and username

        let badge_space = calculate_badges_space(self);
        let username_space = calculate_user_name_space(self);
        let name_display_space = badge_space + username_space;

        let line_width = area.width.saturating_sub(1) as usize;

        // let fragments = std::mem::take(&mut self.message.fragments);
        let lines = get_lines(&self.message.fragments, line_width, Some(name_display_space as usize));

        let number_of_lines = lines.len();

        let mut cursor = Position::new(0, area.height.saturating_sub(number_of_lines as u16));

        write_user_badges(self, &mut cursor, buf);
        write_user_name(self, &mut style, &mut cursor, buf);

        // info!("[chat_item::render()] line_width: {line_width}");

        // TODO: re-use the instance that's already created instead of making
        // a new one every time we render
        // let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).expect("No TUI");
        for line in lines {
            info!("Rendering line: {line:?}");

            // NOTE: This block tries to clear the line where an emote needs to be rendered
            // to attempt fixing artifacts behind the emote, didn't work 100%
            // if has_emote(&line) {
            // let _ = terminal.set_cursor_position(Position { x: 0, y: cursor.y });
            // let _ = terminal
            //     .backend_mut()
            //     .clear_region(ratatui::backend::ClearType::CurrentLine);
            // }

            line.iter().for_each(|fragment| match fragment.r#type {
                FragmentType::Text => handle_text(line_width as u16, fragment, &style, &mut cursor, buf),

                // TODO: Implement emotes
                FragmentType::Cheermote => {}

                FragmentType::Emote => handle_emote(fragment, &mut cursor, buf),

                FragmentType::Mention => handle_mention(fragment, &mut cursor, buf),

                FragmentType::Unknown => {
                    unreachable!("We should never have an unknown fragment type");
                }
            });

            cursor.x = 0;
            cursor.y += 1;
        }

        self.area = area;

        self.area.height = self.area.height.saturating_sub(number_of_lines as u16);
    }
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
