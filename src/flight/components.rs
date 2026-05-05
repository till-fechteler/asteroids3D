//! Marker / state components for FlightPlugin entities.
//! Initial occupant is DampenerState (FR5); future stories add Boost (FR6),
//! Thrusters (FR2 visual marker), and TractorEmitter (FR7) per architecture.md:560.

use bevy::prelude::*;

/// Toggleable inertial-dampener state on the PlayerShip entity. When `active`,
/// `apply_dampener` (in `flight/physics.rs`) bleeds linear + angular velocity
/// toward zero each FixedUpdate tick. Default `active = true` per Epic 3 spec.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DampenerState {
    pub active: bool,
}

impl Default for DampenerState {
    fn default() -> Self {
        Self { active: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dampener_state_default_is_active() {
        assert!(DampenerState::default().active);
    }
}
