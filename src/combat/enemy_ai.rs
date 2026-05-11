//! Enemy AI state machine (FR14) — Idle/Detect/Pursue/Attack distance-driven
//! transitions with hysteresis dead-band. Owns `apply_enemy_ai` (state transition,
//! orientation, and movement) and `enemy_fire_weapon` (Attack-state fire pacing).
//! Pure helper `next_ai_state` is the testable transition core.

use avian3d::prelude::{
    Collider, CollisionEventsEnabled, CollisionLayers, LinearVelocity, RigidBody,
};
use bevy::prelude::*;

use crate::arena::ArenaEntity;
use crate::combat::components::Projectile;
use crate::combat::damage::GameLayer;
use crate::combat::enemy::Enemy;
use crate::flight::PlayerShip;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;
use crate::visual::palette::{SemanticAccent, color_for};
use crate::visual::toon_material::ToonMaterial;

/// Spawn offset along the enemy→player direction for fired projectiles.
/// Mirrors `PROJECTILE_SPAWN_OFFSET` from `projectiles.rs` (player muzzle clearance).
/// Enemy capsule extent = radius 2.0 + half-length 2.0 = 4.0; offset of 3.0 plus
/// the projectile's `ENEMY_PROJECTILE_RADIUS` keeps the spawn outside the capsule.
const ENEMY_PROJECTILE_SPAWN_OFFSET: f32 = 3.0;

/// Mesh AND collider radius for enemy projectiles. Matches the player's
/// `PROJECTILE_RADIUS` so the visual ↔ physics correspondence is consistent.
const ENEMY_PROJECTILE_RADIUS: f32 = 0.2;

/// Enemy AI lifecycle state. Transitions are distance-driven with
/// hysteresis (see `next_ai_state`). Default is `Idle`; the first
/// FixedUpdate tick after Arena entry computes distance-to-player
/// and may transition immediately if the player is within range.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyAiState {
    #[default]
    Idle,
    Detect,
    Pursue,
    Attack,
}

/// Marker for enemy-fired projectiles. Story 4.3 keys hull-damage routing
/// on this marker (player-projectile vs. enemy-projectile differentiation).
/// Story 4.2 spawns these entities but does NOT route damage to PlayerShip;
/// damage routing is Story 4.3's scope.
#[derive(Component, Debug, Clone, Copy)]
#[allow(
    dead_code,
    reason = "EnemyProjectile marker is consumed by Story 4.3 hull-damage routing; spawned here per AC #4 to pre-wire the differentiator."
)]
pub struct EnemyProjectile;

/// Per-enemy primary-weapon rate-limit state mirroring `PrimaryWeaponCooldown`.
/// `remaining` counts seconds until the next shot is permitted. Default `0.0`
/// so the first Attack-state tick fires immediately.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct EnemyFireCooldown {
    pub remaining: f32,
}

/// Pure helper: compute the next AI state given current state, distance to
/// player, and tuning thresholds. No ECS access — first-class testable per
/// the project's pure-helper learning pattern.
///
/// Hysteresis dead-band: outward (less aggressive) thresholds widen by
/// `(1 + hysteresis_pct)` so a target hovering at the boundary does not
/// flicker between states tick-to-tick.
///
/// Single-step semantics: at most one band is crossed per call. A frame
/// where distance jumps from 200m to 10m moves Idle→Detect; subsequent
/// ticks complete Detect→Pursue→Attack. Acceptable trade-off at 60 Hz
/// FixedUpdate (worst-case ~33 ms latency).
pub fn next_ai_state(
    current: EnemyAiState,
    distance: f32,
    detection_range: f32,
    engagement_range: f32,
    hysteresis_pct: f32,
) -> EnemyAiState {
    let attack_range = engagement_range * 0.5;
    let outer = 1.0 + hysteresis_pct;
    match current {
        EnemyAiState::Idle => {
            if distance <= detection_range {
                EnemyAiState::Detect
            } else {
                EnemyAiState::Idle
            }
        }
        EnemyAiState::Detect => {
            if distance <= engagement_range {
                EnemyAiState::Pursue
            } else if distance > detection_range * outer {
                EnemyAiState::Idle
            } else {
                EnemyAiState::Detect
            }
        }
        EnemyAiState::Pursue => {
            if distance <= attack_range {
                EnemyAiState::Attack
            } else if distance > engagement_range * outer {
                EnemyAiState::Detect
            } else {
                EnemyAiState::Pursue
            }
        }
        EnemyAiState::Attack => {
            if distance > attack_range * outer {
                EnemyAiState::Pursue
            } else {
                EnemyAiState::Attack
            }
        }
    }
}

