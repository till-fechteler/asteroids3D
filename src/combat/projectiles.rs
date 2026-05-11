//! Projectile firing + ballistics + lifecycle (FR9). Owns:
//!   - `attach_combat_to_player_ship`: OnEnter(Arena) — adds combat input
//!     map + cooldown to the PlayerShip after FlightSystems::Setup.
//!   - `fire_primary_weapon`: FixedUpdate — reads CombatAction, gates by
//!     PrimaryWeaponCooldown, spawns Projectile entities at ship muzzle.
//!   - `tick_projectile_ttl`: FixedUpdate — decrements ttl, despawns
//!     expired projectiles.
//!
//! Story 3.10 adds CollisionLayers + ProjectileHitAsteroid event +
//! damage routing on top of the entity bundle established here.

use avian3d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, LinearVelocity, RigidBody,
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::arena::ArenaEntity;
use crate::combat::components::{PrimaryWeaponCooldown, Projectile};
use crate::combat::damage::GameLayer;
use crate::combat::input::{CombatAction, default_input_map};
use crate::combat::weapons::{WeaponArchetype, WeaponLoadout, spread_forwards};
use crate::flight::PlayerShip;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;
use crate::visual::palette::{SemanticAccent, color_for};
use crate::visual::toon_material::ToonMaterial;

/// Muzzle clearance: must exceed (ship_collider_radius=2.0 + projectile_radius=0.2) =
/// 2.2 m so a freshly spawned projectile does not overlap the ship's collider.
/// 3.0 m gives ~0.8 m of safety margin against ship motion within the spawn frame.
const PROJECTILE_SPAWN_OFFSET: f32 = 3.0;

/// Projectile mesh AND collider radius (matched for collision-trustworthiness
/// per the Story 3.3 mesh==collider precedent). Small-but-visible from cockpit.
/// Used by Pulse + Shotgun archetypes; Railgun uses a capsule (see below).
const PROJECTILE_RADIUS: f32 = 0.2;

/// Story 4.4 — Railgun lance shape (placeholder-grade per epic-4.4 AC line 146;
/// final visuals are Epic 10 polish). Capsule cross-section radius matches
/// `PROJECTILE_RADIUS` for spawn-clearance parity with Pulse/Shotgun; the
/// `LENGTH` is the cylindrical section between hemisphere caps (Bevy convention).
/// Total tip-to-tip extent = LENGTH + 2*RADIUS = 1.4 m; half-extent along the
/// flight axis = RADIUS + LENGTH/2 = 0.7 m; spawn clearance margin against the
/// 2.0 m ship collider at PROJECTILE_SPAWN_OFFSET=3.0 is 0.3 m (positive).
const RAILGUN_CAPSULE_RADIUS: f32 = 0.2;
const RAILGUN_CAPSULE_LENGTH: f32 = 1.0;

/// OnEnter(Arena) — attach combat input map + cooldown to the PlayerShip
/// AFTER FlightSystems::Setup has spawned the ship. CombatPlugin owns these
/// components; flight remains unaware of combat per the one-way dependency.
pub fn attach_combat_to_player_ship(
    mut commands: Commands,
    ships: Query<Entity, With<PlayerShip>>,
) {
    for entity in &ships {
        commands.entity(entity).insert((
            default_input_map(),
            ActionState::<CombatAction>::default(),
            PrimaryWeaponCooldown::default(),
        ));
    }
}

/// World-space initial velocity for a freshly fired projectile. Combines the
/// ship's current world velocity with `forward * projectile_speed` so a
/// projectile fired while drifting inherits the ship's momentum (Newtonian
/// muzzle-velocity composition; matches Epic 3 AC for "fires while drifting"
/// case → world velocity = ship velocity + projectile_speed forward).
///
/// `forward` is expected to be a unit vector (caller obtains via
/// `*transform.forward()`); the helper performs no normalization or NaN
/// guarding (consistent with the unclamped flight-physics helpers — input
/// hardening lives in TuningConfig deserialization, deferred per
/// deferred-work.md:222 + :228).
pub fn projectile_initial_velocity(
    ship_velocity: Vec3,
    forward: Vec3,
    projectile_speed: f32,
) -> Vec3 {
    ship_velocity + forward * projectile_speed
}

