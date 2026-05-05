//! HUD baseline (FR24) — screen-space corner placeholders for Shields / Hull / Ammo / Salvage.
//! Renders via Bevy 0.18's default-UI-camera fallback on the cockpit Camera3d during Arena;
//! re-targets to the pause Camera2d during Paused (transparent algorithmic consequence).
//! Placeholder values are Epic-3 static; Epic 5 wires Shields/Hull, Epic 6 wires Salvage.

use bevy::prelude::*;

use crate::arena::ArenaEntity;

const SHIELDS_LABEL: &str = "SHIELDS 100";
const HULL_LABEL: &str = "HULL 100";
const AMMO_LABEL: &str = "AMMO ∞";
const SALVAGE_LABEL: &str = "SALVAGE 0";

const HUD_FONT_SIZE: f32 = 24.0;
const HUD_CORNER_MARGIN_PX: f32 = 24.0;
const HUD_TEXT_COLOR: Color = Color::srgb(0.85, 0.95, 1.0); // muted cyan-white per "scientific instrument panel" Design Philosophy

/// Marker for all HUD entities. Used both for granular HUD-only queries
/// (Epic 5/6 placeholder-value updaters will Query<&mut Text, With<HudPlaceholder>>)
/// and as a redundant safety marker — actual cleanup happens via the dual-marker
/// ArenaEntity pattern + cleanup_on_exit::<ArenaEntity> in ArenaPlugin.
#[derive(Component, Debug, Clone, Copy)]
pub struct HudEntity;

/// Identifies which tactical-state field a HUD text node represents. Wired up
/// in Story 3.11 with static placeholder values; Epic 5 connects Shields/Hull
/// to live ShieldHP/HullHP components; Epic 6 connects Salvage to the
/// SalvageCurrency resource. Ammo remains "∞" through Epic 7 (pay-to-shoot
/// economy replaces the ammo concept entirely per FR11).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudField {
    Shields,
    Hull,
    Ammo,
    Salvage,
}

/// Companion component on each HUD value-text node. Future systems mutate
/// the sibling Text component using HudField as the dispatch discriminant.
/// Story 3.11 sets these once at spawn; Epic 5/6 will add update systems
/// that re-write the Text content based on game state.
#[derive(Component, Debug, Clone, Copy)]
pub struct HudPlaceholder {
    #[allow(
        dead_code,
        reason = "HudPlaceholder.field is read by Epic 5 (Shields/Hull update systems) and Epic 6 (Salvage update system); Story 3.11 wires the placeholder slot only."
    )]
    pub field: HudField,
}

pub fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            HudEntity,
            ArenaEntity, // dual-marker — see AC #2
        ))
        .with_children(|parent| {
            // top-left: SHIELDS
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(HUD_CORNER_MARGIN_PX),
                    left: Val::Px(HUD_CORNER_MARGIN_PX),
                    ..default()
                },
                HudEntity,
                ArenaEntity,
                HudPlaceholder {
                    field: HudField::Shields,
                },
                Text::new(SHIELDS_LABEL),
                TextFont {
                    font_size: HUD_FONT_SIZE,
                    ..default()
                },
                TextColor(HUD_TEXT_COLOR),
            ));
            // top-right: HULL
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(HUD_CORNER_MARGIN_PX),
                    right: Val::Px(HUD_CORNER_MARGIN_PX),
                    ..default()
                },
                HudEntity,
                ArenaEntity,
                HudPlaceholder {
                    field: HudField::Hull,
                },
                Text::new(HULL_LABEL),
                TextFont {
                    font_size: HUD_FONT_SIZE,
                    ..default()
                },
                TextColor(HUD_TEXT_COLOR),
            ));
            // bottom-left: AMMO
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(HUD_CORNER_MARGIN_PX),
                    left: Val::Px(HUD_CORNER_MARGIN_PX),
                    ..default()
                },
                HudEntity,
                ArenaEntity,
                HudPlaceholder {
                    field: HudField::Ammo,
                },
                Text::new(AMMO_LABEL),
                TextFont {
                    font_size: HUD_FONT_SIZE,
                    ..default()
                },
                TextColor(HUD_TEXT_COLOR),
            ));
            // bottom-right: SALVAGE
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(HUD_CORNER_MARGIN_PX),
                    right: Val::Px(HUD_CORNER_MARGIN_PX),
                    ..default()
                },
                HudEntity,
                ArenaEntity,
                HudPlaceholder {
                    field: HudField::Salvage,
                },
                Text::new(SALVAGE_LABEL),
                TextFont {
                    font_size: HUD_FONT_SIZE,
                    ..default()
                },
                TextColor(HUD_TEXT_COLOR),
            ));
        });
    info!("spawned HUD with 4 corner placeholders (Shields/Hull/Ammo/Salvage)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_field_variants_are_distinct() {
        // Guards against accidental enum-variant duplication during future
        // refactors (e.g., if Epic 5 adds Hull to HudField a second time,
        // this fails before the spawn loop creates two hull labels).
        assert_ne!(HudField::Shields, HudField::Hull);
        assert_ne!(HudField::Hull, HudField::Ammo);
        assert_ne!(HudField::Ammo, HudField::Salvage);
        assert_ne!(HudField::Shields, HudField::Salvage);
    }

    #[test]
    fn hud_placeholder_carries_specified_field() {
        // Round-trip explicit-construction guard (no Default derive).
        let p = HudPlaceholder {
            field: HudField::Salvage,
        };
        assert_eq!(p.field, HudField::Salvage);
    }

    #[test]
    fn hud_font_size_smaller_than_pause_overlay() {
        // HUD is at-a-glance tactical state; pause overlay is attention-grabbing.
        // The relative sizing relationship is part of FR24 / NFR-A3 design intent
        // and a regression here would mean the HUD has been accidentally upgraded
        // to menu-grade prominence.
        const PAUSE_FONT_SIZE: f32 = 48.0; // mirror of src/pause/mod.rs:175
        const { assert!(HUD_FONT_SIZE < PAUSE_FONT_SIZE) };
    }

    #[test]
    fn hud_corner_labels_contain_expected_field_names() {
        // Lightweight contract test: the four label strings must mention the
        // four field semantic names. Catches accidental cross-wiring (e.g., a
        // refactor that swapped SHIELDS_LABEL and HULL_LABEL would still
        // compile but fail this test).
        assert!(SHIELDS_LABEL.contains("SHIELDS"));
        assert!(HULL_LABEL.contains("HULL"));
        assert!(AMMO_LABEL.contains("AMMO"));
        assert!(SALVAGE_LABEL.contains("SALVAGE"));
    }
}
