use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::CommonVal,
    widgets::Elements,
};

use crate::admin::{templates::TEXT_INPUT_TEMPLATE, AppComponent};

use super::inputs::{InputReceiver, InputState};

#[derive(Default)]
pub struct EditInput {
    return_focus_id: String,
}

impl AppComponent for EditInput {}
impl EditInput {
    pub fn register(
        ident: impl Into<String>,
        return_focus_id: String,
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_command::AddCommand as AppComponent>::register_component(
            builder,
            ident,
            TEXT_INPUT_TEMPLATE.to_template(),
            EditInput { return_focus_id },
            InputState::new(),
            component_ids,
        )
    }
}

impl InputReceiver for EditInput {}

impl Component for EditInput {
    type State = InputState;
    type Message = String;

    fn on_blur(&mut self, state: &mut Self::State, elements: Elements<'_, '_>, mut context: Context<'_, Self::State>) {
        if !*state.focused.to_ref() {
            // NOTE: How terrible is this?
            let id = Box::new(self.return_focus_id.clone());
            context.set_focus("id", CommonVal::Str(id.leak()));
        }

        self._on_blur(state, elements, context);
    }

    fn on_focus(&mut self, state: &mut Self::State, elements: Elements<'_, '_>, context: Context<'_, Self::State>) {
        self._on_focus(state, elements, context);
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        self._on_key(key, state, elements, context);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        elements: Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        self._message(message, state, elements, context);
    }

    fn accept_focus(&self) -> bool {
        true
    }
}
