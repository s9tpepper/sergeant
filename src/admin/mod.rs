use std::collections::HashMap;

use anathema::{
    component::ComponentId,
    prelude::{Document, TuiBackend},
    runtime::{Error, Runtime, RuntimeBuilder},
    state::List,
};
use components::{
    app::{App, AppState},
    commands_view::CommandsView,
    floating::{
        add_command::{AddCommand, AddCommandState},
        confirm::{Confirm, ConfirmState},
        error::{ErrorState, ErrorWindow},
    },
    info_view::{InfoView, InfoViewState},
    inputs::{InputState, TextInput},
    list_view::ListViewState,
};
use templates::{
    ADD_COMMAND_TEMPLATE, APP_TEMPLATE, CONFIRM_TEMPLATE, ERROR_TEMPLATE, INFO_VIEW_TEMPLATE, LIST_VIEW_TEMPLATE,
    TEXT_INPUT_TEMPLATE,
};

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

    fn register_component_id(&mut self, name: &str, component_id: Result<ComponentId<String>, Error>) {
        if self.component_ids.is_none() {
            return;
        }

        let component_ids = self.component_ids.as_mut().unwrap();
        if let Ok(id) = component_id {
            component_ids.insert(name.to_string(), id);
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

    fn register_components(&mut self, builder: &mut RuntimeBuilder<TuiBackend, ()>) {
        if self.component_ids.is_none() {
            panic!("Component IDs map is broken");
        }

        let _ = builder.register_prototype("text_input", TEXT_INPUT_TEMPLATE, || TextInput, InputState::new);

        let add_command_id = builder.register_component(
            "add_command_window",
            ADD_COMMAND_TEMPLATE,
            AddCommand,
            AddCommandState::new(),
        );
        self.register_component_id("add_command_window", add_command_id);

        let info_view_id = builder.register_component("info_view", INFO_VIEW_TEMPLATE, InfoView, InfoViewState::new());
        self.register_component_id("info_view", info_view_id);

        let confirm_window_id =
            builder.register_component("confirm_window", CONFIRM_TEMPLATE, Confirm::new(), ConfirmState::new());
        self.register_component_id("confirm_window", confirm_window_id);

        let commands_view_id = builder.register_component(
            "commands_view",
            LIST_VIEW_TEMPLATE,
            CommandsView::new(),
            ListViewState {
                item_row_fill: "‧".to_string().into(),
                current_last_index: 4.into(),
                visible_items: 5.into(),
                default_color: "#313131".to_string().into(),
                selected_color: "#ffffff".to_string().into(),
                min_width: 10.into(),
                title_background: "yellow".to_string().into(),
                title_foreground: "#131313".to_string().into(),
                title_heading: "Commands".to_string().into(),
                window_list: List::empty(),
                ..Default::default()
            },
        );
        self.register_component_id("commands_view", commands_view_id);

        let error_window_id =
            builder.register_component("error_window", ERROR_TEMPLATE, ErrorWindow, ErrorState::new());
        self.register_component_id("error_window", error_window_id);

        let component_ids = self.component_ids.take().unwrap();
        let app = App { component_ids };
        let _ = builder.register_component("app", APP_TEMPLATE, app, AppState::new());
    }
}
