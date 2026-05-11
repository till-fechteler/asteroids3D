//! Damage events + asteroid HP routing (FR12). Owns:
//!   - `GameLayer` — Avian PhysicsLayer enum: Default (ship), Asteroid, Projectile.
//!   - `ProjectileHitAsteroid` / `AsteroidDestroyed` events (Bevy 0.18 Messages).
//!   - `apply_damage` — pure saturating-subtraction helper.
//!   - `detect_projectile_asteroid_hits` — FixedUpdate, reads CollisionStart,
//!     emits ProjectileHitAsteroid (handles both collider-pair orderings).
//!   - `apply_asteroid_damage` — FixedUpdate, reads ProjectileHitAsteroid,
//!     mutates AsteroidHp, despawns projectile (single-hit), despawns
//!     asteroid + emits AsteroidDestroyed when HP reaches zero.
//!
//! Story 3.10 ships single-hit asteroids (current = 1, damage = 1). Epic 4
//! adds enemy projectiles + HullHP/ShieldHP routing through the same
//! event chain pattern. Epic 5 adds multi-HP asteroids that exercise
//! apply_damage's partial-reduction branch.

use avian3d::prelude::{CollisionStart, PhysicsLayer};
use bevy::prelude::*;

use crate::combat::components::{Asteroid, Projectile};
use crate::combat::enemy::Enemy;
use crate::combat::enemy_ai::EnemyProjectile;
use crate::combat::health::Health;
use crate::flight::PlayerShip;
use crate::state::GameState;

/// Avian collision-layer enum. Bit 0 (`Default`) is reserved for the implicit
/// default layer; the player ship stays on it (no CollisionLayers component on
/// PlayerShip → ship inherits `CollisionLayers::DEFAULT`). Asteroids and
/// projectiles get explicit memberships + filters per Story 3.10 AC #2/#3.
#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Asteroid,
    Projectile,
    Enemy,
}

/// Emitted by `detect_projectile_asteroid_hits` when a projectile and an
/// asteroid begin contacting. Consumed by `apply_asteroid_damage` to apply
/// damage and despawn the projectile.
#[derive(Message, Debug, Clone, Copy)]
pub struct ProjectileHitAsteroid {
    pub projectile: Entity,
    pub asteroid: Entity,
    pub damage: u32,
}

/// Emitted by `apply_asteroid_damage` when an asteroid's HP reaches zero.
/// MVP minimal payload; later epics may extend with position / awarded_salvage
/// / destroyed_by per architecture.md:398.
#[derive(Message, Debug, Clone, Copy)]
#[allow(
    dead_code,
    reason = "AsteroidDestroyed.asteroid is read by Epic 4 enemy AI and Epic 6 salvage — pre-wired here per the architecture-prescribed event shape"
)]
pub struct AsteroidDestroyed {
    pub asteroid: Entity,
}

/// Emitted by `detect_projectile_enemy_hits` when a player projectile and
/// an enemy begin contacting. Consumed by `apply_enemy_damage`.
#[derive(Message, Debug, Clone, Copy)]
pub struct ProjectileHitEnemy {
    pub projectile: Entity,
    pub enemy: Entity,
    pub damage: u32,
}

/// Emitted by `apply_enemy_damage` when an enemy's Health reaches zero.
/// Story 4.5 (salvage retro-tint) and Epic 8 (audio cues) are downstream consumers.
#[derive(Message, Debug, Clone, Copy)]
#[allow(
    dead_code,
    reason = "EnemyDestroyed.enemy is read by Story 4.5 salvage retro-tint and Epic 8 audio cues — pre-wired here per the architecture-prescribed event shape"
)]
pub struct EnemyDestroyed {
    pub enemy: Entity,
}

