//! Hand-designed Arena zone — static asteroid field + key light + stand-in camera.
//! Spawns on OnEnter(Arena); cleanup is owned by ArenaPlugin via cleanup_on_exit::<ArenaEntity>.

use avian3d::prelude::{Collider, CollisionEventsEnabled, CollisionLayers, LayerMask, RigidBody};
use bevy::prelude::*;
use bevy_mod_outline::OutlineVolume;

use super::ArenaEntity;
use crate::combat::components::AsteroidHp;
use crate::combat::damage::GameLayer;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;
use crate::visual::palette::{SemanticAccent, color_for};
use crate::visual::toon_material::ToonMaterial;

/// Hand-picked (position, radius_m) layout — 17 asteroids spanning ~185×110×185 m
/// with every center within ±100 m on each axis. Origin and the +Z corridor toward
/// the spawn camera are kept clear so Story 3.5's PlayerShip at origin retains
/// line-of-sight to ≥3 asteroids within 50 m.
const ASTEROIDS: &[(Vec3, f32)] = &[
    // Close cluster (radial ≤ 50 m of origin) — 5 asteroids
    (Vec3::new(18.0, 3.0, -25.0), 6.5),
    (Vec3::new(-22.0, -4.0, -38.0), 4.5),
    (Vec3::new(-8.0, 10.0, -42.0), 5.0),
    (Vec3::new(30.0, -8.0, -18.0), 3.5),
    (Vec3::new(-5.0, -3.0, 35.0), 4.0),
    // Mid-range (radial 50–100 m) — 6 asteroids
    (Vec3::new(60.0, 20.0, -50.0), 9.0),
    (Vec3::new(-55.0, -15.0, -75.0), 7.5),
    (Vec3::new(45.0, -25.0, -90.0), 6.0),
    (Vec3::new(-70.0, 10.0, -25.0), 8.0),
    (Vec3::new(85.0, 5.0, 40.0), 5.5),
    (Vec3::new(-50.0, 35.0, 55.0), 7.0),
    // Far field (radial ~100–135 m, every axis ≤ 100 m) — 6 asteroids
    (Vec3::new(95.0, -45.0, -75.0), 11.0),
    (Vec3::new(-85.0, 40.0, -95.0), 10.5),
    (Vec3::new(20.0, 55.0, -90.0), 12.0),
    (Vec3::new(-30.0, -55.0, -95.0), 9.5),
    (Vec3::new(75.0, 25.0, 90.0), 8.5),
    (Vec3::new(-90.0, -10.0, 85.0), 10.0),
];

pub fn spawn_arena_zone(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
) {
    // Cold-start safety: tuning.ron is loaded in Startup; if a future re-entry path
    // races OnEnter(Arena) ahead of the asset arriving, fall back to defaults.
    let tuning_opt = tuning_assets.get(tuning_handle.0.id());
    if tuning_opt.is_none() {
        warn!("tuning.ron not loaded at Arena entry; using TuningConfig defaults");
    }
    let tuning = tuning_opt.cloned().unwrap_or_default();
    let outline_volume = || {
        let [r, g, b, a] = tuning.outline_color;
        OutlineVolume {
            visible: true,
            width: tuning.outline_width,
            colour: Color::srgba(r, g, b, a),
        }
    };

    // Key light — non-axis-aligned for legible posterization on multi-facet asteroids.
    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-0.3, -1.0, -0.4).normalize(), Vec3::Y),
        ArenaEntity,
    ));

    // Asteroid field. Visual radius == physics radius for trustworthy 3.10 collisions.
    let neutral_tint = color_for(SemanticAccent::Neutral).into();
    for &(position, radius) in ASTEROIDS {
        let mesh = meshes.add(
            Sphere::new(radius)
                .mesh()
                .ico(2)
                .expect("ico(2): subdivision=2 is within MAX_SUBDIVISIONS=80"),
        );
        let material = materials.add(ToonMaterial {
            tint: neutral_tint,
            ..default()
        });
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position),
            SemanticAccent::Neutral,
            RigidBody::Static,
            Collider::sphere(radius),
            outline_volume(),
            AsteroidHp { current: 1 },
            CollisionLayers::new([GameLayer::Asteroid], LayerMask::ALL),
            CollisionEventsEnabled,
            ArenaEntity,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asteroid_count_in_acceptance_range() {
        assert!(
            (15..=25).contains(&ASTEROIDS.len()),
            "AC #2: expected 15–25 asteroids; got {}",
            ASTEROIDS.len()
        );
    }

    #[test]
    fn asteroid_radii_within_3_to_12() {
        for &(pos, r) in ASTEROIDS {
            assert!(
                (3.0..=12.0).contains(&r),
                "AC #2: radius {} at {:?} outside [3.0, 12.0]",
                r,
                pos
            );
        }
    }

    #[test]
    fn asteroid_positions_within_volume() {
        for &(pos, _) in ASTEROIDS {
            assert!(
                pos.x.abs() <= 100.0 && pos.y.abs() <= 100.0 && pos.z.abs() <= 100.0,
                "AC #2: position {:?} outside ±100 m volume",
                pos
            );
        }
    }

    #[test]
    fn asteroid_colliders_do_not_overlap() {
        for (i, &(p1, r1)) in ASTEROIDS.iter().enumerate() {
            for &(p2, r2) in ASTEROIDS.iter().skip(i + 1) {
                let distance = (p1 - p2).length();
                let min_separation = r1 + r2;
                assert!(
                    distance >= min_separation,
                    "asteroids at {:?} (r={}) and {:?} (r={}) overlap (distance={}, min={})",
                    p1,
                    r1,
                    p2,
                    r2,
                    distance,
                    min_separation
                );
            }
        }
    }

    #[test]
    fn at_least_three_asteroids_within_50m_of_origin() {
        let count = ASTEROIDS.iter().filter(|(p, _)| p.length() <= 50.0).count();
        assert!(
            count >= 3,
            "Story 3.5 line-of-sight precondition: expected ≥3 asteroids within 50 m; got {}",
            count
        );
    }
}