pub fn fire_primary_weapon(
    time: Res<Time>,
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    mut ships: Query<
        (
            &ActionState<CombatAction>,
            &Transform,
            &LinearVelocity,
            &mut PrimaryWeaponCooldown,
            &WeaponLoadout,
        ),
        With<PlayerShip>,
    >,
) {
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    let dt = time.delta_secs();
    for (action, transform, ship_velocity, mut cooldown, loadout) in &mut ships {
        cooldown.remaining = (cooldown.remaining - dt).max(0.0);
        if !action.pressed(&CombatAction::FirePrimary) || cooldown.remaining > 0.0 {
            continue;
        }
        // Active-archetype lookup — skip the fire path if the active slot is
        // empty (Epic 7 unlock-shop forward-compat).
        let Some(archetype) = loadout.active() else {
            continue;
        };
        let stats = archetype.stats_from(&tuning);

        // Defensive guards against tuning corruption / zero-stats. Closes
        // deferred-work.md:289 (zero-damage projectile silently despawning).
        // Cooldown is reset before continuing so the warn fires at fire_rate_hz
        // frequency rather than every FixedUpdate tick while LMB is held.
        if stats.damage == 0 {
            warn!(
                "zero-damage archetype {:?} suppressed (would despawn projectiles harmlessly)",
                archetype
            );
            cooldown.remaining = 1.0 / stats.fire_rate_hz.max(f32::EPSILON);
            continue;
        }
        if stats.projectile_count == 0 {
            warn!("zero-projectile archetype {:?} suppressed", archetype);
            cooldown.remaining = 1.0 / stats.fire_rate_hz.max(f32::EPSILON);
            continue;
        }

        let forward: Vec3 = *transform.forward();
        let up: Vec3 = *transform.up();
        let directions = spread_forwards(forward, up, stats.projectile_count, stats.spread_deg);

        for direction in directions {
            let spawn_pos = transform.translation + direction * PROJECTILE_SPAWN_OFFSET;
            let velocity =
                projectile_initial_velocity(ship_velocity.0, direction, stats.projectile_speed);

            // Per-archetype mesh + collider + spawn-rotation (placeholder-grade
            // per epic-4.4 AC line 146; Epic 10 polish-pass owns final visuals):
            //   - Pulse/Shotgun: 0.2 m sphere, no rotation needed.
            //   - Railgun: 0.2 m × 1.0 m capsule (lance), rotated so its Y-axis
            //     (Bevy capsule convention) aligns with the projectile direction.
            let (projectile_mesh, projectile_collider, projectile_rotation) = match archetype {
                WeaponArchetype::Pulse | WeaponArchetype::Shotgun => (
                    meshes.add(
                        Sphere::new(PROJECTILE_RADIUS)
                            .mesh()
                            .ico(2)
                            .expect("ico(2): subdivision=2 is within MAX_SUBDIVISIONS=80"),
                    ),
                    Collider::sphere(PROJECTILE_RADIUS),
                    Quat::IDENTITY,
                ),
                WeaponArchetype::Railgun => (
                    meshes.add(Capsule3d::new(
                        RAILGUN_CAPSULE_RADIUS,
                        RAILGUN_CAPSULE_LENGTH,
                    )),
                    Collider::capsule(RAILGUN_CAPSULE_RADIUS, RAILGUN_CAPSULE_LENGTH),
                    Quat::from_rotation_arc(Vec3::Y, direction),
                ),
            };
            let projectile_material = materials.add(ToonMaterial {
                tint: color_for(SemanticAccent::Neutral).into(),
                ..default()
            });

            commands.spawn((
                Projectile {
                    ttl: tuning.projectile_ttl_seconds,
                    damage: stats.damage,
                },
                ArenaEntity,
                Mesh3d(projectile_mesh),
                MeshMaterial3d(projectile_material),
                Transform::from_translation(spawn_pos).with_rotation(projectile_rotation),
                RigidBody::Dynamic,
                projectile_collider,
                LinearVelocity(velocity),
                CollisionLayers::new(
                    [GameLayer::Projectile],
                    [GameLayer::Asteroid, GameLayer::Enemy],
                ),
                CollisionEventsEnabled,
            ));
        }

        cooldown.remaining = 1.0 / stats.fire_rate_hz.max(f32::EPSILON);

        info!(
            "fired {:?}: {} projectiles at fire_rate={:.2}Hz (next-fire {:.2}s)",
            archetype, stats.projectile_count, stats.fire_rate_hz, cooldown.remaining
        );
    }
}

pub fn tick_projectile_ttl(
    time: Res<Time>,
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Projectile)>,
) {
    let dt = time.delta_secs();
    for (entity, mut projectile) in &mut projectiles {
        projectile.ttl -= dt;
        if projectile.ttl <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projectile_initial_velocity_stationary_ship_returns_speed_along_forward() {
        let v = projectile_initial_velocity(Vec3::ZERO, Vec3::NEG_Z, 120.0);
        assert!(
            (v - Vec3::new(0.0, 0.0, -120.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn projectile_initial_velocity_drifting_ship_inherits_ship_momentum() {
        // Ship at (0,0,-30), forward=NEG_Z, speed=120 → world velocity (0,0,-150).
        let v = projectile_initial_velocity(Vec3::new(0.0, 0.0, -30.0), Vec3::NEG_Z, 120.0);
        assert!(
            (v - Vec3::new(0.0, 0.0, -150.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn projectile_initial_velocity_strafing_ship_combines_orthogonal_motion() {
        // Lateral ship motion at (20,0,0) does not bend forward speed; both sum.
        let v = projectile_initial_velocity(Vec3::new(20.0, 0.0, 0.0), Vec3::NEG_Z, 120.0);
        assert!(
            (v - Vec3::new(20.0, 0.0, -120.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn projectile_initial_velocity_zero_speed_returns_ship_velocity_unchanged() {
        // speed=0 → projectile inherits ONLY ship velocity (degenerate-but-defined).
        let v = projectile_initial_velocity(Vec3::new(5.0, 10.0, -2.0), Vec3::NEG_Z, 0.0);
        assert!(
            (v - Vec3::new(5.0, 10.0, -2.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }
}
