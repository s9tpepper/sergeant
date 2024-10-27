use std::{collections::HashMap, fs};

use anathema::{
    component::{Component, ComponentId},
    default_widgets::Overflow,
    prelude::Context,
    state::{CommonVal, State, Value},
};

use crate::{
    admin::messages::{ComponentMessages, InfoViewLoad},
    utils::get_data_directory,
};

use super::{
    announcements::AnnouncementsView,
    commands_view::CommandsView,
    floating::{
        add_announcement::AddAnnouncement, add_command::AddCommand, add_reward::AddReward, confirm::Confirm,
        edit_announcement::EditAnnouncement, edit_command::EditCommand, edit_reward::EditReward,
    },
    rewards_view::RewardsView,
    Messenger,
};

#[derive(Default)]
pub struct App {
    pub component_ids: HashMap<String, ComponentId<String>>,
}

impl App {
    fn reset_floating_window(&self, state: &mut AppState, mut context: anathema::prelude::Context<'_, AppState>) {
        match *state.main_display.to_ref() {
            MainDisplay::InfoView => context.set_focus("id", "info_view"),
            MainDisplay::CommandsView => context.set_focus("id", "commands_view"),
            MainDisplay::AnnouncementsView => context.set_focus("id", "announcements_view"),
            MainDisplay::RewardsView => context.set_focus("id", "rewards_view"),

            // TODO: Implement rest when they exist
            // MainDisplay::IrcActionsView => todo!(),
            // MainDisplay::Login => todo!(),
            _ => {}
        }

        state.floating_window.set(FloatingWindow::None);
    }
}

#[derive(Default, State)]
pub struct AppState {
    pub main_display: Value<MainDisplay>,
    pub floating_window: Value<FloatingWindow>,
    pub error_message: Value<String>,
    pub logs: Value<String>,
}

#[derive(Default)]
pub enum FloatingWindow {
    #[default]
    None,
    AddCommand,
    EditCommand,
    AddAnnouncement,
    EditAnnouncement,
    AddReward,
    EditReward,
    AddAction,
    Confirm,
    Error,
}

impl State for FloatingWindow {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        match self {
            FloatingWindow::None => Some(CommonVal::Str("None")),
            FloatingWindow::AddCommand => Some(CommonVal::Str("AddCommand")),
            FloatingWindow::EditCommand => Some(CommonVal::Str("EditCommand")),
            FloatingWindow::AddAnnouncement => Some(CommonVal::Str("AddAnnouncement")),
            FloatingWindow::EditAnnouncement => Some(CommonVal::Str("EditAnnouncement")),
            FloatingWindow::AddReward => Some(CommonVal::Str("AddReward")),
            FloatingWindow::EditReward => Some(CommonVal::Str("EditReward")),
            FloatingWindow::AddAction => Some(CommonVal::Str("AddAction")),
            FloatingWindow::Confirm => Some(CommonVal::Str("Confirm")),
            FloatingWindow::Error => Some(CommonVal::Str("Error")),
        }
    }
}

#[derive(Default)]
pub enum MainDisplay {
    #[default]
    InfoView,
    CommandsView,
    AnnouncementsView,
    RewardsView,
    ActionsView,
    // NOTE: Maybe don't need login
    Login,
    LogsView,
}

impl State for MainDisplay {
    fn to_common(&self) -> Option<anathema::state::CommonVal<'_>> {
        match self {
            MainDisplay::InfoView => Some(CommonVal::Str("InfoView")),
            MainDisplay::CommandsView => Some(CommonVal::Str("CommandsView")),
            MainDisplay::AnnouncementsView => Some(CommonVal::Str("AnnouncementsView")),
            MainDisplay::RewardsView => Some(CommonVal::Str("RewardsView")),
            MainDisplay::ActionsView => Some(CommonVal::Str("ActionsView")),
            MainDisplay::Login => Some(CommonVal::Str("Login")),
            MainDisplay::LogsView => Some(CommonVal::Str("LogsView")),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            main_display: MainDisplay::InfoView.into(),
            floating_window: FloatingWindow::None.into(),
            error_message: String::from("").into(),
            logs: String::from("").into(),
        }
    }
}

impl Messenger for App {}

