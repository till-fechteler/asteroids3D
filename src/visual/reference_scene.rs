//! Dev-only reference scene for the M1 vector-aesthetic tech spike.
//! Spawned on OnEnter(Loading); persists across state transitions for Stories 2.3+.

use bevy::prelude::*;

use super::VisualSystems;
use super::palette::{SemanticAccent, color_for};
use crate::state::GameState;

pub(super) struct ReferenceScenePlugin;

impl Plugin for ReferenceScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading),
            spawn_reference_scene.in_set(VisualSystems::Setup),
        )
        .add_systems(OnEnter(GameState::MainMenu), spawn_palette_swatches);
    }
}

#[derive(Component)]
struct ReferenceSceneEntity;

fn spawn_reference_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera3d at order: -1 so the splash Camera2d (order: 0) overlays its text on top.
    // NOTE for future MainMenu UI authors: this Camera3d persists past OnExit(Loading); any
    // new UI Camera2d (or other foreground camera) must use order >= 0 to overlay correctly.
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            ..default()
        },
        Transform::from_xyz(0.0, 1.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        ReferenceSceneEntity,
    ));

    // Asteroid placeholder (icosphere). unwrap: subdivisions=2 is well below Bevy's MAX_SUBDIVISIONS=80 cap.
    let asteroid_mesh = meshes.add(Sphere::new(1.0).mesh().ico(2).unwrap());
    let asteroid_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.50, 0.45),
        ..default()
    });
    commands.spawn((
        Mesh3d(asteroid_mesh),
        MeshMaterial3d(asteroid_mat),
        Transform::from_xyz(-2.0, 0.0, 0.0),
        ReferenceSceneEntity,
    ));

    // Ship-cockpit placeholder (cuboid).
    let ship_mesh = meshes.add(Cuboid::new(1.0, 0.5, 1.5));
    let ship_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.30, 0.55),
        ..default()
    });
    commands.spawn((
        Mesh3d(ship_mesh),
        MeshMaterial3d(ship_mat),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ReferenceSceneEntity,
    ));

    // Projectile placeholder (small UV-sphere).
    let projectile_mesh = meshes.add(Sphere::new(0.15));
    let projectile_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.85, 0.20),
        ..default()
    });
    commands.spawn((
        Mesh3d(projectile_mesh),
        MeshMaterial3d(projectile_mat),
        Transform::from_xyz(2.0, 0.0, 0.0),
        ReferenceSceneEntity,
    ));

    // 3-point lighting: key (warm white, dominant), fill (cool, soft), back/rim (warm, behind).
    commands.spawn((
        PointLight {
            intensity: 800_000.0,
            color: Color::WHITE,
            range: 50.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.0, 5.0, 4.0),
        ReferenceSceneEntity,
    ));
    commands.spawn((
        PointLight {
            intensity: 300_000.0,
            color: Color::srgb(0.85, 0.85, 1.0),
            range: 50.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.0, 2.0, 4.0),
        ReferenceSceneEntity,
    ));
    commands.spawn((
        PointLight {
            intensity: 500_000.0,
            color: Color::srgb(1.0, 0.9, 0.7),
            range: 50.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, -3.0),
        ReferenceSceneEntity,
    ));
}

fn spawn_palette_swatches(mut commands: Commands) {
    // Swatch UI camera — order: 1 puts it above the splash Camera2d (order 0, despawned by now)
    // and above the reference-scene Camera3d (order -1).
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        ReferenceSceneEntity,
    ));

    let accents = [
        SemanticAccent::Enemy,
        SemanticAccent::Salvage,
        SemanticAccent::Hazard,
        SemanticAccent::PlayerOwned,
        SemanticAccent::Neutral,
    ];

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(12.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            ReferenceSceneEntity,
        ))
        .with_children(|parent| {
            for accent in accents {
                parent
                    .spawn((
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        ReferenceSceneEntity,
                    ))
                    .with_children(|column| {
                        column.spawn((
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                ..default()
                            },
                            BackgroundColor(color_for(accent)),
                            ReferenceSceneEntity,
                        ));
                        column.spawn((
                            Text::new(label_for(accent)),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            ReferenceSceneEntity,
                        ));
                    });
            }
        });
}

fn label_for(accent: SemanticAccent) -> &'static str {
    match accent {
        SemanticAccent::Enemy => "ENEMY",
        SemanticAccent::Salvage => "SALVAGE",
        SemanticAccent::Hazard => "HAZARD",
        SemanticAccent::PlayerOwned => "PLAYER",
        SemanticAccent::Neutral => "NEUTRAL",
    }
}
