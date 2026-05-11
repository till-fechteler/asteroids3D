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
    #[serde(default = "default_ship_thrust_newtons")]
    pub ship_thrust_newtons: f32,
    #[serde(default = "default_mouse_sensitivity")]
    pub mouse_sensitivity: f32,
    #[serde(default = "default_ship_torque_nm")]
    pub ship_torque_nm: f32,
    #[serde(default = "default_dampener_linear_strength")]
    pub dampener_linear_strength: f32,
    #[serde(default = "default_dampener_angular_strength")]
    pub dampener_angular_strength: f32,
    #[serde(default = "default_projectile_speed")]
    pub projectile_speed: f32,
    #[serde(default = "default_projectile_fire_rate_hz")]
    #[allow(
        dead_code,
        reason = "Story 3.9 baseline player-fire-rate; superseded by per-archetype weapon_*.fire_rate_hz in Story 4.4. Kept for hot-reload-during-development convenience and forward-compat with pre-4.4 tuning.ron files; a future cleanup story may remove."
    )]
    pub projectile_fire_rate_hz: f32,
    #[serde(default = "default_projectile_ttl_seconds")]
    pub projectile_ttl_seconds: f32,
    #[serde(default = "default_enemy_detection_range")]
    pub enemy_detection_range: f32,
    #[serde(default = "default_enemy_engagement_range")]
    pub enemy_engagement_range: f32,
    #[serde(default = "default_enemy_speed")]
    pub enemy_speed: f32,
    #[serde(default = "default_enemy_fire_rate_hz")]
    pub enemy_fire_rate_hz: f32,
    #[serde(default = "default_enemy_ai_hysteresis_pct")]
    pub enemy_ai_hysteresis_pct: f32,
    #[serde(default = "default_player_hull_max")]
    pub player_hull_max: u32,
    // M3 Story 4.4 — weapon archetype stats (FR10). Per-field serde defaults
    // preserve forward compat with pre-4.4 tuning.ron files.
    #[serde(default = "default_weapon_pulse")]
    pub weapon_pulse: WeaponArchetypeStats,
    #[serde(default = "default_weapon_shotgun")]
    pub weapon_shotgun: WeaponArchetypeStats,
    #[serde(default = "default_weapon_railgun")]
    pub weapon_railgun: WeaponArchetypeStats,
}

/// Per-archetype weapon stats consumed by `fire_primary_weapon` via
/// `WeaponArchetype::stats_from`. Story 4.4 ships 3 archetype stat blocks
/// (Pulse/Shotgun/Railgun) — the active archetype's stats determine
/// projectile damage, fire-rate-cooldown, muzzle speed, fan-spread, and
/// per-trigger projectile count.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub struct WeaponArchetypeStats {
    pub damage: u32,
    pub fire_rate_hz: f32,
    pub projectile_speed: f32,
    pub projectile_count: u32,
    pub spread_deg: f32,
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

fn default_mouse_sensitivity() -> f32 {
    1.0
}

fn default_ship_torque_nm() -> f32 {
    80.0
}

fn default_dampener_linear_strength() -> f32 {
    2.0
}

fn default_dampener_angular_strength() -> f32 {
    3.0
}

fn default_projectile_speed() -> f32 {
    120.0
}

fn default_projectile_fire_rate_hz() -> f32 {
    4.0
}

fn default_projectile_ttl_seconds() -> f32 {
    3.0
}

fn default_enemy_detection_range() -> f32 {
    100.0
}

fn default_enemy_engagement_range() -> f32 {
    50.0
}

fn default_enemy_speed() -> f32 {
    20.0
}

fn default_enemy_fire_rate_hz() -> f32 {
    1.0
}

fn default_enemy_ai_hysteresis_pct() -> f32 {
    0.1
}

fn default_player_hull_max() -> u32 {
    3
}

fn default_weapon_pulse() -> WeaponArchetypeStats {
    WeaponArchetypeStats {
        damage: 1,
        fire_rate_hz: 4.0,
        projectile_speed: 120.0,
        projectile_count: 1,
        spread_deg: 0.0,
    }
}

fn default_weapon_shotgun() -> WeaponArchetypeStats {
    WeaponArchetypeStats {
        damage: 1,
        fire_rate_hz: 1.5,
        projectile_speed: 80.0,
        projectile_count: 5,
        spread_deg: 15.0,
    }
}

