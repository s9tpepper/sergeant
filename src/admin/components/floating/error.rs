use anathema::{component::Component, state::State};

#[derive(Default)]
pub struct ErrorWindow;

#[derive(Default, State)]
pub struct ErrorState {}

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
        println!("hello");
    }
}
