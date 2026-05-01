//! Hot-reloadable tuning resource — gameplay knobs in assets/config/tuning.ron.
//! Story 2.3 introduces three toon-shader knobs; Story 2.4 adds outline width/colour;
//! gameplay tunables (4.x onward) per architecture.md:355-359.

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use serde::Deserialize;
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
pub struct TuningConfig {
    pub toon_steps: u32,
    pub toon_rim_power: f32,
    pub toon_rim_intensity: f32,
    // M1 Story 2.4 — outline (FR49). Per-field serde defaults preserve forward compat
    // when future stories add fields without re-editing every existing tuning.ron.
    #[serde(default = "default_outline_width")]
    pub outline_width: f32,
    #[serde(default = "default_outline_color")]
    pub outline_color: [f32; 4],
    // M2 Story 3.6 — gameplay knob (FR2 6-DOF translation thrust magnitude in newtons).
    #[serde(default = "default_ship_thrust_newtons")]
    pub ship_thrust_newtons: f32,
}

fn default_outline_width() -> f32 {
    3.0
}

fn default_outline_color() -> [f32; 4] {
    [0.05, 0.05, 0.05, 1.0]
}

fn default_ship_thrust_newtons() -> f32 {
    500.0
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            toon_steps: 4,
            toon_rim_power: 2.0,
            toon_rim_intensity: 0.3,
            outline_width: default_outline_width(),
            outline_color: default_outline_color(),
            ship_thrust_newtons: default_ship_thrust_newtons(),
        }
    }
}

#[derive(Default, TypePath)]
pub struct TuningConfigLoader;

#[derive(Debug, Error)]
pub enum TuningConfigLoadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

impl AssetLoader for TuningConfigLoader {
    type Asset = TuningConfig;
    type Settings = ();
    type Error = TuningConfigLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<TuningConfig, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuning_config_default_matches_ron_initial_values() {
        let cfg = TuningConfig::default();
        assert_eq!(cfg.toon_steps, 4);
        assert_eq!(cfg.toon_rim_power, 2.0);
        assert_eq!(cfg.toon_rim_intensity, 0.3);
        assert_eq!(cfg.outline_width, 3.0);
        assert_eq!(cfg.outline_color, [0.05, 0.05, 0.05, 1.0]);
        assert_eq!(cfg.ship_thrust_newtons, 500.0);
    }

    #[test]
    fn tuning_config_deserializes_from_ron_bytes() {
        // RON parses `[T; N]` fixed-size arrays via serde's tuple deserializer → tuple syntax `(...)`.
        let bytes = b"TuningConfig(toon_steps: 5, toon_rim_power: 1.5, toon_rim_intensity: 0.4, outline_width: 5.0, outline_color: (1.0, 0.0, 0.0, 1.0), ship_thrust_newtons: 750.0)";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.toon_steps, 5);
        assert_eq!(cfg.toon_rim_power, 1.5);
        assert_eq!(cfg.toon_rim_intensity, 0.4);
        assert_eq!(cfg.outline_width, 5.0);
        assert_eq!(cfg.outline_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(cfg.ship_thrust_newtons, 750.0);
    }

    #[test]
    fn tuning_config_legacy_schema_uses_defaults_for_added_fields() {
        // Story 2.3 schema lacked outline + ship_thrust fields; #[serde(default = "...")] fallback fills them.
        // Forward-compat contract: future stories adding fields to TuningConfig should use the
        // same per-field serde-default pattern so older tuning.ron files keep deserializing.
        let bytes = b"TuningConfig(toon_steps: 4, toon_rim_power: 2.0, toon_rim_intensity: 0.3)";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.outline_width, 3.0);
        assert_eq!(cfg.outline_color, [0.05, 0.05, 0.05, 1.0]);
        assert_eq!(cfg.ship_thrust_newtons, 500.0);
    }
}
