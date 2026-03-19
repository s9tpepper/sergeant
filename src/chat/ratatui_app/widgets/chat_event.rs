use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
    widgets::Widget,
};

use crate::{
    chat::ratatui_app::{
        widgets::{get_color, get_line_count, handle_emote, handle_text, Style},
        ChatEvent,
    },
    twitch::eventsub::deserialization::FragmentType,
};

impl Widget for &mut ChatEvent {
    fn render(self, writeable_area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // NOTE: first_msg is not available in EventSub yet - 03/2025
        // let needs_borders = self.first_msg || is_animated;

        let line_width = writeable_area.width.saturating_sub(1);
        let number_of_lines = get_line_count(&self.message.text, &writeable_area, None) + 1;

        let y = writeable_area.height.saturating_sub(number_of_lines as u16);
        let mut cursor = Position::new(0, y);

        let style = Style {
            // TODO: Use some math on user's screen name to calculate a color based on their name
            // to default to if one is not set instead of defaulting all users to LightGreen
            fg: get_color(&self.color).unwrap_or(Color::LightGreen),
            bg: None,
        };

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

        self.area = writeable_area;
        self.area.height = self.area.height.saturating_sub(number_of_lines as u16);
    }
}
