//! Visual presentation plugin: toon shader, outlines, palette.
//! Story 2.1 establishes the skeleton + a dev-only reference scene gated by debug_assertions.
//! Story 2.2 adds the SemanticAccent palette primitives (FR50 / NFR-A1 foundation).
//! Story 2.3 adds the WGSL `ToonMaterial` (FR49) wired through `MaterialPlugin`.

use bevy::prelude::*;

pub mod palette;
pub mod toon_material;

pub struct VisualPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum VisualSystems {
    Setup,
}

impl Plugin for VisualPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<toon_material::ToonMaterial>::default());

        app.configure_sets(
            OnEnter(crate::state::GameState::Loading),
            VisualSystems::Setup,
        );

        app.add_systems(
            Update,
            apply_tuning_to_toon_materials.in_set(crate::tuning::TuningSystems::Reload),
        );

        #[cfg(debug_assertions)]
        app.add_plugins(reference_scene::ReferenceScenePlugin);
    }
}

fn apply_tuning_to_toon_materials(
    mut events: MessageReader<crate::tuning::TuningReloaded>,
    mut materials: ResMut<Assets<toon_material::ToonMaterial>>,
) {
    for event in events.read() {
        for (_, material) in materials.iter_mut() {
            material.steps = event.0.toon_steps;
            material.rim_power = event.0.toon_rim_power;
            material.rim_intensity = event.0.toon_rim_intensity;
        }
    }
}

#[cfg(debug_assertions)]
mod reference_scene;
