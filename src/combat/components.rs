//! Marker / state components for CombatPlugin entities.
//! Initial occupants are Projectile (FR9 in-flight projectile state) and
//! PrimaryWeaponCooldown (per-ship rate-limit state). Future stories add
//! HullHP / ShieldHP / Weapon archetypes per architecture.md:566.

use bevy::prelude::*;

/// In-flight projectile state. `ttl` is the remaining seconds before despawn
/// (decremented by `tick_projectile_ttl`); `damage` is the hit-point quantity
/// applied by Story 3.10's damage system. Default damage in 3.9 is 1
/// (single-hit asteroid destruction); future weapon archetypes vary it.
#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub ttl: f32,
    pub damage: u32,
}

/// Per-ship primary-weapon rate-limit state. `remaining` counts seconds
/// until the next shot is permitted (regardless of archetype — the cooldown
/// is a single shared timer, NOT per-archetype). Cycling to a different
/// archetype does NOT reset the cooldown — the player cannot dodge a slow
/// archetype's cooldown by cycling away then back. Cooldown duration on
/// each fire is computed from the active archetype's `fire_rate_hz`
/// (Story 4.4) — was `tuning.projectile_fire_rate_hz` (Story 3.9).
/// Default `0.0` so the first `FirePrimary` press fires instantly.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct PrimaryWeaponCooldown {
    pub remaining: f32,
}

/// Marker for asteroid entities. Queried by `detect_projectile_asteroid_hits`
/// to disambiguate projectile-vs-asteroid collision pairs from other pairs
/// (e.g., projectile-vs-enemy, ship-vs-asteroid). Health is on a separate
/// component to allow Stories 4.2/4.3 enemy + player to share the same
/// Health vocabulary.
#[derive(Component, Debug, Clone, Copy)]
pub struct Asteroid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_weapon_cooldown_default_is_zero() {
        assert_eq!(PrimaryWeaponCooldown::default().remaining, 0.0);
    }
}
