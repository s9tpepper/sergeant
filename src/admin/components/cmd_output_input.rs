use anathema::{component::Component, prelude::Context, widgets::Elements};

use super::inputs::{InputReceiver, InputState};

#[derive(Default)]
pub struct CmdOutputInput;

impl InputReceiver for CmdOutputInput {}

impl Component for CmdOutputInput {
    type State = InputState;
    type Message = String;

    fn on_blur(&mut self, state: &mut Self::State, elements: Elements<'_, '_>, mut context: Context<'_, Self::State>) {
        context.set_focus("id", "edit_command_window");

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
