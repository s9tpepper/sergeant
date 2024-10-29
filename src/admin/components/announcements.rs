use std::{collections::HashMap, ops::Div};

use anathema::{
    component::{Component, ComponentId, KeyCode::Char},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::{List, State, Value},
};
use serde::{Deserialize, Serialize};

use crate::{
    admin::{
        messages::{ComponentMessages, DeleteAnnouncementConfirmMessage, DeleteAnnouncementConfirmationDetails},
        templates::ANNOUNCEMENTS_LIST_VIEW_TEMPLATE,
        AppComponent,
    },
    commands::remove_chat_command,
    twitch::announcements::get_announcements,
};

use super::{
    app::{AppMessageHandler, FloatingWindow},
    list_view::{Item, ListComponent, ListViewState},
    MessageSender,
};

#[derive(Default)]
pub struct AnnouncementsView {
    announcements: Option<Vec<Announce>>,
}

impl AppComponent for AnnouncementsView {}

impl AppMessageHandler for AnnouncementsView {
    fn handle_message<F>(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::app::AppState,
        mut context: Context<'_, super::app::AppState>,
        component_ids: &HashMap<String, ComponentId<String>>,
        _fun: F,
    ) where
        F: Fn(&mut super::app::AppState, Context<'_, super::app::AppState>),
    {
        let event: String = ident.into();

        match event.as_str() {
            "announcements__add" => {
                state.floating_window.set(super::app::FloatingWindow::AddAnnouncement);
                context.set_focus("id", "add_announcement_window");
            }

            "announcements__close" => {
                state.main_display.set(super::app::MainDisplay::Info);
                context.set_focus("id", "app");
            }

            "announcements__edit_selection" => {
                if let Ok(item) = serde_json::from_str::<Announce>(&value.to_string()) {
                    state.floating_window.set(FloatingWindow::EditAnnouncement);
                    context.set_focus("id", "edit_announcement_window");

                    if let Some(id) = component_ids.get("announce_name_input") {
                        let _ = context.emitter.emit(*id, item.name);
                    }

                    if let Some(id) = component_ids.get("announce_message_input") {
                        let _ = context.emitter.emit(*id, item.message);
                    }

                    if let Some(id) = component_ids.get("announce_timing_input") {
                        let _ = context.emitter.emit(*id, item.timing.to_string());
                    }
                }
            }

            "announcements__delete_selection" => {
                if let Ok(item) = serde_json::from_str::<Announce>(&value.to_string()) {
                    if let Some(id) = component_ids.get("confirm_window") {
                        state.floating_window.set(FloatingWindow::Confirm);
                        context.set_focus("id", "confirm_window");

                        let message = format!("Are you sure you want to delete: {}", item.name);
                        let confirmation_details = DeleteAnnouncementConfirmationDetails {
                            title: "Delete Announcement",
                            waiting: "announcements_view",
                            message: &message,
                            item,
                        };

                        let _ = MessageSender::send_message(
                            *id,
                            ComponentMessages::DeleteAnnoucementConfirmMessage(DeleteAnnouncementConfirmMessage {
                                payload: confirmation_details,
                            }),
                            context.emitter.clone(),
                        );
                    }
                }
            }
            "announcements__show_delete_error" => {}

            _ => {}
        }
    }
}

impl AnnouncementsView {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::announcements::AnnouncementsView as AppComponent>::register_component(
            builder,
            "announcements_view",
            ANNOUNCEMENTS_LIST_VIEW_TEMPLATE,
            AnnouncementsView::new(),
            ListViewState {
                item_row_fill: "‧".to_string().into(),
                current_last_index: 4.into(),
                visible_items: 5.into(),
                default_color: "#313131".to_string().into(),
                selected_color: "#ffffff".to_string().into(),
                min_width: 10.into(),
                title_background: "yellow".to_string().into(),
                title_foreground: "#131313".to_string().into(),
                title_heading: "Announcements".to_string().into(),
                window_list: List::empty(),
                ..Default::default()
            },
            component_ids,
        )
    }
}

impl AnnouncementsView {
    pub fn new() -> Self {
        AnnouncementsView { announcements: None }
    }
}

impl Component for AnnouncementsView {
    type State = ListViewState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        if let Ok(msg) = serde_json::from_str::<ComponentMessages>(&message.to_string()) {
            match msg {
                ComponentMessages::AnnouncementsViewReload(_) => self.load(state),

                // TODO: Update this delete confirm when delete is implemented for announcements
                ComponentMessages::DeleteAnnoucementConfirmMessage(delete_confirmed) => {
                    match remove_chat_command(&delete_confirmed.payload.item.name) {
                        Ok(_) => {
                            self.load(state);
                            self.refresh(state);
                        }
                        Err(_) => context.publish("show_delete_announcement_error", |state| &state.cursor),
                    }
                }

                _ => {}
            }
        }
    }

    fn resize(
        &mut self,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let size = context.viewport.size();
        if size.height == 0 {
            return;
        }

        let visible_items: u8 = (size.height.saturating_sub(5)) as u8;
        state.visible_items.set(visible_items);
        state.current_last_index.set(visible_items.saturating_sub(1));
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        elements: anathema::widgets::Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let size = context.viewport.size();
        if size.height == 0 {
            return;
        }

        let visible_items: u8 = (size.height.saturating_sub(5)) as u8;
        state.visible_items.set(visible_items);
        state.current_last_index.set(visible_items.saturating_sub(1));

        ListComponent::on_focus(self, state, elements, context);
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        elements: anathema::widgets::Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        match event.code {
            Char(char) => match char {
                'a' => context.publish("add_announcement", |state| &state.cursor),
                'e' => self.send_item_selection(state, context),
                'd' => self.send_delete_selection(state, context),
                'b' => self.send_cancel_view(context),

                _ => ListComponent::on_key(self, event, state, elements, context),
            },

            _ => ListComponent::on_key(self, event, state, elements, context),
        }
    }
}

#[derive(State)]
pub struct AnnouncementValues {
    pub name: Value<String>,
    pub message: Value<String>,
    pub timing: Value<String>,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Announce {
    pub name: String,
    pub message: String,
    pub timing: usize,
    pub index: usize,
}

impl From<AnnouncementValues> for Announce {
    fn from(value: AnnouncementValues) -> Self {
        Announce {
            name: value.name.to_ref().to_string(),
            message: value.message.to_ref().to_string(),
            timing: value.timing.to_ref().parse::<usize>().unwrap_or(0),
            index: 0,
        }
    }
}

impl From<Announce> for Item {
    fn from(value: Announce) -> Self {
        Item {
            name: format!("{} - every {} mins", value.name, value.timing).into(),
            details: value.message.into(),
            index: value.index.into(),
            color: "#333333".to_string().into(),
        }
    }
}

impl ListComponent<'_, Announce> for AnnouncementsView {
    fn get_list(&self) -> Vec<Announce> {
        match &self.announcements {
            Some(list) => list.to_vec(),
            None => vec![],
        }
    }

    fn load(&mut self, _state: &mut super::list_view::ListViewState) {
        match get_announcements() {
            Ok(announcements) => {
                let list: Vec<Announce> = announcements
                    .iter()
                    .enumerate()
                    .map(|(index, announcement)| Announce {
                        name: announcement.name.clone(),
                        message: announcement.message.clone(),
                        timing: announcement.timing.as_secs().div(60) as usize,
                        index,
                    })
                    .collect();

                self.announcements = Some(list);
            }
            Err(_) => todo!(),
        }
    }
}
