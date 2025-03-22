use ratatui::{layout::Size, widgets::StatefulWidget};

use crate::chat::ratatui_app::{ChatLogItem, RatatuiApp};

use super::scroll_view::ScrollViewState;

impl StatefulWidget for &mut RatatuiApp {
    type State = ScrollViewState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        buf.reset();

        let content_size = Size {
            // Subtract one to avoid getting horizontal scrollbar from tui-scrollview
            width: area.width.saturating_sub(1),
            height: area.height * 2, // space to scroll the text up
        };

        self.scrollview.resize(content_size);

        let mut available_area = area;
        available_area.height = content_size.height;

        if self.chat_log.is_empty() {
            state.scroll_to_bottom();
        }

        self.chat_log.iter_mut().for_each(|chat_log_item| match chat_log_item {
            ChatLogItem::Message(chat_item) => {
                println!("Rendering chat item");
            }

            ChatLogItem::MessageWithEffect(chat_item_with_effect) => {
                println!("Rendering chat item with effect");
            }
        });

        self.scrollview.render(buf.area, buf, state);
    }
}
