use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId, KeyCode},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{Number, State, Value},
};

use crate::{
    admin::{
        components::{app::AppMessageHandler, MessageSender},
        messages::{AnnouncementsViewReload, ComponentMessages},
        templates::EDIT_ANNOUNCEMENT_TEMPLATE,
        AppComponent,
    },
    commands::add_chat_command,
};

use super::add_announcement::Announcement;

#[derive(Default)]
pub struct EditAnnouncement;

impl AppComponent for EditAnnouncement {}
impl EditAnnouncement {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_command::AddCommand as AppComponent>::register_component(
            builder,
            "edit_announcement_window",
            EDIT_ANNOUNCEMENT_TEMPLATE,
            EditAnnouncement,
            EditAnnouncementState::new(),
            component_ids,
        )
    }
}

#[derive(Default, State)]
pub struct EditAnnouncementState {
    announcement: Value<Announcement>,
}

impl EditAnnouncementState {
    pub fn new() -> Self {
        EditAnnouncementState {
            announcement: Announcement {
                name: String::from("").into(),
                message: String::from("").into(),
                timing: 0.into(),
                common: String::from("::::").into(),
            }
            .into(),
        }
    }
}

impl AppMessageHandler for EditAnnouncement {
    fn handle_message<F>(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut crate::admin::components::app::AppState,
        context: anathema::prelude::Context<'_, crate::admin::components::app::AppState>,
        component_ids: &HashMap<String, ComponentId<String>>,
        fun: F,
    ) where
        F: Fn(
            &mut crate::admin::components::app::AppState,
            anathema::prelude::Context<'_, crate::admin::components::app::AppState>,
        ),
    {
        let event: String = ident.into();
        match event.as_str() {
            "edit_announcement__cancel" => {
                if let Some(id) = component_ids.get("announce_name_input") {
                    let _ = context.emitter.emit(*id, String::from(""));
                }

                if let Some(id) = component_ids.get("announce_message_input") {
                    let _ = context.emitter.emit(*id, String::from(""));
                }

                if let Some(id) = component_ids.get("announce_timing_input") {
                    let _ = context.emitter.emit(*id, String::from(""));
                }

                fun(state, context);
            }

            "edit_announcement__submit" => {
                let announcement: Announcement = value.into();

                let default_timing = Number::Usize(5);
                let timing = announcement.timing.to_number().unwrap_or(default_timing).as_uint();

                match add_chat_command(
                    &announcement.name.to_ref(),
                    &announcement.message.to_ref(),
                    Some(timing),
                ) {
                    Ok(_) => {
                        if let Some(id) = component_ids.get("announcements_view") {
                            let _ = MessageSender::send_message(
                                *id,
                                ComponentMessages::AnnouncementsViewReload(AnnouncementsViewReload {}),
                                context.emitter.clone(),
                            );
                        }
                    }

                    Err(_) => {
                        // TODO: bring up a message window with an error message
                    }
                };

                fun(state, context);
            }

            _ => {}
        }
    }
}

impl Component for EditAnnouncement {
    type State = EditAnnouncementState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match ident {
            "edit_announcement__name_update" => {
                state.announcement.to_mut().name.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    value,
                    *state.announcement.to_ref().message.to_ref(),
                    *state.announcement.to_ref().timing.to_ref()
                );
                state.announcement.to_mut().common.set(common);
            }

            "edit_announcement__message_update" => {
                state.announcement.to_mut().message.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    *state.announcement.to_ref().name.to_ref(),
                    value,
                    *state.announcement.to_ref().timing.to_ref()
                );
                state.announcement.to_mut().common.set(common);
            }

            "edit_announcement__timing_update" => {
                let default_timing = Number::Usize(5);
                let timing = value.to_number().unwrap_or(default_timing).as_uint();
                state.announcement.to_mut().timing.set(timing);

                let common = format!(
                    "{}::::{}::::{}",
                    *state.announcement.to_ref().name.to_ref(),
                    *state.announcement.to_ref().message.to_ref(),
                    value
                );
                state.announcement.to_mut().common.set(common);
            }

            "edit_announcement__name_focus_change" => {
                context.set_focus("id", "edit_announcement_window");
            }

            "edit_announcement__message_focus_change" => {
                context.set_focus("id", "edit_announcement_window");
            }

            "edit_announcement__timing_focus_change" => {
                context.set_focus("id", "edit_announcement_window");
            }

            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Char(char) => match char {
                's' => {
                    context.publish("edit_announcement__submit", |state| &state.announcement);
                }

                'c' => context.publish("edit_announcement__cancel", |state| &state.announcement),

                'n' => context.set_focus("id", "edit_announcement_name_input"),
                'm' => context.set_focus("id", "edit_announcement_message_input"),
                't' => context.set_focus("id", "edit_announcement_timing_input"),

                _ => {}
            },

            KeyCode::Esc => context.publish("edit_announcement__cancel", |state| &state.announcement),

            _ => {}
        }
    }
}
