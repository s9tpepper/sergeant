use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId},
    state::{CommonVal, State, Value},
};

use super::{ComponentMessage, Messenger};

#[derive(Default)]
pub struct App {
    pub component_ids: HashMap<String, ComponentId<String>>,
}

#[derive(Default, State)]
pub struct AppState {
    main_display: Value<MainDisplay>,
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
                        ComponentMessage {
                            r#type: "load_data",
                            payload: "",
                        },
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

            anathema::component::KeyCode::Esc => todo!(),

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

            "add_command" => println!("Add new command dialog"),

            "edit_command_selection" => println!("Selected item: {value}"),

            "delete_command_selection" => println!("Selected item to delete: {value}"),

            _ => {}
        }
    }
}
