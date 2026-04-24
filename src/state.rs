//! Top-level application states.
//! Registered via `App::init_state::<GameState>()` in `main.rs`.

use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
#[expect(
    dead_code,
    reason = "non-default variants become live as state transitions land in later stories"
)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Arena,
    Caravan,
    PostRun,
    PhotoMode,
    Paused,
}

pub fn log_loading_entered() {
    info!("entered Loading");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_loading() {
        assert_eq!(GameState::default(), GameState::Loading);
    }
}
