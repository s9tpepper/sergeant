use std::collections::HashMap;

use anathema::{
    component::ComponentId,
    prelude::{Document, TuiBackend},
    runtime::{Runtime, RuntimeBuilder},
};
use components::{
    app::{App, AppState},
    commands_view::{CommandsView, CommandsViewState},
    info_view::{InfoView, InfoViewState},
};
use templates::{APP_TEMPLATE, COMMANDS_VIEW_TEMPLATE, INFO_VIEW_TEMPLATE};

mod components;
mod templates;

pub fn admin() {
    let _ = Admin::new().run();
}

struct Admin {
    component_ids: HashMap<String, ComponentId<String>>,
}

impl Admin {
    pub fn new() -> Self {
        Admin {
            component_ids: HashMap::new(),
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
        let _ = builder.register_component("app", APP_TEMPLATE, App, AppState::new());

        let _ = builder.register_component("info_view", INFO_VIEW_TEMPLATE, InfoView, InfoViewState::new());

        let _ = builder.register_component(
            "commands_view",
            COMMANDS_VIEW_TEMPLATE,
            CommandsView,
            CommandsViewState::new(),
        );
    }
}
