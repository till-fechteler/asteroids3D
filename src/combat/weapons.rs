//! Weapon archetype dispatch (FR10) — 3 prefab archetypes (Pulse/Shotgun/Railgun),
//! loadout component with cycle / direct-select systems, and a pure-helper
//! `spread_forwards` for the Shotgun fan distribution.
//!
//! Per-archetype stats live in `TuningConfig` (hot-reloadable) and are looked up
//! via `WeaponArchetype::stats_from`. The `fire_primary_weapon` system in
//! `src/combat/projectiles.rs` reads the active archetype from `WeaponLoadout`,
//! resolves stats, and spawns N projectiles in a fan per `spread_forwards`.
//!
//! Story 4.4 wires this for the player only — enemy weapons remain
//! single-archetype per Story 4.2. Post-MVP (C#6) crafting will replace this
//! prefab enum with a composable modules system.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::combat::input::CombatAction;
use crate::flight::PlayerShip;
use crate::tuning::config::{TuningConfig, WeaponArchetypeStats};

/// Player-equippable weapon archetype. Exhaustive match in `stats_from`
/// forces compile-time updates at every call site when a variant is added.
/// No `Default` derive — explicit construction at call sites.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponArchetype {
    Pulse,
    Shotgun,
    Railgun,
}

impl WeaponArchetype {
    /// Pure lookup: returns the canonical stats for this archetype from the
    /// tuning resource. Exhaustive match.
    pub fn stats_from(self, tuning: &TuningConfig) -> WeaponArchetypeStats {
        match self {
            WeaponArchetype::Pulse => tuning.weapon_pulse,
            WeaponArchetype::Shotgun => tuning.weapon_shotgun,
            WeaponArchetype::Railgun => tuning.weapon_railgun,
        }
    }
}

/// Player weapon loadout: up to 3 equipped slots + active-slot index.
/// `Option<WeaponArchetype>` slots allow partial loadouts (Epic 7 unlock-shop
/// forward-compat). Story 4.4 ships all 3 slots filled via `Default`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeaponLoadout {
    pub slots: [Option<WeaponArchetype>; 3],
    pub active_slot: usize,
}

impl Default for WeaponLoadout {
    fn default() -> Self {
        Self {
            slots: [
                Some(WeaponArchetype::Pulse),
                Some(WeaponArchetype::Shotgun),
                Some(WeaponArchetype::Railgun),
            ],
            active_slot: 0,
        }
    }
}

impl WeaponLoadout {
    /// Returns the currently-active archetype, or `None` if the active slot
    /// is empty or out-of-bounds (defensive — `set_active` and `cycle_next`
    /// maintain the in-bounds invariant).
    pub fn active(&self) -> Option<WeaponArchetype> {
        self.slots.get(self.active_slot).copied().flatten()
    }

    /// Cycles to the next non-empty slot (wrapping). No-op if ALL slots are
    /// empty (impossible in 4.4; defensive against Epic 7 "shop took my last
    /// weapon" edge case). At most 3 probes before bail-out.
    pub fn cycle_next(&mut self) {
        for offset in 1..=3 {
            let candidate = (self.active_slot + offset) % 3;
            if self.slots[candidate].is_some() {
                self.active_slot = candidate;
                return;
            }
        }
    }

    /// Direct-select slot N (0-indexed). No-op if N >= 3 or slot N is empty.
    pub fn set_active(&mut self, slot: usize) {
        if slot < 3 && self.slots[slot].is_some() {
            self.active_slot = slot;
        }
    }
}

