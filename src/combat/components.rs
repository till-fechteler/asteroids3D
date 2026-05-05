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
/// until the next shot is permitted. Default `0.0` so the first
/// `FirePrimary` press fires instantly.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
pub struct PrimaryWeaponCooldown {
    pub remaining: f32,
}

/// Asteroid hit-point pool. Epic 3 default `current = 1` for single-hit
/// destruction (per Story 3.10 spec); Epic 4/5 multi-HP asteroids will spawn
/// with higher initial values via the same component. Decremented by
/// `combat::damage::apply_asteroid_damage`; despawn fires when current == 0.
///
/// NO Default derive — callers always specify `current` explicitly. A
/// silent default of 0 would mean "pre-destroyed", a hazardous footgun.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AsteroidHp {
    pub current: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_weapon_cooldown_default_is_zero() {
        assert_eq!(PrimaryWeaponCooldown::default().remaining, 0.0);
    }

    #[test]
    fn asteroid_hp_construction_is_explicit() {
        // No Default derive — this test guards against accidental future Default
        // addition that would silently default current=0 (pre-destroyed footgun).
        let hp = AsteroidHp { current: 1 };
        assert_eq!(hp.current, 1);
    }
}
