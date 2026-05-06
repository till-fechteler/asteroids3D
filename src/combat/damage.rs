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
use crate::combat::health::Health;

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
        } else if enemies.get(event.collider1).is_ok() && projectiles.get(event.collider2).is_ok()
        {
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
                destroyed.write(EnemyDestroyed {
                    enemy: event.enemy,
                });
                info!("enemy destroyed: entity={:?}", event.enemy);
            }
        }
    }
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
}