/// Pure helper: distribute `count` projectile-forward unit vectors symmetrically
/// across a horizontal fan of total angular width `2 * spread_deg` around the
/// ship-forward axis, rotating around the ship-up axis.
///
/// Edge cases:
/// - `count == 0` → empty vec.
/// - `count == 1` → `[forward]` (no spread regardless of spread_deg).
/// - `up` collinear with `forward` → fallback to `Vec3::Y` as the rotation axis
///   (matches `enemy_ai.rs:159-163` look_at degenerate-up guard pattern).
///
/// For `count > 1` the angles linspace from `-spread_deg` to `+spread_deg`:
/// e.g., count=5, spread=15° → [-15, -7.5, 0, +7.5, +15]. Deterministic (no RNG).
pub fn spread_forwards(forward: Vec3, up: Vec3, count: u32, spread_deg: f32) -> Vec<Vec3> {
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![forward];
    }
    let rotation_axis = if forward
        .normalize_or_zero()
        .dot(up.normalize_or_zero())
        .abs()
        > 1.0 - 1e-4
    {
        // forward and up are collinear (ship aimed straight up/down); Vec3::Y
        // would produce zero rotation, collapsing the fan. Pick any axis
        // perpendicular to forward instead.
        let candidate = forward.cross(Vec3::X);
        if candidate.length_squared() > 1e-4 {
            candidate.normalize()
        } else {
            forward.cross(Vec3::Z).normalize()
        }
    } else {
        up.normalize_or_zero()
    };
    (0..count)
        .map(|i| {
            let t = i as f32 / (count - 1) as f32;
            let angle_deg = -spread_deg + 2.0 * spread_deg * t;
            Quat::from_axis_angle(rotation_axis, angle_deg.to_radians()) * forward
        })
        .collect()
}

/// FixedUpdate — reads `CycleWeapon` action (just_pressed) and advances the
/// loadout's active slot via `WeaponLoadout::cycle_next`. Held Tab does NOT
/// spam-cycle (just_pressed semantics).
pub fn cycle_active_weapon(
    mut ships: Query<(&mut WeaponLoadout, &ActionState<CombatAction>), With<PlayerShip>>,
) {
    for (mut loadout, action_state) in &mut ships {
        if !action_state.just_pressed(&CombatAction::CycleWeapon) {
            continue;
        }
        let prev = loadout.active_slot;
        loadout.cycle_next();
        if let Some(now_active) = loadout.active() {
            info!(
                "weapon cycle: slot {} → slot {} ({:?})",
                prev, loadout.active_slot, now_active
            );
        }
    }
}

