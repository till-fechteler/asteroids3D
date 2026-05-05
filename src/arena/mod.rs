//! ArenaPlugin — owns GameState::Arena entity lifecycle (spawn / cleanup).
//! Spawn and cleanup are scoped to `OnTransition` (specific from→to pairs)
//! rather than blanket `OnEnter` / `OnExit`, so the Pause round-trip
//! (Arena ↔ Paused) preserves entities across the transition. Later stories
//! attach the player ship, projectiles, and HUD via additional systems.

pub mod zone;

use bevy::prelude::*;

use crate::state::GameState;

pub struct ArenaPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ArenaSystems {
    Setup,
}

#[derive(Component)]
pub struct ArenaEntity;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            OnTransition {
                exited: GameState::MainMenu,
                entered: GameState::Arena,
            },
            ArenaSystems::Setup,
        );
        app.add_systems(
            OnTransition {
                exited: GameState::MainMenu,
                entered: GameState::Arena,
            },
            zone::spawn_arena_zone.in_set(ArenaSystems::Setup),
        );
        // Cleanup runs only on terminal exits (Arena → MainMenu / PostRun).
        // Forward-compat: Arena → MainMenu wiring lands in Story 4.7 title-screen
        // restart flow; Arena → PostRun in Epic 4 death/run-end flow. Until then,
        // the cleanup branch is dormant — Pause round-trip preserves all
        // ArenaEntity-marked entities (PlayerShip, asteroids, projectiles).
        app.add_systems(
            OnTransition {
                exited: GameState::Arena,
                entered: GameState::MainMenu,
            },
            cleanup_on_exit::<ArenaEntity>,
        );
    }
}

pub fn cleanup_on_exit<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
