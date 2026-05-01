//! 6-DOF translation thrust system (FR2). Reads ActionState<FlightAction>, applies
//! ship-local force via Avian's Forces query (auto-cleared each FixedUpdate).

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::flight::PlayerShip;
use crate::flight::input::FlightAction;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;

/// Sum of pressed-action axes in ship-LOCAL space. Magnitude is NOT clamped:
/// pressing W+D returns √2 magnitude per epic-3 spec ("forces sum"). Returns
/// Vec3::ZERO when no flight-translation action is pressed.
///
/// Bevy convention: forward = -Z, right = +X, up = +Y in entity-local space.
pub fn ship_local_thrust_vector(action_state: &ActionState<FlightAction>) -> Vec3 {
    let mut force = Vec3::ZERO;
    if action_state.pressed(&FlightAction::ThrustForward) {
        force += Vec3::NEG_Z;
    }
    if action_state.pressed(&FlightAction::ThrustReverse) {
        force += Vec3::Z;
    }
    if action_state.pressed(&FlightAction::StrafeLeft) {
        force += Vec3::NEG_X;
    }
    if action_state.pressed(&FlightAction::StrafeRight) {
        force += Vec3::X;
    }
    if action_state.pressed(&FlightAction::ThrustUp) {
        force += Vec3::Y;
    }
    if action_state.pressed(&FlightAction::ThrustDown) {
        force += Vec3::NEG_Y;
    }
    force
}

pub fn apply_thrust(
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
    mut ships: Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>,
) {
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    for (mut forces, action_state) in &mut ships {
        let local_force = ship_local_thrust_vector(action_state) * tuning.ship_thrust_newtons;
        // apply_local_force is a no-op for Vec3::ZERO (avoids waking sleeping bodies).
        forces.apply_local_force(local_force);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_input() -> ActionState<FlightAction> {
        ActionState::default()
    }

    fn pressed(actions: &[FlightAction]) -> ActionState<FlightAction> {
        let mut state = ActionState::default();
        for &a in actions {
            state.press(&a);
        }
        state
    }

    #[test]
    fn no_action_returns_zero_vector() {
        let v = ship_local_thrust_vector(&no_input());
        assert_eq!(v, Vec3::ZERO);
    }

    #[test]
    fn forward_only_returns_neg_z_in_local_space() {
        // Bevy convention: forward = -Z in local space.
        let v = ship_local_thrust_vector(&pressed(&[FlightAction::ThrustForward]));
        assert!(
            (v - Vec3::NEG_Z).length() < 1e-5,
            "expected -Z, got {:?}",
            v
        );
    }

    #[test]
    fn forward_plus_right_sums_with_unclamped_magnitude() {
        // Forward (-Z) + Right (+X) = (1, 0, -1); magnitude is √2 (deliberately unclamped per epic).
        let v = ship_local_thrust_vector(&pressed(&[
            FlightAction::ThrustForward,
            FlightAction::StrafeRight,
        ]));
        assert!(
            (v - Vec3::new(1.0, 0.0, -1.0)).length() < 1e-5,
            "got {:?}",
            v
        );
        assert!((v.length() - std::f32::consts::SQRT_2).abs() < 1e-5);
    }
}