/// FixedUpdate — reads `SelectSlot1`/`SelectSlot2`/`SelectSlot3` actions
/// (just_pressed) and calls `WeaponLoadout::set_active`. Held digits do NOT
/// spam-re-select.
pub fn select_active_weapon(
    mut ships: Query<(&mut WeaponLoadout, &ActionState<CombatAction>), With<PlayerShip>>,
) {
    const SLOT_KEYS: [(CombatAction, usize); 3] = [
        (CombatAction::SelectSlot1, 0),
        (CombatAction::SelectSlot2, 1),
        (CombatAction::SelectSlot3, 2),
    ];
    for (mut loadout, action_state) in &mut ships {
        for (action, slot) in SLOT_KEYS {
            if !action_state.just_pressed(&action) {
                continue;
            }
            let prev = loadout.active_slot;
            loadout.set_active(slot);
            if prev != loadout.active_slot
                && let Some(now_active) = loadout.active()
            {
                info!(
                    "weapon select: slot {} → slot {} ({:?})",
                    prev, loadout.active_slot, now_active
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_archetype_variants_are_distinct() {
        // Mirrors DeathCause::variants_distinct (damage.rs:429). Guards future variant additions.
        assert_ne!(WeaponArchetype::Pulse, WeaponArchetype::Shotgun);
        assert_ne!(WeaponArchetype::Shotgun, WeaponArchetype::Railgun);
        assert_ne!(WeaponArchetype::Pulse, WeaponArchetype::Railgun);
    }

    #[test]
    fn weapon_archetype_stats_from_returns_correct_archetype_data() {
        let tuning = TuningConfig::default();
        assert_eq!(WeaponArchetype::Pulse.stats_from(&tuning).damage, 1);
        assert_eq!(
            WeaponArchetype::Shotgun
                .stats_from(&tuning)
                .projectile_count,
            5
        );
        assert_eq!(WeaponArchetype::Railgun.stats_from(&tuning).damage, 5);
        assert!(
            (WeaponArchetype::Shotgun.stats_from(&tuning).spread_deg - 15.0).abs() < 1e-5,
            "shotgun spread_deg should be 15.0"
        );
    }

    #[test]
    fn weapon_loadout_default_is_three_full_slots_slot_zero_active() {
        let loadout = WeaponLoadout::default();
        assert_eq!(
            loadout.slots,
            [
                Some(WeaponArchetype::Pulse),
                Some(WeaponArchetype::Shotgun),
                Some(WeaponArchetype::Railgun)
            ]
        );
        assert_eq!(loadout.active_slot, 0);
        assert_eq!(loadout.active(), Some(WeaponArchetype::Pulse));
    }

    #[test]
    fn weapon_loadout_cycle_next_wraps_around_three_full_slots() {
        let mut loadout = WeaponLoadout::default();
        loadout.cycle_next();
        assert_eq!(loadout.active_slot, 1);
        loadout.cycle_next();
        assert_eq!(loadout.active_slot, 2);
        loadout.cycle_next();
        assert_eq!(loadout.active_slot, 0, "cycle from 2 should wrap to 0");
    }

    #[test]
    fn weapon_loadout_cycle_next_skips_empty_slots() {
        // Epic 7 unlock-shop forward-compat: slot 1 empty; cycle from slot 0 → slot 2.
        let mut loadout = WeaponLoadout {
            slots: [
                Some(WeaponArchetype::Pulse),
                None,
                Some(WeaponArchetype::Railgun),
            ],
            active_slot: 0,
        };
        loadout.cycle_next();
        assert_eq!(loadout.active_slot, 2, "cycle should skip empty slot 1");
        loadout.cycle_next();
        assert_eq!(
            loadout.active_slot, 0,
            "cycle from 2 should wrap to 0 (skipping 1)"
        );
    }

    #[test]
    fn weapon_loadout_set_active_ignores_empty_slot_and_oob() {
        let mut loadout = WeaponLoadout {
            slots: [
                Some(WeaponArchetype::Pulse),
                None,
                Some(WeaponArchetype::Railgun),
            ],
            active_slot: 0,
        };
        loadout.set_active(1); // empty slot — no-op
        assert_eq!(loadout.active_slot, 0);
        loadout.set_active(2); // populated slot — activates
        assert_eq!(loadout.active_slot, 2);
        loadout.set_active(99); // OOB — no-op
        assert_eq!(loadout.active_slot, 2);
    }

    #[test]
    fn spread_forwards_count_one_returns_single_forward() {
        let dirs = spread_forwards(Vec3::NEG_Z, Vec3::Y, 1, 30.0);
        assert_eq!(dirs.len(), 1);
        assert!((dirs[0] - Vec3::NEG_Z).length() < 1e-5);
    }

    #[test]
    fn spread_forwards_count_five_is_symmetric_around_forward() {
        // count=5, spread=15° → angles [-15, -7.5, 0, +7.5, +15]; middle is forward.
        let dirs = spread_forwards(Vec3::NEG_Z, Vec3::Y, 5, 15.0);
        assert_eq!(dirs.len(), 5);
        assert!(
            (dirs[2] - Vec3::NEG_Z).length() < 1e-5,
            "middle direction = {:?}",
            dirs[2]
        );
        let left_offset = (dirs[0] - Vec3::NEG_Z).length();
        let right_offset = (dirs[4] - Vec3::NEG_Z).length();
        assert!(
            (left_offset - right_offset).abs() < 1e-5,
            "fan asymmetric: left={} right={}",
            left_offset,
            right_offset
        );
        for (i, d) in dirs.iter().enumerate() {
            assert!(
                (d.length() - 1.0).abs() < 1e-5,
                "dirs[{}] not unit: {:?}",
                i,
                d
            );
        }
    }
}