/// Cause-of-death taxonomy for `HullDepleted`. Story 4.3 only routes
/// `EnemyFire`; `AsteroidCollision` and `Unknown` are pre-wired enum
/// variants for forward-compat (asteroid-collision damage is a polish-pass
/// item; `Unknown` covers future unattributable damage sources).
///
/// Forward-compat note: when Epic 6 Story 6.1 introduces `RunPlugin`,
/// `DeathCause` (and `RunResult`, `RunStartedAt`) may relocate from
/// `src/combat/damage.rs` to `src/run/director.rs` per architecture.md:687.
/// No `Default` derive — silent default would mask attribution bugs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    EnemyFire,
    #[allow(
        dead_code,
        reason = "AsteroidCollision is a forward-compat enum slot; Story 4.3 routes only EnemyFire. Asteroid-collision damage attribution lands as a future polish-pass item."
    )]
    AsteroidCollision,
    #[allow(
        dead_code,
        reason = "Unknown is a forward-compat enum slot for future unattributable damage sources (e.g., out-of-bounds, scripted hazards)."
    )]
    Unknown,
}

/// Run-end summary handed to PostRun (Story 4.9 consumer). Inserted as a
/// `Resource` by `apply_player_damage` on Hull=0, read by Story 4.9's
/// post-run summary screen, removed by Story 4.9 on PostRun → MainMenu /
/// PostRun → Arena exit. Story 4.3 hardcodes `salvage_banked: 0` (Epic 6
/// economy wires real value).
///
/// No `Default` derive — silent default would imply zero-cause-zero-duration
/// which is misleading. Caller-side struct-literal construction matches the
/// project's `Health` / `Projectile` / `EnemyShip` no-Default precedent.
#[derive(Resource, Debug, Clone, Copy)]
#[allow(
    dead_code,
    reason = "RunResult fields are read by Story 4.9's PostRun summary screen; Story 4.3 wires the resource and producer only."
)]
pub struct RunResult {
    pub cause: DeathCause,
    pub salvage_banked: u32,
    pub run_duration_seconds: f32,
}

/// Anchor for `RunResult.run_duration_seconds`. Records the value of
/// `Time<Virtual>::elapsed_secs()` at Arena entry; subtracted at HullDepleted
/// to yield wall-clock-of-unpaused-gameplay duration. Idempotent overwrite
/// at every Arena entry — no cleanup needed.
///
/// `Time<Virtual>` (NOT `Time<Real>`) is the right clock: it pauses with
/// `pause_simulation_clocks` (`pause/mod.rs:122`), so paused intervals do
/// NOT inflate `run_duration_seconds`. Forward-compat: same relocation note
/// as `RunResult` / `DeathCause`.
#[derive(Resource, Debug, Clone, Copy)]
pub struct RunStartedAt(pub f32);

/// Emitted by `detect_projectile_player_hits` when an enemy projectile and
/// the PlayerShip begin contacting. Consumed by `apply_player_damage`.
/// Field naming mirrors `ProjectileHitAsteroid` / `ProjectileHitEnemy` for
/// cross-event symmetry.
#[derive(Message, Debug, Clone, Copy)]
pub struct ProjectileHitPlayer {
    pub projectile: Entity,
    pub player: Entity,
    pub damage: u32,
}

/// Emitted by `apply_player_damage` when PlayerShip Health reaches zero.
/// Triggers `NextState<GameState>::PostRun` and the `RunResult` insertion.
/// Story 4.9's PostRun summary screen is the downstream consumer.
#[derive(Message, Debug, Clone, Copy)]
pub struct HullDepleted {
    pub player: Entity,
    pub cause: DeathCause,
}

/// Saturating-subtraction damage application. Returns the new HP, clamped at
/// zero to prevent underflow on over-damage cases (e.g., a Story 4.4 high-damage
/// weapon vs. a 1-HP asteroid). Pure function — no ECS access, trivially
/// testable in isolation. Symmetric helper to projectile_initial_velocity
/// from Story 3.9.
pub fn apply_damage(current: u32, damage: u32) -> u32 {
    current.saturating_sub(damage)
}

