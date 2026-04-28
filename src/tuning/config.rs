//! Hot-reloadable tuning resource — gameplay knobs in assets/config/tuning.ron.
//! Story 2.3 introduces three toon-shader knobs; future stories extend with outline (2.4) and
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
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            toon_steps: 4,
            toon_rim_power: 2.0,
            toon_rim_intensity: 0.3,
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
    }

    #[test]
    fn tuning_config_deserializes_from_ron_bytes() {
        let bytes = b"TuningConfig(toon_steps: 5, toon_rim_power: 1.5, toon_rim_intensity: 0.4)";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.toon_steps, 5);
        assert_eq!(cfg.toon_rim_power, 1.5);
        assert_eq!(cfg.toon_rim_intensity, 0.4);
    }
}
