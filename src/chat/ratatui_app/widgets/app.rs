use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use log::info;
use ratatui::{
    layout::Size,
    prelude::{Buffer, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::chat::ratatui_app::{widgets::scroll_view::ScrollView, ChatLogItem, RatatuiApp};

pub struct AppWidget<'a> {
    pub phantom: PhantomData<&'a mut RatatuiApp>,
}

impl<'a> StatefulWidget for AppWidget<'a> {
    type State = Arc<Mutex<&'a mut RatatuiApp>>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        buf.reset();

        let content_size = Size {
            // Subtract one to avoid getting horizontal scrollbar from tui-scrollview
            width: area.width.saturating_sub(1),

            height: area.height * 2,
        };

        let Ok(mut st) = state.lock() else {
            return;
        };

        let mut available_area = area;
        available_area.height = content_size.height;

        let mut chat_log = st.chat_log.clone();
        let iterator = chat_log.iter_mut();
        let mut scrollview = ScrollView::new(content_size);

        for chat_log_item in iterator {
            if available_area.height == 0 {
                break;
            }

            match chat_log_item {
                ChatLogItem::Event(chat_event) => {
                    chat_event.render(available_area, scrollview.buf_mut());
                    available_area = chat_event.area;
                }

                ChatLogItem::Message(chat_item) => {
                    chat_item.render(available_area, scrollview.buf_mut());
                    available_area = chat_item.area;
                }

                // TODO: Implement messages with effects
                ChatLogItem::MessageWithEffect(_chat_item_with_effect) => {
                    info!("Rendering chat item with effect");
                }
            }
        }

        if chat_log.is_empty() {
            st.scrollstate.scroll_to_bottom();
        }

        scrollview.render(buf.area, buf, &mut st.scrollstate);
    }
}
