//! Flight force/torque application (FR2 6-DOF translation, FR3 3-axis rotation).
//! Reads ActionState<FlightAction>, applies ship-local force/torque via Avian's
//! Forces query (auto-cleared each FixedUpdate).

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

/// Sum of pitch/yaw/roll contributions in ship-LOCAL space.
/// Pitch (mouse Y axis) → torque around local +X (positive mouse_y → +X → nose-down).
/// Yaw (mouse X axis) → torque around local -Y (positive mouse_x → -Y → yaw-right).
/// Roll (Q/E buttons) → torque around local ±Z (RollLeft → +Z, RollRight → -Z).
/// Magnitude is NOT clamped: large mouse flicks produce proportionally large torque.
/// Returns Vec3::ZERO if no axis has a non-zero value AND neither roll button is pressed.
pub fn ship_local_torque_vector(
    action_state: &ActionState<FlightAction>,
    mouse_sensitivity: f32,
    ship_torque_nm: f32,
) -> Vec3 {
    let mut torque = Vec3::ZERO;
    let pitch = action_state.value(&FlightAction::Pitch);
    let yaw = action_state.value(&FlightAction::Yaw);
    torque.x += pitch * mouse_sensitivity;
    torque.y += -yaw * mouse_sensitivity;
    if action_state.pressed(&FlightAction::RollLeft) {
        torque.z += ship_torque_nm;
    }
    if action_state.pressed(&FlightAction::RollRight) {
        torque.z -= ship_torque_nm;
    }
    torque
}

pub fn apply_torque(
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
    mut ships: Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>,
) {
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    for (mut forces, action_state) in &mut ships {
        let local_torque = ship_local_torque_vector(
            action_state,
            tuning.mouse_sensitivity,
            tuning.ship_torque_nm,
        );
        // apply_local_torque is a no-op for Vec3::ZERO (avoids waking sleeping bodies).
        forces.apply_local_torque(local_torque);
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

    fn pressed_with_axes(
        buttons: &[FlightAction],
        axes: &[(FlightAction, f32)],
    ) -> ActionState<FlightAction> {
        let mut state = ActionState::default();
        for &b in buttons {
            state.press(&b);
        }
        for &(axis, value) in axes {
            state.set_value(&axis, value);
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

    #[test]
    fn no_input_returns_zero_torque() {
        let v = ship_local_torque_vector(&no_input(), 1.0, 80.0);
        assert_eq!(v, Vec3::ZERO);
    }

    #[test]
    fn pitch_axis_value_maps_to_local_x_torque() {
        // pitch=5.0, sensitivity=2.0 → +X torque of 10.0 (positive mouse_y → +X → nose-down).
        let state = pressed_with_axes(&[], &[(FlightAction::Pitch, 5.0)]);
        let v = ship_local_torque_vector(&state, 2.0, 80.0);
        assert!(
            (v - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn yaw_axis_value_maps_to_negative_local_y_torque() {
        // yaw=3.0, sensitivity=1.0 → -Y torque of 3.0 (positive mouse_x → -Y → yaw-right).
        let state = pressed_with_axes(&[], &[(FlightAction::Yaw, 3.0)]);
        let v = ship_local_torque_vector(&state, 1.0, 80.0);
        assert!(
            (v - Vec3::new(0.0, -3.0, 0.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn roll_left_maps_to_positive_local_z_torque() {
        // RollLeft → +Z torque of magnitude ship_torque_nm (right-hand rule + Bevy local +Z = backward).
        let state = pressed(&[FlightAction::RollLeft]);
        let v = ship_local_torque_vector(&state, 1.0, 80.0);
        assert!(
            (v - Vec3::new(0.0, 0.0, 80.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn pitch_plus_roll_right_sums_components() {
        // Pitch contributes (2, 0, 0); RollRight contributes (0, 0, -80); sum = (2, 0, -80).
        let state = pressed_with_axes(&[FlightAction::RollRight], &[(FlightAction::Pitch, 2.0)]);
        let v = ship_local_torque_vector(&state, 1.0, 80.0);
        assert!(
            (v - Vec3::new(2.0, 0.0, -80.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }
}
