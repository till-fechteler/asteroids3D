//! FlightPlugin — owns PlayerShip + CockpitCamera spawn, 6-DOF translation, 3-axis
//! rotation, inertial dampener toggle, and Arena cursor-grab. Weapons land in subsequent stories.

pub mod components;
pub mod input;
pub mod physics;

use avian3d::prelude::{
    AngularVelocity, Collider, CollisionEventsEnabled, LinearVelocity, RigidBody,
};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_mod_outline::OutlineVolume;
use leafwing_input_manager::prelude::*;

use crate::arena::{ArenaEntity, ArenaSystems};
use crate::combat::health::Health;
use crate::flight::components::DampenerState;
use crate::flight::input::{FlightAction, default_input_map};
use crate::flight::physics::{MouseLookDelta, MouseLookSuppressFrames};
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
        // Spawn lives on `OnTransition { MainMenu → Arena }` (NOT `OnEnter(Arena)`) so
        // Pause round-trip (Arena ↔ Paused) does not respawn the ship at origin.
        app.configure_sets(
            OnTransition {
                exited: GameState::MainMenu,
                entered: GameState::Arena,
            },
            (ArenaSystems::Setup, FlightSystems::Setup).chain(),
        );
        app.add_systems(
            OnTransition {
                exited: GameState::MainMenu,
                entered: GameState::Arena,
            },
            spawn_player_ship.in_set(FlightSystems::Setup),
        );

        // add_plugins first so ActionState<A> is populated by leafwing's PreUpdate before our FixedUpdate reads it.
        app.add_plugins(InputManagerPlugin::<FlightAction>::default());
        app.init_resource::<MouseLookDelta>();
        app.init_resource::<MouseLookSuppressFrames>();
        app.add_systems(
            PreUpdate,
            physics::accumulate_mouse_look.run_if(in_state(GameState::Arena)),
        );
        app.configure_sets(FixedUpdate, FlightSystems::ApplyForces);
        app.add_systems(
            FixedUpdate,
            (
                physics::apply_thrust,
                physics::apply_torque,
                physics::apply_dampener,
            )
                .in_set(FlightSystems::ApplyForces)
                .run_if(in_state(GameState::Arena)),
        );
        app.add_systems(
            Update,
            physics::toggle_dampener.run_if(in_state(GameState::Arena)),
        );
        app.add_systems(OnEnter(GameState::Arena), grab_cursor_for_arena);
        app.add_systems(OnExit(GameState::Arena), release_cursor_on_arena_exit);
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
            CollisionEventsEnabled,
            Health {
                current: tuning.player_hull_max,
                max: tuning.player_hull_max,
            },
            default_input_map(),
            ActionState::<FlightAction>::default(),
            DampenerState::default(),
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

pub fn grab_cursor_for_arena(
    mut window: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut buffer: ResMut<MouseLookDelta>,
    mut suppress: ResMut<MouseLookSuppressFrames>,
) {
    // CursorGrabMode::Confined: native on Windows / X11; on macOS Bevy auto-falls-back
    // to Locked (per bevy_window-0.18 platform notes). Both achieve cockpit-aim feel.
    window.grab_mode = CursorGrabMode::Confined;
    window.visible = false;
    // Discard any pre-grab accumulated mouse motion and suppress the next 3
    // PreUpdate accumulations so the OS cursor-warp delta (arrives 1–2 frames
    // after grab) does not register as a torque spike on Arena entry / resume.
    buffer.0 = Vec2::ZERO;
    suppress.0 = 3;
}

pub fn release_cursor_on_arena_exit(mut window: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    window.grab_mode = CursorGrabMode::None;
    window.visible = true;
}