/// FixedUpdate — reads Avian's `CollisionStart` messages and emits
/// `ProjectileHitAsteroid` whenever a projectile-asteroid contact pair is
/// detected. Avian gives no canonical (collider1, collider2) ordering, so
/// both orderings are tried.
pub fn detect_projectile_asteroid_hits(
    mut collisions: MessageReader<CollisionStart>,
    projectiles: Query<&Projectile>,
    asteroids: Query<(), With<Asteroid>>,
    mut hits: MessageWriter<ProjectileHitAsteroid>,
) {
    for event in collisions.read() {
        // Resolve which side of the contact pair is the projectile vs. asteroid.
        // Avian's CollisionStart gives no canonical ordering, so we try both.
        let (projectile_entity, asteroid_entity) = if projectiles.get(event.collider1).is_ok()
            && asteroids.get(event.collider2).is_ok()
        {
            (event.collider1, event.collider2)
        } else if asteroids.get(event.collider1).is_ok() && projectiles.get(event.collider2).is_ok()
        {
            (event.collider2, event.collider1)
        } else {
            continue; // Not a projectile-asteroid pair (e.g., ship↔asteroid bounce).
        };

        // projectile_entity was validated by .is_ok() in the if-condition above.
        let projectile = projectiles
            .get(projectile_entity)
            .expect("projectile_entity verified above");
        hits.write(ProjectileHitAsteroid {
            projectile: projectile_entity,
            asteroid: asteroid_entity,
            damage: projectile.damage,
        });
    }
}

/// FixedUpdate — reads Avian's `CollisionStart` messages and emits
/// `ProjectileHitEnemy` whenever a projectile-enemy contact pair is detected.
/// NO `EnemyProjectile` filter is needed — enemy projectiles use filter mask
/// `[GameLayer::Default]` (cannot reach the `Enemy` layer), so only player
/// projectiles can collide with enemies. Mirrors the asteroid-pair pattern.
pub fn detect_projectile_enemy_hits(
    mut collisions: MessageReader<CollisionStart>,
    projectiles: Query<&Projectile>,
    enemies: Query<(), With<Enemy>>,
    mut hits: MessageWriter<ProjectileHitEnemy>,
) {
    for event in collisions.read() {
        let (projectile_entity, enemy_entity) = if projectiles.get(event.collider1).is_ok()
            && enemies.get(event.collider2).is_ok()
        {
            (event.collider1, event.collider2)
        } else if enemies.get(event.collider1).is_ok() && projectiles.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue; // Not a projectile-enemy pair.
        };

        let projectile = projectiles
            .get(projectile_entity)
            .expect("projectile_entity verified above");
        hits.write(ProjectileHitEnemy {
            projectile: projectile_entity,
            enemy: enemy_entity,
            damage: projectile.damage,
        });
    }
}

/// FixedUpdate — applies damage from `ProjectileHitEnemy` events to enemy
/// `Health` components, despawns the projectile, and on HP=0 despawns the
/// enemy + emits `EnemyDestroyed`. Mirrors `apply_asteroid_damage`.
pub fn apply_enemy_damage(
    mut hits: MessageReader<ProjectileHitEnemy>,
    mut commands: Commands,
    mut enemies: Query<&mut Health, With<Enemy>>,
    mut destroyed: MessageWriter<EnemyDestroyed>,
) {
    for event in hits.read() {
        commands.entity(event.projectile).despawn();
        if let Ok(mut hp) = enemies.get_mut(event.enemy) {
            if hp.current == 0 {
                continue;
            }
            hp.current = apply_damage(hp.current, event.damage);
            if hp.current == 0 {
                commands.entity(event.enemy).despawn();
                destroyed.write(EnemyDestroyed { enemy: event.enemy });
                info!("enemy destroyed: entity={:?}", event.enemy);
            }
        }
    }
}

