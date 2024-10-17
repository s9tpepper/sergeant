use anathema::{component::Component, state::State};

#[derive(Default)]
pub struct App;

#[derive(Default, State)]
pub struct AppState {}

impl AppState {
    pub fn new() -> Self {
        AppState {}
    }
}

impl Component for App {
    type State = AppState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        false
    }
}
