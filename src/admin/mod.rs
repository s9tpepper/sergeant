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
    info_view::{InfoView, InfoViewState},
    list_view::ListViewState,
};
use templates::{APP_TEMPLATE, INFO_VIEW_TEMPLATE, LIST_VIEW_TEMPLATE};

mod components;
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

        let info_view_id = builder.register_component("info_view", INFO_VIEW_TEMPLATE, InfoView, InfoViewState::new());
        self.register_component_id("info_view", info_view_id);

        let commands_view_id = builder.register_component(
            "commands_view",
            LIST_VIEW_TEMPLATE,
            CommandsView::new(),
            ListViewState {
                cursor: 0.into(),
                item_count: 0.into(),
                current_first_index: 0.into(),
                current_last_index: 4.into(),
                visible_items: 5.into(),
                window_list: List::empty(),
                selected_item: "".to_string().into(),
                default_color: "#313131".to_string().into(),
                selected_color: "#ffffff".to_string().into(),
                min_width: 10.into(),
                // max_width: usize::MAX.into(),
                max_width: None.into(),
                title_background: "yellow".to_string().into(),
                title_foreground: "#131313".to_string().into(),
                title_heading: "Commands".to_string().into(),
                title_subheading: "".to_string().into(),
                footer_background: "".to_string().into(),
                footer_foreground: "".to_string().into(),
                footer_heading: "".to_string().into(),
                footer_subheading: "".to_string().into(),
                item_row_fill: "‧".to_string().into(),
            },
        );
        self.register_component_id("commands_view", commands_view_id);

        let component_ids = self.component_ids.take().unwrap();
        let app = App { component_ids };
        let _ = builder.register_component("app", APP_TEMPLATE, app, AppState::new());
    }
}
