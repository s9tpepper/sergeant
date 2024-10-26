use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId, KeyCode},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{CommonVal, Number, State, Value},
};

use crate::{
    admin::{
        components::{app::AppMessageHandler, MessageSender},
        messages::{AnnouncementsViewReload, ComponentMessages},
        templates::ADD_ANNOUNCEMENT_TEMPLATE,
        AppComponent,
    },
    commands::add_chat_command,
};

#[derive(Default)]
pub struct AddAnnouncement;

impl AddAnnouncement {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_announcement::AddAnnouncement as AppComponent>::register_component(
            builder,
            "add_announcement_window",
            ADD_ANNOUNCEMENT_TEMPLATE,
            AddAnnouncement,
            AddAnnouncementState::new(),
            component_ids,
        )
    }
}

impl AppComponent for AddAnnouncement {}

impl AppMessageHandler for AddAnnouncement {
    fn handle_message<F>(
        value: CommonVal<'_>,
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
            "add_announcement__cancel" => {
                fun(state, context);
            }

            "add_announcement__submit" => {
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

#[derive(Default, State)]
pub struct AddAnnouncementState {
    announcement: Value<Announcement>,
}

#[derive(Default, Debug)]
pub struct Announcement {
    pub name: Value<String>,
    pub message: Value<String>,
    pub timing: Value<usize>,
    pub common: Value<String>,
}

impl State for Announcement {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        let str = self.common.to_ref().to_string().clone().into_boxed_str();

        Some(CommonVal::Str(Box::leak(str)))
    }
}

impl From<CommonVal<'_>> for Announcement {
    fn from(value: CommonVal) -> Self {
        let serialized_announcement = value.to_string();
        let fields: Vec<&str> = serialized_announcement.split("::::").collect();

        match fields.as_slice() {
            [name, message, timing] => Announcement {
                name: String::from(*name).into(),
                message: String::from(*message).into(),
                timing: timing.parse::<usize>().unwrap_or(5).into(),
                common: serialized_announcement.into(),
            },

            _ => Announcement {
                name: String::from("").into(),
                message: String::from("").into(),
                timing: 0.into(),
                common: String::from("::::").into(),
            },
        }
    }
}

impl AddAnnouncementState {
    pub fn new() -> Self {
        AddAnnouncementState {
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

impl Component for AddAnnouncement {
    type State = AddAnnouncementState;
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
            "add_announcement__name_update" => {
                state.announcement.to_mut().name.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    value,
                    *state.announcement.to_ref().message.to_ref(),
                    *state.announcement.to_ref().timing.to_ref()
                );
                state.announcement.to_mut().common.set(common);
            }

            "add_announcement__message_update" => {
                state.announcement.to_mut().message.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    *state.announcement.to_ref().name.to_ref(),
                    value,
                    *state.announcement.to_ref().timing.to_ref()
                );
                state.announcement.to_mut().common.set(common);
            }

            "add_announcement__timing_update" => {
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

            "add_announcement__name_focus_change" => {
                context.set_focus("id", "add_announcement_window");
            }

            "add_announcement__message_focus_change" => {
                context.set_focus("id", "add_announcement_window");
            }

            "add_announcement__timing_focus_change" => {
                context.set_focus("id", "add_announcement_window");
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
                's' => context.publish("add_announcement__submit", |state| &state.announcement),

                'c' => context.publish("add_announcement__cancel", |state| &state.announcement),

                'n' => context.set_focus("id", "add_announcement_name_input"),

                'a' => context.set_focus("id", "add_announcement_message_input"),

                't' => context.set_focus("id", "add_announcement_timing_input"),

                _ => {}
            },

            KeyCode::Esc => context.publish("add_announcement__cancel", |state| &state.announcement),

            _ => {}
        }
    }
}
