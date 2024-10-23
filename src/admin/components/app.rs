use std::{collections::HashMap, str::FromStr, thread::sleep, time::Duration};

use anathema::{
    component::{Component, ComponentId},
    state::{CommonVal, State, Value},
};

use crate::{
    admin::messages::{
        CommandsViewReload, ComponentMessages, DeleteCommandConfirmMessage, DeleteCommandConfirmationDetails,
        InfoViewLoad,
    },
    commands::add_chat_command,
    twitch::pubsub::send_to_error_log,
};

use super::{commands_view::Cmd, floating::add_command::Command, ComponentMessage, Messenger};

#[derive(Default)]
pub struct App {
    pub component_ids: HashMap<String, ComponentId<String>>,
}

impl App {
    fn reset_floating_window(&self, state: &mut AppState, mut context: anathema::prelude::Context<'_, AppState>) {
        match *state.main_display.to_ref() {
            MainDisplay::InfoView => context.set_focus("id", "info_view"),
            MainDisplay::CommandsView => context.set_focus("id", "commands_view"),

            // TODO: Implement rest when they exist
            // MainDisplay::AnnouncementsView => todo!(),
            // MainDisplay::RewardsView => todo!(),
            // MainDisplay::IrcActionsView => todo!(),
            // MainDisplay::Login => todo!(),
            _ => {}
        }

        state.floating_window.set(FloatingWindow::None);
    }
}

#[derive(Default, State)]
pub struct AppState {
    main_display: Value<MainDisplay>,
    floating_window: Value<FloatingWindow>,
    error_message: Value<String>,
}

#[derive(Default)]
enum FloatingWindow {
    #[default]
    None,
    AddCommand,
    EditCommand,
    AddAnnouncement,
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
            FloatingWindow::Confirm => Some(CommonVal::Str("Confirm")),
            FloatingWindow::Error => Some(CommonVal::Str("Error")),
        }
    }
}

#[derive(Default)]
enum MainDisplay {
    #[default]
    InfoView,
    CommandsView,
    AnnouncementsView,
    RewardsView,
    IrcActionsView,
    // NOTE: Maybe don't need login
    Login,
}

impl State for MainDisplay {
    fn to_common(&self) -> Option<anathema::state::CommonVal<'_>> {
        match self {
            MainDisplay::InfoView => Some(CommonVal::Str("InfoView")),
            MainDisplay::CommandsView => Some(CommonVal::Str("CommandsView")),
            MainDisplay::AnnouncementsView => Some(CommonVal::Str("AnnouncementsView")),
            MainDisplay::RewardsView => Some(CommonVal::Str("RewardsView")),
            MainDisplay::IrcActionsView => Some(CommonVal::Str("IrcActionsView")),
            MainDisplay::Login => Some(CommonVal::Str("Login")),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            main_display: MainDisplay::InfoView.into(),
            floating_window: FloatingWindow::None.into(),
            error_message: String::from("").into(),
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
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'c' => {
                    state.main_display.set(MainDisplay::CommandsView);
                    context.set_focus("id", "commands_view");
                }
                'a' => {}
                'r' => {}
                'i' => {}
                'l' => {}

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
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match ident {
            "close_commands_view" => {
                state.main_display.set(MainDisplay::InfoView);
                context.set_focus("id", "app");
            }

            "add_command" => {
                state.floating_window.set(FloatingWindow::AddCommand);
                context.set_focus("id", "add_command_window");
            }
            "cancel_add_command" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "commands_view");
            }
            "submit_add_command" => {
                state.floating_window.set(FloatingWindow::None);
                context.set_focus("id", "commands_view");

                let command: Command = value.into();

                match add_chat_command(&command.name.to_ref(), &command.output.to_ref(), None) {
                    Ok(_) => {
                        if let Some(id) = self.component_ids.get("commands_view") {
                            let _ = self.send_message(
                                *id,
                                ComponentMessages::CommandsViewReload(CommandsViewReload {}),
                                context.emitter.clone(),
                            );
                        }
                    }

                    Err(_) => {
                        // TODO: bring up a message window with an error message
                    }
                };
            }

            "edit_command_selection" => println!("Selected item: {value}"),

            "delete_command_selection" => {
                if let Ok(item) = serde_json::from_str::<Cmd>(&value.to_string()) {
                    if let Some(id) = self.component_ids.get("confirm_window") {
                        state.floating_window.set(FloatingWindow::Confirm);
                        context.set_focus("id", "confirm_window");

                        let message = format!("Are you sure you want to delete: {}", item.name);
                        let confirmation_details = DeleteCommandConfirmationDetails {
                            title: "Delete Command",
                            waiting: "commands_view",
                            message: &message,
                            item,
                        };

                        let _ = self.send_message(
                            *id,
                            ComponentMessages::DeleteCommandConfirmMessage(DeleteCommandConfirmMessage {
                                payload: confirmation_details,
                            }),
                            context.emitter.clone(),
                        );
                    }
                }
            }

            "cancel_confirmation" => {
                self.reset_floating_window(state, context);
            }

            "confirm_delete_command" => {
                match serde_json::from_str::<ComponentMessages>(&value.to_string()) {
                    Ok(component_messages) => {
                        if let ComponentMessages::DeleteCommandConfirmMessage(delete_msg) = component_messages {
                            if let Some(id) = self.component_ids.get(delete_msg.payload.waiting) {
                                let _ = self.send_message(
                                    *id,
                                    ComponentMessages::DeleteCommandConfirmMessage(delete_msg),
                                    context.emitter.clone(),
                                );
                            }
                        }
                    }

                    Err(error) => send_to_error_log(error.to_string(), format!("Could not deserialize {}", value)),
                }

                self.reset_floating_window(state, context);
            }

            "show_delete_command_error" => {
                state.floating_window.set(FloatingWindow::Error);
                state.error_message.set(String::from("Could not delete command"));
                context.set_focus("id", "error_window");

                if let Some(id) = self.component_ids.get("error_window") {
                    let _ = self.send_message(
                        *id,
                        ComponentMessages::CommandsViewReload(CommandsViewReload {}),
                        context.emitter.clone(),
                    );
                }

                sleep(Duration::from_secs(5));
                self.reset_floating_window(state, context);
            }

            _ => {}
        }
    }
}
