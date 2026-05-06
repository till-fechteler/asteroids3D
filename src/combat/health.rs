//! Generic hit-point pool component shared by asteroids (Story 4.2 refactor
//! from `AsteroidHp`), enemies (Story 4.2), and PlayerShip (Story 4.3).
//! Epic 5's Story 5.1 splits this into formal HullHP / ShieldHP components.

use bevy::prelude::*;

/// Generic hit-point pool with maximum capacity. Used by asteroids
/// (Story 4.2 refactor from `AsteroidHp`), enemies (Story 4.2), and
/// PlayerShip (Story 4.3). Epic 5's Story 5.1 splits this into formal
/// HullHP / ShieldHP components.
///
/// NO Default derive — callers always specify both fields explicitly. A
/// silent default of (0, 0) would mean "pre-destroyed / unkillable" footgun.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_construction_is_explicit() {
        // No Default derive — guards against accidental future Default addition
        // that would silently default to (current=0, max=0) (pre-destroyed/unkillable footgun).
        // Two-field round-trip per the AsteroidHp / EnemyShip / HudPlaceholder explicit-construction precedent.
        let h = Health { current: 2, max: 2 };
        assert_eq!(h.current, 2);
        assert_eq!(h.max, 2);
    }
}
