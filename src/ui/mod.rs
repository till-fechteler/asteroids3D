//! UiPlugin — bevy_ui screen-space UI surfaces.
//! Title-screen stub now; future home for HUD, pause overlay, settings, post-run summary.

use bevy::prelude::*;

pub mod hud;
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

        // 3.11: HUD spawn on MainMenu → Arena. Cleanup transitively via
        // cleanup_on_exit::<ArenaEntity> registered by ArenaPlugin (HUD
        // entities are dual-marked HudEntity + ArenaEntity).
        app.add_systems(
            OnTransition {
                exited: crate::state::GameState::MainMenu,
                entered: crate::state::GameState::Arena,
            },
            hud::spawn_hud,
        );

        // 4.3: live Hull wiring — read PlayerShip Health and write to the
        // Hull HUD text node. Update-schedule + Arena gate keeps Pause-state
        // HUD updates suppressed (paused gameplay should not visually mutate).
        app.add_systems(
            Update,
            hud::update_hud_hull.run_if(in_state(crate::state::GameState::Arena)),
        );
    }
}
