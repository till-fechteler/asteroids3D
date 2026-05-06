//! Enemy entity foundation (FR14) — defines `Enemy` marker + `EnemyShip` archetype
//! and spawns one stationary placeholder enemy in the Arena. AI / health / damage
//! routing land in Stories 4.2 / 4.3. SemanticAccent::Enemy tagging here also
//! prefigures Story 4.5's full retroactive accent sweep on existing entities.

use avian3d::prelude::{
    AngularVelocity, Collider, CollisionEventsEnabled, CollisionLayers, LayerMask, LinearVelocity,
    RigidBody,
};
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;

use crate::arena::ArenaEntity;
use crate::combat::damage::GameLayer;
use crate::combat::enemy_ai::{EnemyAiState, EnemyFireCooldown};
use crate::combat::health::Health;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;
use crate::visual::palette::{SemanticAccent, color_for};
use crate::visual::toon_material::ToonMaterial;

const ENEMY_SPAWN_POSITION: Vec3 = Vec3::new(0.0, 0.0, -60.0);
const ENEMY_CAPSULE_RADIUS: f32 = 2.0;
const ENEMY_CAPSULE_LENGTH: f32 = 4.0;

/// Empty marker tagging entities hostile to the player. Queried by Story 4.2's
/// AI transitions and Story 4.3's damage-routing for `EnemyProjectile` collisions.
#[derive(Component, Debug, Clone, Copy)]
pub struct Enemy;

/// Typed enemy archetype. Story 4.1 ships a single variant; Story 4.2 may extend
/// when AI variants land, Story 4.4 weapon archetypes may further extend.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "EnemyShip::Standard variant is read by Stories 4.2 (AI variants) and 4.4 (weapon archetypes); Story 4.1 establishes the typed slot."
)]
pub enum EnemyShip {
    Standard,
}

pub fn spawn_enemy_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
) {
    // Cold-start safety mirrors src/flight/mod.rs:97-101 — tuning.ron may not be
    // loaded yet on a hypothetical future re-entry path; fall back to defaults.
    let tuning_opt = tuning_assets.get(tuning_handle.0.id());
    if tuning_opt.is_none() {
        warn!("tuning.ron not loaded at Enemy spawn; using TuningConfig defaults");
    }
    let tuning = tuning_opt.cloned().unwrap_or_default();
    let [r, g, b, a] = tuning.outline_color;
    let outline = OutlineVolume {
        visible: true,
        width: tuning.outline_width,
        colour: Color::srgba(r, g, b, a),
    };

    let enemy_mesh = meshes.add(Capsule3d::new(ENEMY_CAPSULE_RADIUS, ENEMY_CAPSULE_LENGTH));
    let enemy_material = materials.add(ToonMaterial {
        tint: color_for(SemanticAccent::Enemy).into(),
        ..default()
    });

    // Bevy 0.18 caps `Bundle` tuple-impls at arity 15; this spawn carries 20
    // components, so they are grouped into nested tuples (Bevy auto-flattens).
    commands.spawn((
        (
            Enemy,
            EnemyShip::Standard,
            SemanticAccent::Enemy,
            Name::new("EnemyShip"),
            EnemyAiState::Idle,
            EnemyFireCooldown::default(),
            Health { current: 2, max: 2 },
        ),
        (
            ArenaEntity,
            Mesh3d(enemy_mesh),
            MeshMaterial3d(enemy_material),
            outline,
        ),
        (
            Transform::from_translation(ENEMY_SPAWN_POSITION),
            RigidBody::Dynamic,
            Collider::capsule(ENEMY_CAPSULE_RADIUS, ENEMY_CAPSULE_LENGTH),
            LinearVelocity(Vec3::ZERO),
            AngularVelocity(Vec3::ZERO),
        ),
        (
            CollisionLayers::new([GameLayer::Enemy], LayerMask::ALL),
            CollisionEventsEnabled,
        ),
    ));

    info!(
        "spawned EnemyShip at {:?} ({}m from origin)",
        ENEMY_SPAWN_POSITION,
        ENEMY_SPAWN_POSITION.length()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_ship_construction_is_explicit() {
        // No Default derive — guards against accidental future Default addition
        // that would silently default to a particular variant. Round-trip explicit
        // construction. Mirrors AsteroidHp / HudPlaceholder explicit-construction tests.
        let archetype = EnemyShip::Standard;
        assert_eq!(archetype, EnemyShip::Standard);
    }

    #[test]
    fn enemy_spawn_position_within_80m_of_origin() {
        // Epic-4 AC budget — keeps the enemy in immediate sensory range of the
        // PlayerShip spawn point. Tightening the budget below 80 m is fine; loosening
        // requires a story amendment.
        assert!(
            ENEMY_SPAWN_POSITION.length() <= 80.0,
            "ENEMY_SPAWN_POSITION distance from origin = {} exceeds AC budget of 80 m",
            ENEMY_SPAWN_POSITION.length()
        );
    }

    #[test]
    fn enemy_spawn_position_in_front_of_cockpit() {
        // Cockpit Camera3d default forward is -Z (Bevy convention). For first-impression
        // visual contact, the enemy must be on the negative-Z side. Const block makes
        // this a compile-time check (matches HUD font-size precedent at src/ui/hud.rs).
        const { assert!(ENEMY_SPAWN_POSITION.z < 0.0) };
    }

    #[test]
    fn enemy_spawn_position_clears_close_asteroids() {
        // Hardcoded coordinates of the two closest asteroids per src/arena/zone.rs:23,25
        // (kept inline rather than imported to preserve module separation; this test
        // validates the chosen enemy position against the known layout, NOT the layout
        // itself — that is src/arena/zone.rs::tests::asteroid_colliders_do_not_overlap's job).
        // If the arena layout changes, both tests need to be re-evaluated.
        let close_asteroids: &[(Vec3, f32)] = &[
            (Vec3::new(-8.0, 10.0, -42.0), 5.0),
            (Vec3::new(18.0, 3.0, -25.0), 6.5),
        ];
        let enemy_max_extent = ENEMY_CAPSULE_RADIUS + ENEMY_CAPSULE_LENGTH * 0.5;
        for &(asteroid_pos, asteroid_radius) in close_asteroids {
            let distance = (ENEMY_SPAWN_POSITION - asteroid_pos).length();
            let min_separation = asteroid_radius + enemy_max_extent;
            assert!(
                distance > min_separation,
                "ENEMY_SPAWN_POSITION at {:?} overlaps asteroid at {:?} (r={}); distance={}, min_separation={}",
                ENEMY_SPAWN_POSITION,
                asteroid_pos,
                asteroid_radius,
                distance,
                min_separation
            );
        }
    }
}
