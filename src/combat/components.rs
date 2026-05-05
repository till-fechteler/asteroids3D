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
#[allow(
    dead_code,
    reason = "Projectile.damage is read by Story 3.10's ProjectileHitAsteroid event handler — pre-wired here per the architecture-prescribed component shape"
)]
pub struct Projectile {
    pub ttl: f32,
    pub damage: u32,
}

/// Per-ship primary-weapon rate-limit state. `remaining` counts seconds
/// until the next shot is permitted. Default `0.0` so the first
/// `FirePrimary` press fires instantly.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct PrimaryWeaponCooldown {
    pub remaining: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_weapon_cooldown_default_is_zero() {
        assert_eq!(PrimaryWeaponCooldown::default().remaining, 0.0);
    }
}
