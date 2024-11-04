use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId},
    prelude::{ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::State,
};

use crate::admin::{templates::ERROR_TEMPLATE, AppComponent};

#[derive(Default)]
pub struct ErrorWindow;

#[derive(Default, State)]
pub struct ErrorState {}

impl AppComponent for ErrorWindow {}
impl ErrorWindow {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_command::AddCommand as AppComponent>::register_component(
            builder,
            "error_window",
            ERROR_TEMPLATE.to_template(),
            ErrorWindow,
            ErrorState::new(),
            component_ids,
        )
    }
}

impl ErrorState {
    pub fn new() -> Self {
        ErrorState {}
    }
}

impl Component for ErrorWindow {
    type State = ErrorState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        println!("Error Window got focused");
    }
}
