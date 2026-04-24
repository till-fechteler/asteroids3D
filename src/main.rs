//! asteroids3D — app entry point.
//! Registers DefaultPlugins and GameState.

use bevy::prelude::*;

mod state;

use state::{GameState, log_loading_entered};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Loading), log_loading_entered)
        .run()
}
