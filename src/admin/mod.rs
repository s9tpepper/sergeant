use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId},
    prelude::{Document, ToSourceKind, TuiBackend},
    runtime::{Runtime, RuntimeBuilder},
};
use components::{
    actions_view::ActionsView,
    announcements::AnnouncementsView,
    app::{App, AppState},
    commands_view::CommandsView,
    edit_input::EditInput,
    floating::{
        add_action::AddAction, add_announcement::AddAnnouncement, add_command::AddCommand, add_reward::AddReward,
        confirm::Confirm, edit_action::EditAction, edit_announcement::EditAnnouncement, edit_command::EditCommand,
        edit_reward::EditReward, error::ErrorWindow,
    },
    info_view::InfoView,
    inputs::{InputState, TextInput},
    rewards_view::RewardsView,
};
use templates::{APP_TEMPLATE, TEXT_INPUT_TEMPLATE};

mod components;
mod messages;
mod templates;

pub fn admin() {
    let _ = Admin::new().run();
}

struct Admin {
    component_ids: Option<HashMap<String, ComponentId<String>>>,
}

impl Admin {
    pub fn new() -> Self {
        Admin {
            component_ids: Some(HashMap::new()),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let doc = Document::new("@app [id: \"app\"]");

        let tui = TuiBackend::builder()
            // .enable_alt_screen()
            .enable_raw_mode()
            .hide_cursor()
            .finish();

        if let Err(ref error) = tui {
            println!("[ERROR] Could not start terminal interface");
            println!("{error:?}");
        }

        let backend = tui.unwrap();
        let mut runtime_builder = Runtime::builder(doc, backend);
        self.register_components(&mut runtime_builder);

        let runtime = runtime_builder.finish();
        if let Ok(mut runtime) = runtime {
            runtime.run();
        } else if let Err(error) = runtime {
            println!("{:?}", error);
        }

        Ok(())
    }

    fn get_edit_inputs(&self) -> Vec<(String, String)> {
        vec![
            ("add_cmd_name_input".to_string(), "add_command_window".to_string()),
            ("add_cmd_output_input".to_string(), "add_command_window".to_string()),
            ("cmd_name_input".to_string(), "edit_command_window".to_string()),
            ("cmd_output_input".to_string(), "edit_command_window".to_string()),
            (
                "announce_name_input".to_string(),
                "edit_announcement_window".to_string(),
            ),
            (
                "announce_message_input".to_string(),
                "edit_announcement_window".to_string(),
            ),
            (
                "announce_timing_input".to_string(),
                "edit_announcement_window".to_string(),
            ),
            ("reward_name_input".to_string(), "edit_reward_window".to_string()),
            (
                "reward_shell_command_input".to_string(),
                "edit_reward_window".to_string(),
            ),
            ("add_action_name_input".to_string(), "add_action_window".to_string()),
            ("add_action_command_input".to_string(), "add_action_window".to_string()),
            ("add_action_option_input".to_string(), "add_action_window".to_string()),
            ("edit_action_name_input".to_string(), "edit_action_window".to_string()),
            (
                "edit_action_shell_command_input".to_string(),
                "edit_action_window".to_string(),
            ),
            ("edit_action_option_input".to_string(), "edit_action_window".to_string()),
            ("add_reward_name_input".to_string(), "add_reward_window".to_string()),
            ("add_reward_command_input".to_string(), "add_reward_window".to_string()),
        ]
    }

    // fn register_editable_inputs(&mut self, builder: &mut RuntimeBuilder<TuiBackend, ()>) {
    //     let inputs = self.get_edit_inputs();
    //     let component_ids = self.component_ids.as_mut().unwrap();
    //
    //     for (ident, return_focus_id) in inputs {
    //         EditInput::register(ident, return_focus_id.to_string(), builder, component_ids);
    //     }
    // }

    fn register_components(&mut self, builder: &mut RuntimeBuilder<TuiBackend, ()>) {
        let inputs = self.get_edit_inputs();
        if self.component_ids.is_none() {
            panic!("Component IDs map is broken");
        }

        let _ = builder.register_prototype("text_input", TEXT_INPUT_TEMPLATE, || TextInput, InputState::new);

        let component_ids = self.component_ids.as_mut().unwrap();

        for (ident, return_focus_id) in inputs {
            EditInput::register(ident, return_focus_id.to_string(), builder, component_ids);
        }

        AddCommand::register(builder, component_ids);
        InfoView::register(builder, component_ids);
        Confirm::register(builder, component_ids);
        CommandsView::register(builder, component_ids);
        ErrorWindow::register(builder, component_ids);
        EditCommand::register(builder, component_ids);
        AnnouncementsView::register(builder, component_ids);
        AddAnnouncement::register(builder, component_ids);
        EditAnnouncement::register(builder, component_ids);
        RewardsView::register(builder, component_ids);
        AddReward::register(builder, component_ids);
        EditReward::register(builder, component_ids);
        ActionsView::register(builder, component_ids);
        AddAction::register(builder, component_ids);
        EditAction::register(builder, component_ids);

        let component_ids = self.component_ids.take().unwrap();
        let app = App { component_ids };
        let _ = builder.register_component("app", APP_TEMPLATE, app, AppState::new());
    }
}

pub trait AppComponent {
    fn register_component<C: Component<Message = String> + 'static>(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        ident: impl Into<String>,
        template: impl ToSourceKind,
        component: C,
        state: C::State,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        let component_ident: String = ident.into();
        if let Ok(id) = builder.register_component(component_ident.clone(), template, component, state) {
            component_ids.insert(component_ident, id);
        }
    }
}
