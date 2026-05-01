//! FlightPlugin — owns PlayerShip + CockpitCamera spawn and 6-DOF translation thrust.
//! Rotation, dampener, weapons land in subsequent stories via additional systems.

pub mod input;
pub mod physics;

use avian3d::prelude::{AngularVelocity, Collider, LinearVelocity, RigidBody};
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;
use leafwing_input_manager::prelude::*;

use crate::arena::{ArenaEntity, ArenaSystems};
use crate::flight::input::{FlightAction, default_input_map};
use crate::state::GameState;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;
use crate::visual::palette::{SemanticAccent, color_for};
use crate::visual::toon_material::ToonMaterial;

pub struct FlightPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum FlightSystems {
    Setup,
    ApplyForces,
}

#[derive(Component)]
pub struct PlayerShip;

#[derive(Component)]
pub struct CockpitCamera;

impl Plugin for FlightPlugin {
    fn build(&self, app: &mut App) {
        // Cross-plugin chain: PlayerShip spawn must run after spawn_arena_zone so the
        // origin-cleared corridor (≥3 asteroids within 50 m) is queryable. Architecture
        // forbids `.after(specific_function)`; configure_sets is the approved pattern.
        app.configure_sets(
            OnEnter(GameState::Arena),
            (ArenaSystems::Setup, FlightSystems::Setup).chain(),
        );
        app.add_systems(
            OnEnter(GameState::Arena),
            spawn_player_ship.in_set(FlightSystems::Setup),
        );

        // add_plugins first so ActionState<A> is populated by leafwing's PreUpdate before our FixedUpdate reads it.
        app.add_plugins(InputManagerPlugin::<FlightAction>::default());
        app.configure_sets(FixedUpdate, FlightSystems::ApplyForces);
        app.add_systems(
            FixedUpdate,
            physics::apply_thrust
                .in_set(FlightSystems::ApplyForces)
                .run_if(in_state(GameState::Arena)),
        );
    }
}

pub fn spawn_player_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
) {
    // Cold-start safety mirrors src/arena/zone.rs:48-54 — tuning.ron may not be loaded
    // yet on a hypothetical future re-entry path; fall back to defaults with a warn.
    let tuning_opt = tuning_assets.get(tuning_handle.0.id());
    if tuning_opt.is_none() {
        warn!("tuning.ron not loaded at PlayerShip spawn; using TuningConfig defaults");
    }
    let tuning = tuning_opt.cloned().unwrap_or_default();
    let [r, g, b, a] = tuning.outline_color;
    let outline = OutlineVolume {
        visible: true,
        width: tuning.outline_width,
        colour: Color::srgba(r, g, b, a),
    };

    let ship_mesh = meshes.add(Cuboid::new(4.0, 2.0, 6.0));
    let ship_material = materials.add(ToonMaterial {
        tint: color_for(SemanticAccent::Neutral).into(),
        ..default()
    });

    commands
        .spawn((
            PlayerShip,
            ArenaEntity,
            Mesh3d(ship_mesh),
            MeshMaterial3d(ship_material),
            Transform::from_xyz(0.0, 0.0, 0.0),
            outline,
            RigidBody::Dynamic,
            Collider::sphere(2.0),
            LinearVelocity(Vec3::ZERO),
            AngularVelocity(Vec3::ZERO),
            default_input_map(),
            ActionState::<FlightAction>::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                CockpitCamera,
                Transform::from_xyz(0.0, 0.6, 0.5),
            ));
        });

    info!("spawned PlayerShip at origin with cockpit Camera3d child");
}