/// FixedUpdate — distance-driven AI state transition + orientation + per-state
/// movement for every `Enemy`. State transitions delegate to `next_ai_state`
/// (pure helper); orientation snap-rotates via `Transform::look_at` for any
/// non-Idle state; velocity is written directly per state per AC #8(c–e).
///
/// `Without<PlayerShip>` filter on the enemy query is REQUIRED to satisfy
/// Bevy's disjoint-Query invariant given `player_query` borrows PlayerShip's
/// Transform immutably.
pub fn apply_enemy_ai(
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
    player_query: Single<&Transform, With<PlayerShip>>,
    mut enemies: Query<
        (&mut EnemyAiState, &mut Transform, &mut LinearVelocity),
        (With<Enemy>, Without<PlayerShip>),
    >,
) {
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    let player_pos = player_query.translation;
    for (mut state, mut transform, mut velocity) in &mut enemies {
        let enemy_pos = transform.translation;
        let distance = (player_pos - enemy_pos).length();

        // (a) State transition.
        let next = next_ai_state(
            *state,
            distance,
            tuning.enemy_detection_range,
            tuning.enemy_engagement_range,
            tuning.enemy_ai_hysteresis_pct,
        );
        *state = next;

        // (b) Orientation: any non-Idle state snap-rotates to face the player.
        // Guard: skip if collocated (zero forward) or if forward is collinear with Vec3::Y
        // (player directly above/below enemy), both of which degenerate look_at.
        if next != EnemyAiState::Idle && distance > f32::EPSILON {
            let forward = (player_pos - enemy_pos).normalize();
            let up = if forward.y.abs() > 1.0 - 1e-4 {
                Vec3::Z
            } else {
                Vec3::Y
            };
            transform.look_at(player_pos, up);
        }

        // (c–e) Per-state velocity write.
        match next {
            EnemyAiState::Idle | EnemyAiState::Detect | EnemyAiState::Attack => {
                velocity.0 = Vec3::ZERO;
            }
            EnemyAiState::Pursue => {
                let direction = (player_pos - enemy_pos).normalize_or_zero();
                velocity.0 = direction * tuning.enemy_speed;
            }
        }
    }
}

