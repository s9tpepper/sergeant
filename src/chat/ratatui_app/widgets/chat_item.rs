use ratatui::{
    prelude::{Buffer, Rect},
    widgets::Widget,
};

use crate::chat::ratatui_app::ChatItem;

impl Widget for ChatItem {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // NOTE: first_msg is not available in EventSub yet - 03/2025
        // let needs_borders = self.first_msg || is_animated;
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
