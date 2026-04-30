//! Semantic accent palette — FR50 colors with NFR-A1 colorblind distinguishability.
//! Wong (2011) "Points of view: Color blindness", Nature Methods 8(6), p.441.

use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[allow(
    dead_code,
    reason = "Neutral consumed by spawn_arena_zone; Enemy/Salvage/Hazard/PlayerOwned variants pending consumer in Story 4.5"
)]
pub enum SemanticAccent {
    Enemy,
    Salvage,
    Hazard,
    PlayerOwned,
    #[default]
    Neutral,
}

pub fn color_for(accent: SemanticAccent) -> Color {
    match accent {
        SemanticAccent::Enemy => Color::srgb_u8(0xD5, 0x5E, 0x00), // #D55E00 vermillion
        SemanticAccent::Salvage => Color::srgb_u8(0x00, 0x9E, 0x73), // #009E73 bluish-green
        SemanticAccent::Hazard => Color::srgb_u8(0xF0, 0xE4, 0x42), // #F0E442 yellow
        SemanticAccent::PlayerOwned => Color::srgb_u8(0x56, 0xB4, 0xE9), // #56B4E9 sky-blue
        SemanticAccent::Neutral => Color::srgb_u8(0x9A, 0x9A, 0x9A), // #9A9A9A neutral grey
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn color_for_enemy_is_vermillion() {
        assert_eq!(
            color_for(SemanticAccent::Enemy),
            Color::srgb_u8(0xD5, 0x5E, 0x00)
        );
    }

    #[test]
    fn color_for_salvage_is_bluish_green() {
        assert_eq!(
            color_for(SemanticAccent::Salvage),
            Color::srgb_u8(0x00, 0x9E, 0x73)
        );
    }

    #[test]
    fn color_for_hazard_is_yellow() {
        assert_eq!(
            color_for(SemanticAccent::Hazard),
            Color::srgb_u8(0xF0, 0xE4, 0x42)
        );
    }

    #[test]
    fn color_for_player_owned_is_sky_blue() {
        assert_eq!(
            color_for(SemanticAccent::PlayerOwned),
            Color::srgb_u8(0x56, 0xB4, 0xE9)
        );
    }

    #[test]
    fn color_for_neutral_matches_default() {
        assert_eq!(
            color_for(SemanticAccent::default()),
            color_for(SemanticAccent::Neutral)
        );
    }

    #[test]
    fn all_five_colors_are_unique() {
        let accents = [
            SemanticAccent::Enemy,
            SemanticAccent::Salvage,
            SemanticAccent::Hazard,
            SemanticAccent::PlayerOwned,
            SemanticAccent::Neutral,
        ];
        let rgb_set: HashSet<[u8; 3]> = accents
            .iter()
            .map(|a| {
                let srgba = color_for(*a).to_srgba();
                [
                    (srgba.red * 255.0).round() as u8,
                    (srgba.green * 255.0).round() as u8,
                    (srgba.blue * 255.0).round() as u8,
                ]
            })
            .collect();
        assert_eq!(
            rgb_set.len(),
            5,
            "all 5 SemanticAccent variants must map to distinct RGB triples"
        );
    }
}
