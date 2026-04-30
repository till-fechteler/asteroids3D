//! ArenaPlugin — owns GameState::Arena entity lifecycle (spawn / cleanup).
//! Later stories attach the player ship, projectiles, and HUD via additional systems.

pub mod zone;

use bevy::prelude::*;

pub struct ArenaPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ArenaSystems {
    Setup,
}

#[derive(Component)]
pub struct ArenaEntity;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(OnEnter(crate::state::GameState::Arena), ArenaSystems::Setup);
        app.add_systems(
            OnEnter(crate::state::GameState::Arena),
            zone::spawn_arena_zone.in_set(ArenaSystems::Setup),
        );
        app.add_systems(
            OnExit(crate::state::GameState::Arena),
            cleanup_on_exit::<ArenaEntity>,
        );
    }
}

pub fn cleanup_on_exit<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
