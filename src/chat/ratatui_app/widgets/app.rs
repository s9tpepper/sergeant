use ratatui::{
    layout::Size,
    prelude::{Buffer, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::chat::ratatui_app::{ChatLogItem, RatatuiApp};

use super::scroll_view::ScrollViewState;

impl StatefulWidget for &mut RatatuiApp {
    type State = ScrollViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.reset();

        let content_size = Size {
            // Subtract one to avoid getting horizontal scrollbar from tui-scrollview
            width: area.width.saturating_sub(1),

            // TODO: Return this height to * 2 after fixing scrollview
            height: area.height, // space to scroll the text up
                                 // height: area.height * 2, // space to scroll the text up
        };

        self.scrollview.resize(content_size);

        let mut available_area = area;
        available_area.height = content_size.height;

        self.chat_log.iter_mut().for_each(|chat_log_item| match chat_log_item {
            ChatLogItem::Message(chat_item) => {
                chat_item.render(available_area, buf);

                available_area = chat_item.area;
            }

            // TODO: Implement messages with effects
            ChatLogItem::MessageWithEffect(_chat_item_with_effect) => {
                println!("Rendering chat item with effect");
            }
        });

        // TODO: Fix scrolling in scrollview
        // if !self.chat_log.is_empty() {
        state.scroll_to_bottom();
        // }

        // self.scrollview.render(buf.area, buf, state);
    }
}