impl Component for App {
    type State = AppState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match *state.main_display.to_ref() {
            MainDisplay::InfoView => {
                if let Some(id) = self.component_ids.get("info_view") {
                    let _ = self.send_message(
                        *id,
                        ComponentMessages::InfoViewLoad(InfoViewLoad {}),
                        context.emitter.clone(),
                    );
                }
            }
            MainDisplay::CommandsView => {}

            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'c' => {
                    state.main_display.set(MainDisplay::CommandsView);
                    context.set_focus("id", "commands_view");
                }

                'a' => {
                    state.main_display.set(MainDisplay::ActionsView);
                    context.set_focus("id", "actions_view");
                }

                'r' => {
                    state.main_display.set(MainDisplay::RewardsView);
                    context.set_focus("id", "rewards_view");
                }

                'i' => {}
                'l' => {}

                'n' => {
                    state.main_display.set(MainDisplay::AnnouncementsView);
                    context.set_focus("id", "announcements_view");
                }

                'g' => {
                    state.main_display.set(MainDisplay::LogsView);
                    let mut error_log = get_data_directory(Some("error_log")).unwrap();
                    error_log.push("log.txt");
                    match fs::read_to_string(error_log) {
                        Ok(logs) => state.logs.set(logs),
                        Err(_) => state.logs.set(String::from("Logs unavailable.")),
                    }
                }

                'b' => {
                    state.main_display.set(MainDisplay::InfoView);
                    context.set_focus("id", "app");
                }

                'd' => {
                    if key.ctrl {
                        if let MainDisplay::LogsView = *state.main_display.to_ref() {
                            elements
                                .by_attribute("id", "logs_container")
                                .first(|element, _attributes| {
                                    let size = element.size();
                                    if size.height > 0 {
                                        let scroll_by = size.height.saturating_div(2);
                                        let overflow = element.to::<Overflow>();
                                        overflow.scroll_down_by(scroll_by as i32);
                                    }
                                })
                        }
                    }
                }

                'u' => {
                    if key.ctrl {
                        if let MainDisplay::LogsView = *state.main_display.to_ref() {
                            elements
                                .by_attribute("id", "logs_container")
                                .first(|element, _attributes| {
                                    let size = element.size();
                                    if size.height > 0 {
                                        let scroll_by = size.height.saturating_div(2);
                                        let overflow = element.to::<Overflow>();
                                        overflow.scroll_up_by(scroll_by as i32);
                                    }
                                })
                        }
                    }
                }

                _ => {}
            },

            anathema::component::KeyCode::Esc => {}

            _ => {}
        }
    }

    fn receive(
        &mut self,
        ident: &str,
        value: CommonVal<'_>,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if let Some((component_name, _)) = ident.split_once("__") {
            match component_name {
                "rewards" => {
                    RewardsView::handle_message(value, ident, state, context, &self.component_ids, |state, context| {
                        self.reset_floating_window(state, context)
                    })
                }

                "add_reward" => {
                    AddReward::handle_message(value, ident, state, context, &self.component_ids, |state, context| {
                        self.reset_floating_window(state, context)
                    })
                }

                "edit_reward" => {
                    EditReward::handle_message(value, ident, state, context, &self.component_ids, |state, context| {
                        self.reset_floating_window(state, context)
                    })
                }

                "announcements" => AnnouncementsView::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    &self.component_ids,
                    |state, context| self.reset_floating_window(state, context),
                ),

                "add_announcement" => AddAnnouncement::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    &self.component_ids,
                    |state, context| self.reset_floating_window(state, context),
                ),

                "edit_announcement" => EditAnnouncement::handle_message(
                    value,
                    ident,
                    state,
                    context,
                    &self.component_ids,
                    |state, context| self.reset_floating_window(state, context),
                ),

                "confirm" => {
                    Confirm::handle_message(value, ident, state, context, &self.component_ids, |state, context| {
                        self.reset_floating_window(state, context)
                    })
                }

                "commands" => {
                    CommandsView::handle_message(value, ident, state, context, &self.component_ids, |state, context| {
                        self.reset_floating_window(state, context)
                    })
                }

                "add_command" => {
                    AddCommand::handle_message(value, ident, state, context, &self.component_ids, |state, context| {
                        self.reset_floating_window(state, context)
                    });
                }

                "edit_command" => {
                    EditCommand::handle_message(value, ident, state, context, &self.component_ids, |state, context| {
                        self.reset_floating_window(state, context)
                    })
                }

                _ => {}
            }
        }
    }
}

pub trait AppMessageHandler {
    fn handle_message<F>(
        value: CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut AppState,
        context: Context<'_, AppState>,
        component_ids: &HashMap<String, ComponentId<String>>,
        fun: F,
    ) where
        F: Fn(&mut AppState, Context<'_, AppState>);
}