/// FixedUpdate — reads Avian's `CollisionStart` messages and emits
/// `ProjectileHitPlayer` whenever an enemy-projectile vs. PlayerShip pair
/// is detected. Filtered by `With<EnemyProjectile>` so player-fired
/// projectiles cannot self-damage (defense-in-depth; the collision-layer
/// design also prevents player-projectile→ship contact).
pub fn detect_projectile_player_hits(
    mut collisions: MessageReader<CollisionStart>,
    projectiles: Query<&Projectile, With<EnemyProjectile>>,
    players: Query<(), With<PlayerShip>>,
    mut hits: MessageWriter<ProjectileHitPlayer>,
) {
    for event in collisions.read() {
        let (projectile_entity, player_entity) = if projectiles.get(event.collider1).is_ok()
            && players.get(event.collider2).is_ok()
        {
            (event.collider1, event.collider2)
        } else if players.get(event.collider1).is_ok() && projectiles.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue; // Not an enemy-projectile-vs-player pair.
        };

        let projectile = projectiles
            .get(projectile_entity)
            .expect("projectile_entity verified above");
        hits.write(ProjectileHitPlayer {
            projectile: projectile_entity,
            player: player_entity,
            damage: projectile.damage,
        });
    }
}

/// FixedUpdate — applies damage from `ProjectileHitPlayer` events to the
/// PlayerShip's `Health`, despawns the projectile (single-hit), and on HP=0
/// emits `HullDepleted` + inserts `RunResult` as a Resource. The PlayerShip
/// itself is NOT despawned here — Arena → PostRun cleanup is owned by
/// `cleanup_on_exit::<ArenaEntity>` registered on the state transition.
pub fn apply_player_damage(
    mut hits: MessageReader<ProjectileHitPlayer>,
    mut commands: Commands,
    mut players: Query<&mut Health, With<PlayerShip>>,
    mut depleted: MessageWriter<HullDepleted>,
    run_start: Res<RunStartedAt>,
    virtual_time: Res<Time<Virtual>>,
) {
    for event in hits.read() {
        // Despawn projectile unconditionally — single-hit-per-projectile (Epic 3 AC).
        commands.entity(event.projectile).despawn();
        // Apply damage if the player is still alive. Two guards mirror
        // `apply_asteroid_damage`: (1) entity gone (programmer-error case for
        // the player; ship despawn does NOT happen in 4.3 but the guard is
        // symmetry-preserving free); (2) hp.current == 0 already-dead-this-tick
        // guard for multi-hit-same-tick races.
        if let Ok(mut hp) = players.get_mut(event.player) {
            if hp.current == 0 {
                continue;
            }
            hp.current = apply_damage(hp.current, event.damage);
            if hp.current == 0 {
                let cause = DeathCause::EnemyFire;
                let run_duration_seconds = virtual_time.elapsed_secs() - run_start.0;
                depleted.write(HullDepleted {
                    player: event.player,
                    cause,
                });
                commands.insert_resource(RunResult {
                    cause,
                    salvage_banked: 0,
                    run_duration_seconds,
                });
                info!(
                    "hull depleted: cause={:?} run_duration={:.2}s",
                    cause, run_duration_seconds
                );
            }
        }
    }
}

/// FixedUpdate — reads `HullDepleted` events and triggers the
/// `Arena → PostRun` state transition. Multiple events within a single tick
/// (multi-projectile-hit case) collapse to a single transition: only the
/// first event is consumed; remaining events stay in the reader's cursor
/// until the system runs again next tick (by which point Arena has exited
/// and the run-if guard suppresses re-entry).
pub fn check_player_death(
    mut depleted: MessageReader<HullDepleted>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(event) = depleted.read().next() {
        info!(
            "transitioning to PostRun (cause={:?} player={:?})",
            event.cause, event.player
        );
        next_state.set(GameState::PostRun);
    }
}

