use std::cmp::{max, min};

use anathema::state::{List, State, Value};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct ListView;

#[derive(State)]
pub struct Item {
    pub name: Value<String>,
    pub details: Value<String>,
    pub index: Value<usize>,
    pub color: Value<String>,
}

#[derive(State)]
pub struct ListViewState {
    pub cursor: Value<u8>,
    pub current_first_index: Value<u8>,
    pub current_last_index: Value<u8>,
    pub visible_items: Value<u8>,
    pub window_list: Value<List<Item>>,
    pub item_count: Value<u8>,
    pub selected_item: Value<String>,
    pub default_color: Value<String>,
    pub selected_color: Value<String>,
    pub min_width: Value<usize>,
    pub max_width: Value<Option<usize>>,
    pub title_background: Value<String>,
    pub title_foreground: Value<String>,
    pub title_heading: Value<String>,
    pub title_subheading: Value<String>,
    pub footer_background: Value<String>,
    pub footer_foreground: Value<String>,
    pub footer_heading: Value<String>,
    pub footer_subheading: Value<String>,
    pub item_row_fill: Value<String>,
}

impl ListViewState {
    pub fn new() -> Self {
        ListViewState {
            cursor: 0.into(),
            item_count: 0.into(),
            current_first_index: 0.into(),
            current_last_index: 4.into(),
            visible_items: 5.into(),
            window_list: List::empty(),
            selected_item: "".to_string().into(),
            default_color: "".to_string().into(),
            selected_color: "".to_string().into(),
            min_width: 0.into(),
            max_width: None.into(),
            title_background: "".to_string().into(),
            title_foreground: "".to_string().into(),
            title_heading: "".to_string().into(),
            title_subheading: "".to_string().into(),
            footer_background: "".to_string().into(),
            footer_foreground: "".to_string().into(),
            footer_heading: "".to_string().into(),
            footer_subheading: "".to_string().into(),
            item_row_fill: "‧".to_string().into(),
        }
    }
}

pub trait ListComponent<'a, O>
where
    O: Clone + Into<Item> + Deserialize<'a> + Serialize,
{
    fn get_list(&self) -> Vec<O>;

    fn move_cursor_down(&self, state: &mut ListViewState) {
        let last_complete_list_index = self.get_list().len().saturating_sub(1);
        let new_cursor = min(*state.cursor.to_ref() + 1, last_complete_list_index as u8);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor > last_index {
            last_index = new_cursor;
            first_index = new_cursor - (*state.visible_items.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_item_list(first_index.into(), last_index.into(), new_cursor.into(), state);
    }

    fn move_cursor_up(&self, state: &mut ListViewState) {
        let new_cursor = max(state.cursor.to_ref().saturating_sub(1), 0);
        state.cursor.set(new_cursor);

        let mut first_index = *state.current_first_index.to_ref();
        let mut last_index = *state.current_last_index.to_ref();

        if new_cursor < first_index {
            first_index = new_cursor;
            last_index = new_cursor + (*state.visible_items.to_ref() - 1);

            state.current_first_index.set(first_index);
            state.current_last_index.set(last_index);
        }

        self.update_item_list(first_index.into(), last_index.into(), new_cursor.into(), state);
    }

    fn update_item_list(
        &self,
        first_index: usize,
        last_index: usize,
        selected_index: usize,
        state: &mut ListViewState,
    ) {
        if self.get_list().is_empty() {
            return;
        }

        let range_end = min(last_index, self.get_list().len().saturating_sub(1));
        let original_list = &self.get_list()[first_index..=range_end];
        let mut new_item_list: Vec<Item> = vec![];
        original_list.iter().for_each(|original_type| {
            new_item_list.push(original_type.clone().into());
        });

        loop {
            if state.window_list.len() > 0 {
                state.window_list.pop_front();
            } else {
                break;
            }
        }

        new_item_list.into_iter().enumerate().for_each(|(index, mut item)| {
            let visible_index = selected_index.saturating_sub(first_index);
            if index == visible_index {
                item.color = state.selected_color.to_ref().clone().into();
            } else {
                item.color = state.default_color.to_ref().clone().into();
            }

            state.window_list.push(item);
        });
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut ListViewState,
        _: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, ListViewState>,
    ) {
        match event.code {
            anathema::component::KeyCode::Char(char) => match char {
                'j' => self.move_cursor_down(state),
                'k' => self.move_cursor_up(state),
                'd' => self.send_delete_selection(state, context),
                _ => {}
            },

            anathema::component::KeyCode::Up => self.move_cursor_up(state),
            anathema::component::KeyCode::Down => self.move_cursor_down(state),
            anathema::component::KeyCode::Esc => self.send_cancel_view(context),
            anathema::component::KeyCode::Enter => self.send_item_selection(state, context),

            _ => {}
        }
    }

    fn send_cancel_view(&self, mut context: anathema::prelude::Context<'_, ListViewState>) {
        // NOTE: This sends cursor to satisfy publish() but is not used
        context.publish("cancel_item_window", |state| &state.cursor)
    }

    fn send_delete_selection(
        &self,
        state: &mut ListViewState,
        mut context: anathema::prelude::Context<'_, ListViewState>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let list = self.get_list();
        let original_item = list.get(selected_index);

        match original_item {
            Some(item) => match serde_json::to_string(item) {
                Ok(item_json) => {
                    state.selected_item.set(item_json);
                    context.publish("delete_item_selection", |state| &state.selected_item)
                }

                Err(_) => context.publish("cancel_item_window", |state| &state.cursor),
            },
            None => context.publish("cancel_item_window", |state| &state.cursor),
        }
    }

    fn send_item_selection(
        &self,
        state: &mut ListViewState,
        mut context: anathema::prelude::Context<'_, ListViewState>,
    ) {
        let selected_index = *state.cursor.to_ref() as usize;
        let list = self.get_list();
        let original_item = list.get(selected_index);

        match original_item {
            Some(item) => match serde_json::to_string(item) {
                Ok(item_json) => {
                    state.selected_item.set(item_json);
                    context.publish("item_selection", |state| &state.selected_item);
                }
                Err(_) => context.publish("cancel_item_window", |state| &state.cursor),
            },
            None => context.publish("cancel_item_window", |state| &state.cursor),
        }
    }

    fn on_focus(
        &mut self,
        state: &mut ListViewState,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, ListViewState>,
    ) {
        self.load(state);

        // Reset navigation state
        state.cursor.set(0);
        state.current_first_index.set(0);
        state
            .current_last_index
            .set(state.visible_items.to_ref().saturating_sub(1));

        let first_index: usize = *state.current_first_index.to_ref() as usize;
        let last_index: usize = *state.current_last_index.to_ref() as usize;
        let selected_index = 0;

        self.update_item_list(first_index, last_index, selected_index, state)
    }

    fn load(&mut self, state: &mut ListViewState);
}
