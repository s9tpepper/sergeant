use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId},
    prelude::{Document, ToSourceKind, TuiBackend},
    runtime::{Runtime, RuntimeBuilder},
};
use components::{
    announcements::AnnouncementsView,
    app::{App, AppState},
    cmd_name_input::CmdNameInput,
    cmd_output_input::CmdOutputInput,
    commands_view::CommandsView,
    floating::{
        add_announcement::AddAnnouncement, add_command::AddCommand, confirm::Confirm, edit_command::EditCommand,
        error::ErrorWindow,
    },
    info_view::InfoView,
    inputs::{InputState, TextInput},
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

    fn register_components(&mut self, builder: &mut RuntimeBuilder<TuiBackend, ()>) {
        if self.component_ids.is_none() {
            panic!("Component IDs map is broken");
        }

        let _ = builder.register_prototype("text_input", TEXT_INPUT_TEMPLATE, || TextInput, InputState::new);

        let component_ids = self.component_ids.as_mut().unwrap();

        AddCommand::register(builder, component_ids);
        InfoView::register(builder, component_ids);
        Confirm::register(builder, component_ids);
        CommandsView::register(builder, component_ids);
        ErrorWindow::register(builder, component_ids);
        CmdNameInput::register(builder, component_ids);
        CmdOutputInput::register(builder, component_ids);
        EditCommand::register(builder, component_ids);
        AnnouncementsView::register(builder, component_ids);
        AddAnnouncement::register(builder, component_ids);

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