/// OnTransition MainMenu → Arena — captures the wall-clock-of-unpaused-gameplay
/// anchor for `RunResult.run_duration_seconds`. `Time<Virtual>` is paused by
/// `pause_simulation_clocks`, so paused intervals do not inflate the duration.
pub fn record_run_started_at(virtual_time: Res<Time<Virtual>>, mut commands: Commands) {
    commands.insert_resource(RunStartedAt(virtual_time.elapsed_secs()));
    info!(
        "run started at virtual elapsed = {:.2}s",
        virtual_time.elapsed_secs()
    );
}

/// FixedUpdate — applies damage from `ProjectileHitAsteroid` events to
/// asteroid `Health` components, despawns the projectile (single-hit-per-projectile
/// per Epic 3 AC), and on HP=0 despawns the asteroid + emits `AsteroidDestroyed`.
pub fn apply_asteroid_damage(
    mut hits: MessageReader<ProjectileHitAsteroid>,
    mut commands: Commands,
    mut asteroids: Query<&mut Health, With<Asteroid>>,
    mut destroyed: MessageWriter<AsteroidDestroyed>,
) {
    for event in hits.read() {
        // Despawn projectile unconditionally — single-hit-per-projectile (Epic 3 AC).
        commands.entity(event.projectile).despawn();
        // Apply damage to asteroid if it's still alive. Two guards:
        // (1) entity gone: get_mut returns Err when a prior despawn was already
        //     flushed (cross-system case). (2) hp.current == 0: same-system case
        //     where a prior event this tick already reduced HP to zero and queued
        //     despawn, but the deferred command hasn't been flushed yet — entity
        //     still queryable. Without this guard, a second simultaneous hit would
        //     emit a duplicate AsteroidDestroyed and queue a redundant despawn.
        if let Ok(mut hp) = asteroids.get_mut(event.asteroid) {
            if hp.current == 0 {
                continue; // Already dead this tick; another hit queued destruction first.
            }
            hp.current = apply_damage(hp.current, event.damage);
            if hp.current == 0 {
                commands.entity(event.asteroid).despawn();
                destroyed.write(AsteroidDestroyed {
                    asteroid: event.asteroid,
                });
                info!("asteroid destroyed: entity={:?}", event.asteroid);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_damage_full_destruction_returns_zero() {
        // Standard case: 1 HP asteroid + 1 damage projectile = destruction.
        assert_eq!(apply_damage(1, 1), 0);
    }

    #[test]
    fn apply_damage_partial_returns_remaining_hp() {
        // Multi-hit forward-compat: Epic 5 multi-HP asteroids must reduce, not destroy.
        assert_eq!(apply_damage(3, 1), 2);
    }

    #[test]
    fn apply_damage_overdamage_clamps_at_zero() {
        // Saturating-sub guards against u32 underflow. Story 4.4 high-damage weapon
        // vs. low-HP target hits this branch.
        assert_eq!(apply_damage(1, 5), 0);
    }

    #[test]
    fn death_cause_variants_are_distinct() {
        // No Default derive — guards future variant additions; round-trip explicit
        // construction. Mirrors HudField::variants_distinct precedent at hud.rs:154.
        assert_ne!(DeathCause::EnemyFire, DeathCause::AsteroidCollision);
        assert_ne!(DeathCause::AsteroidCollision, DeathCause::Unknown);
        assert_ne!(DeathCause::EnemyFire, DeathCause::Unknown);
    }

    #[test]
    fn run_result_construction_is_explicit() {
        // No Default derive — silent default would imply zero-cause-zero-duration
        // which is misleading. Round-trip explicit construction.
        let r = RunResult {
            cause: DeathCause::EnemyFire,
            salvage_banked: 0,
            run_duration_seconds: 42.5,
        };
        assert_eq!(r.cause, DeathCause::EnemyFire);
        assert_eq!(r.salvage_banked, 0);
        assert!((r.run_duration_seconds - 42.5).abs() < 1e-5);
    }

    #[test]
    fn run_started_at_construction_is_explicit() {
        // No Default derive — captures intent at the call site.
        let r = RunStartedAt(0.0);
        assert_eq!(r.0, 0.0);
    }
}
