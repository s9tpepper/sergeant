use std::collections::HashMap;

use anathema::{
    component::ComponentId,
    prelude::{Document, TuiBackend},
    runtime::{Runtime, RuntimeBuilder},
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
    }
}