/// FixedUpdate — Attack-state enemies fire a vermillion-tinted projectile at
/// `enemy_fire_rate_hz`. Cooldown ticks every frame regardless of state so
/// state changes do not let the cooldown desync.
pub fn enemy_fire_weapon(
    time: Res<Time>,
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    player: Single<&Transform, With<PlayerShip>>,
    mut enemies: Query<
        (&Transform, &EnemyAiState, &mut EnemyFireCooldown),
        (With<Enemy>, Without<PlayerShip>),
    >,
) {
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    let dt = time.delta_secs();
    let player_pos = player.translation;
    for (transform, ai_state, mut cooldown) in &mut enemies {
        cooldown.remaining = (cooldown.remaining - dt).max(0.0);
        if *ai_state != EnemyAiState::Attack || cooldown.remaining > 0.0 {
            continue;
        }
        let enemy_pos = transform.translation;
        let direction = (player_pos - enemy_pos).normalize_or_zero();
        let spawn_pos = enemy_pos + direction * ENEMY_PROJECTILE_SPAWN_OFFSET;

        let projectile_mesh = meshes.add(
            Sphere::new(ENEMY_PROJECTILE_RADIUS)
                .mesh()
                .ico(2)
                .expect("ico(2): subdivision=2 is within MAX_SUBDIVISIONS=80"),
        );
        let projectile_material = materials.add(ToonMaterial {
            tint: color_for(SemanticAccent::Enemy).into(),
            ..default()
        });

        commands.spawn((
            Projectile {
                ttl: tuning.projectile_ttl_seconds,
                damage: 1,
            },
            EnemyProjectile,
            ArenaEntity,
            Mesh3d(projectile_mesh),
            MeshMaterial3d(projectile_material),
            Transform::from_translation(spawn_pos),
            RigidBody::Dynamic,
            Collider::sphere(ENEMY_PROJECTILE_RADIUS),
            LinearVelocity(direction * tuning.projectile_speed),
            CollisionLayers::new([GameLayer::Projectile], [GameLayer::Default]),
            CollisionEventsEnabled,
        ));

        cooldown.remaining = 1.0 / tuning.enemy_fire_rate_hz.max(f32::EPSILON);

        info!(
            "enemy fired projectile from {:?} toward {:?}",
            enemy_pos, player_pos
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn det() -> f32 {
        100.0
    }
    fn eng() -> f32 {
        50.0
    }
    fn hyst() -> f32 {
        0.1
    }

    #[test]
    fn idle_when_distance_exceeds_detection_range() {
        // Far away, idle stays idle; out of detect range.
        assert_eq!(
            next_ai_state(EnemyAiState::Idle, 200.0, det(), eng(), hyst()),
            EnemyAiState::Idle
        );
    }

    #[test]
    fn idle_to_detect_when_player_enters_detection_range() {
        // Inside detection_range but outside engagement_range → Detect.
        assert_eq!(
            next_ai_state(EnemyAiState::Idle, 75.0, det(), eng(), hyst()),
            EnemyAiState::Detect
        );
    }

    #[test]
    fn detect_to_pursue_when_player_enters_engagement_range() {
        // Inside engagement_range but outside attack-range (engagement * 0.5) → Pursue.
        assert_eq!(
            next_ai_state(EnemyAiState::Detect, 35.0, det(), eng(), hyst()),
            EnemyAiState::Pursue
        );
    }

    #[test]
    fn pursue_to_attack_when_player_enters_attack_range() {
        // Inside engagement_range * 0.5 = 25.0 → Attack.
        assert_eq!(
            next_ai_state(EnemyAiState::Pursue, 20.0, det(), eng(), hyst()),
            EnemyAiState::Attack
        );
    }

    #[test]
    fn attack_holds_within_hysteresis_band_outward() {
        // Hysteresis dead-band: at distance 27.0 (between attack=25 and 25*1.1=27.5),
        // an Attack-state enemy stays in Attack — does not flicker back to Pursue.
        assert_eq!(
            next_ai_state(EnemyAiState::Attack, 27.0, det(), eng(), hyst()),
            EnemyAiState::Attack
        );
    }

    #[test]
    fn attack_to_pursue_when_distance_exceeds_outer_hysteresis_threshold() {
        // Distance 28.0 > 25*1.1 = 27.5 → Attack exits to Pursue.
        assert_eq!(
            next_ai_state(EnemyAiState::Attack, 28.0, det(), eng(), hyst()),
            EnemyAiState::Pursue
        );
    }

    #[test]
    fn detect_holds_within_hysteresis_band_outward() {
        // Distance 105.0 (between detect=100 and 100*1.1=110), Detect stays.
        assert_eq!(
            next_ai_state(EnemyAiState::Detect, 105.0, det(), eng(), hyst()),
            EnemyAiState::Detect
        );
    }
}
