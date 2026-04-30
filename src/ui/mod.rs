//! UiPlugin — bevy_ui screen-space UI surfaces.
//! Title-screen stub now; future home for HUD, pause overlay, settings, post-run summary.

use bevy::prelude::*;

pub mod main_menu;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::state::GameState::MainMenu),
            main_menu::spawn_main_menu,
        )
        .add_systems(
            Update,
            main_menu::handle_main_menu_input.run_if(in_state(crate::state::GameState::MainMenu)),
        )
        .add_systems(
            OnExit(crate::state::GameState::MainMenu),
            main_menu::cleanup_main_menu,
        );
    }
}
