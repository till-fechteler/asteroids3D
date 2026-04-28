//! Visual presentation plugin: toon shader, outlines, palette.
//! Story 2.1 establishes the skeleton + a dev-only reference scene gated by debug_assertions.
//! Story 2.2 adds the SemanticAccent palette primitives (FR50 / NFR-A1 foundation).

use bevy::prelude::*;

pub mod palette;

pub struct VisualPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VisualSystems {
    Setup,
}

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            OnEnter(crate::state::GameState::Loading),
            VisualSystems::Setup,
        );

        #[cfg(debug_assertions)]
        app.add_plugins(reference_scene::ReferenceScenePlugin);
    }
}

#[cfg(debug_assertions)]
mod reference_scene;