fn default_weapon_railgun() -> WeaponArchetypeStats {
    WeaponArchetypeStats {
        damage: 5,
        fire_rate_hz: 0.5,
        projectile_speed: 300.0,
        projectile_count: 1,
        spread_deg: 0.0,
    }
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
            mouse_sensitivity: default_mouse_sensitivity(),
            ship_torque_nm: default_ship_torque_nm(),
            dampener_linear_strength: default_dampener_linear_strength(),
            dampener_angular_strength: default_dampener_angular_strength(),
            projectile_speed: default_projectile_speed(),
            projectile_fire_rate_hz: default_projectile_fire_rate_hz(),
            projectile_ttl_seconds: default_projectile_ttl_seconds(),
            enemy_detection_range: default_enemy_detection_range(),
            enemy_engagement_range: default_enemy_engagement_range(),
            enemy_speed: default_enemy_speed(),
            enemy_fire_rate_hz: default_enemy_fire_rate_hz(),
            enemy_ai_hysteresis_pct: default_enemy_ai_hysteresis_pct(),
            player_hull_max: default_player_hull_max(),
            weapon_pulse: default_weapon_pulse(),
            weapon_shotgun: default_weapon_shotgun(),
            weapon_railgun: default_weapon_railgun(),
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
        assert_eq!(cfg.mouse_sensitivity, 1.0);
        assert_eq!(cfg.ship_torque_nm, 80.0);
        assert_eq!(cfg.dampener_linear_strength, 2.0);
        assert_eq!(cfg.dampener_angular_strength, 3.0);
        assert_eq!(cfg.projectile_speed, 120.0);
        assert_eq!(cfg.projectile_fire_rate_hz, 4.0);
        assert_eq!(cfg.projectile_ttl_seconds, 3.0);
        assert_eq!(cfg.enemy_detection_range, 100.0);
        assert_eq!(cfg.enemy_engagement_range, 50.0);
        assert_eq!(cfg.enemy_speed, 20.0);
        assert_eq!(cfg.enemy_fire_rate_hz, 1.0);
        assert_eq!(cfg.enemy_ai_hysteresis_pct, 0.1);
        assert_eq!(cfg.player_hull_max, 3);
        // Story 4.4: per-archetype weapon stats (5 sub-fields × 3 archetypes).
        assert_eq!(cfg.weapon_pulse.damage, 1);
        assert_eq!(cfg.weapon_pulse.fire_rate_hz, 4.0);
        assert_eq!(cfg.weapon_pulse.projectile_speed, 120.0);
        assert_eq!(cfg.weapon_pulse.projectile_count, 1);
        assert_eq!(cfg.weapon_pulse.spread_deg, 0.0);
        assert_eq!(cfg.weapon_shotgun.damage, 1);
        assert_eq!(cfg.weapon_shotgun.fire_rate_hz, 1.5);
        assert_eq!(cfg.weapon_shotgun.projectile_speed, 80.0);
        assert_eq!(cfg.weapon_shotgun.projectile_count, 5);
        assert_eq!(cfg.weapon_shotgun.spread_deg, 15.0);
        assert_eq!(cfg.weapon_railgun.damage, 5);
        assert_eq!(cfg.weapon_railgun.fire_rate_hz, 0.5);
        assert_eq!(cfg.weapon_railgun.projectile_speed, 300.0);
        assert_eq!(cfg.weapon_railgun.projectile_count, 1);
        assert_eq!(cfg.weapon_railgun.spread_deg, 0.0);
    }

    #[test]
    fn tuning_config_deserializes_from_ron_bytes() {
        // RON parses `[T; N]` fixed-size arrays via serde's tuple deserializer → tuple syntax `(...)`.
        let bytes = b"TuningConfig(toon_steps: 5, toon_rim_power: 1.5, toon_rim_intensity: 0.4, outline_width: 5.0, outline_color: (1.0, 0.0, 0.0, 1.0), ship_thrust_newtons: 750.0, mouse_sensitivity: 0.5, ship_torque_nm: 120.0, dampener_linear_strength: 4.0, dampener_angular_strength: 6.0, projectile_speed: 200.0, projectile_fire_rate_hz: 8.0, projectile_ttl_seconds: 5.0, enemy_detection_range: 150.0, enemy_engagement_range: 75.0, enemy_speed: 30.0, enemy_fire_rate_hz: 2.0, enemy_ai_hysteresis_pct: 0.2, player_hull_max: 5, weapon_pulse: (damage: 2, fire_rate_hz: 6.0, projectile_speed: 150.0, projectile_count: 2, spread_deg: 5.0), weapon_shotgun: (damage: 3, fire_rate_hz: 2.5, projectile_speed: 100.0, projectile_count: 7, spread_deg: 20.0), weapon_railgun: (damage: 10, fire_rate_hz: 1.0, projectile_speed: 400.0, projectile_count: 1, spread_deg: 0.0))";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.toon_steps, 5);
        assert_eq!(cfg.toon_rim_power, 1.5);
        assert_eq!(cfg.toon_rim_intensity, 0.4);
        assert_eq!(cfg.outline_width, 5.0);
        assert_eq!(cfg.outline_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(cfg.ship_thrust_newtons, 750.0);
        assert_eq!(cfg.mouse_sensitivity, 0.5);
        assert_eq!(cfg.ship_torque_nm, 120.0);
        assert_eq!(cfg.dampener_linear_strength, 4.0);
        assert_eq!(cfg.dampener_angular_strength, 6.0);
        assert_eq!(cfg.projectile_speed, 200.0);
        assert_eq!(cfg.projectile_fire_rate_hz, 8.0);
        assert_eq!(cfg.projectile_ttl_seconds, 5.0);
        assert_eq!(cfg.enemy_detection_range, 150.0);
        assert_eq!(cfg.enemy_engagement_range, 75.0);
        assert_eq!(cfg.enemy_speed, 30.0);
        assert_eq!(cfg.enemy_fire_rate_hz, 2.0);
        assert_eq!(cfg.enemy_ai_hysteresis_pct, 0.2);
        assert_eq!(cfg.player_hull_max, 5);
        // Story 4.4: per-archetype round-trip with distinct values.
        assert_eq!(cfg.weapon_pulse.damage, 2);
        assert_eq!(cfg.weapon_pulse.fire_rate_hz, 6.0);
        assert_eq!(cfg.weapon_pulse.projectile_speed, 150.0);
        assert_eq!(cfg.weapon_pulse.projectile_count, 2);
        assert_eq!(cfg.weapon_pulse.spread_deg, 5.0);
        assert_eq!(cfg.weapon_shotgun.damage, 3);
        assert_eq!(cfg.weapon_shotgun.fire_rate_hz, 2.5);
        assert_eq!(cfg.weapon_shotgun.projectile_speed, 100.0);
        assert_eq!(cfg.weapon_shotgun.projectile_count, 7);
        assert_eq!(cfg.weapon_shotgun.spread_deg, 20.0);
        assert_eq!(cfg.weapon_railgun.damage, 10);
        assert_eq!(cfg.weapon_railgun.fire_rate_hz, 1.0);
        assert_eq!(cfg.weapon_railgun.projectile_speed, 400.0);
        assert_eq!(cfg.weapon_railgun.projectile_count, 1);
        assert_eq!(cfg.weapon_railgun.spread_deg, 0.0);
    }

    #[test]
    fn tuning_config_legacy_schema_uses_defaults_for_added_fields() {
        // Story 2.3 schema lacked outline + ship_thrust + rotation fields; #[serde(default = "...")] fallback fills them.
        // Forward-compat contract: future stories adding fields to TuningConfig should use the
        // same per-field serde-default pattern so older tuning.ron files keep deserializing.
        let bytes = b"TuningConfig(toon_steps: 4, toon_rim_power: 2.0, toon_rim_intensity: 0.3)";
        let cfg: TuningConfig = ron::de::from_bytes(bytes).unwrap();
        assert_eq!(cfg.outline_width, 3.0);
        assert_eq!(cfg.outline_color, [0.05, 0.05, 0.05, 1.0]);
        assert_eq!(cfg.ship_thrust_newtons, 500.0);
        assert_eq!(cfg.mouse_sensitivity, 1.0);
        assert_eq!(cfg.ship_torque_nm, 80.0);
        assert_eq!(cfg.dampener_linear_strength, 2.0);
        assert_eq!(cfg.dampener_angular_strength, 3.0);
        assert_eq!(cfg.projectile_speed, 120.0);
        assert_eq!(cfg.projectile_fire_rate_hz, 4.0);
        assert_eq!(cfg.projectile_ttl_seconds, 3.0);
        assert_eq!(cfg.enemy_detection_range, 100.0);
        assert_eq!(cfg.enemy_engagement_range, 50.0);
        assert_eq!(cfg.enemy_speed, 20.0);
        assert_eq!(cfg.enemy_fire_rate_hz, 1.0);
        assert_eq!(cfg.enemy_ai_hysteresis_pct, 0.1);
        assert_eq!(cfg.player_hull_max, 3);
        // Story 4.4: per-archetype fields default via serde-default fns when absent.
        assert_eq!(cfg.weapon_pulse, default_weapon_pulse());
        assert_eq!(cfg.weapon_shotgun, default_weapon_shotgun());
        assert_eq!(cfg.weapon_railgun, default_weapon_railgun());
    }
}
