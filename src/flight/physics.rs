//! Flight force/torque/acceleration application (FR2 6-DOF translation, FR3 3-axis
//! rotation, FR5 inertial dampener). Reads ActionState<FlightAction>, applies
//! ship-local force/torque/acceleration via Avian's Forces query (auto-cleared each FixedUpdate).

use avian3d::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::flight::PlayerShip;
use crate::flight::components::DampenerState;
use crate::flight::input::FlightAction;
use crate::tuning::TuningHandle;
use crate::tuning::config::TuningConfig;

/// Per-Update mouse-motion accumulator drained by FixedUpdate's apply_torque.
/// Decouples per-render-frame mouse delta from fixed-step physics integration:
/// total angular impulse over 1 s is independent of render framerate.
#[derive(Resource, Default)]
pub struct MouseLookDelta(pub Vec2);

/// PreUpdate ticks left during which accumulate_mouse_look drops incoming mouse
/// delta. Set on cursor grab so the OS cursor-warp motion (which arrives 1–2
/// frames after the grab is requested) does not become a torque spike.
#[derive(Resource, Default)]
pub struct MouseLookSuppressFrames(pub u8);

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
/// `mouse_pitch` (mouse Y delta) → torque around local +X (positive mouse_y → +X → nose-down).
/// `mouse_yaw` (mouse X delta) → torque around local -Y (positive mouse_x → -Y → yaw-right).
/// Roll (Q/E buttons) → torque around local ±Z (RollLeft → +Z, RollRight → -Z).
/// Magnitude is NOT clamped: large mouse flicks produce proportionally large torque.
/// Returns Vec3::ZERO if mouse_pitch + mouse_yaw are both 0 AND neither roll button is pressed.
pub fn ship_local_torque_vector(
    action_state: &ActionState<FlightAction>,
    mouse_pitch: f32,
    mouse_yaw: f32,
    mouse_sensitivity: f32,
    ship_torque_nm: f32,
) -> Vec3 {
    let mut torque = Vec3::ZERO;
    torque.x += mouse_pitch * mouse_sensitivity;
    torque.y += -mouse_yaw * mouse_sensitivity;
    if action_state.pressed(&FlightAction::RollLeft) {
        torque.z += ship_torque_nm;
    }
    if action_state.pressed(&FlightAction::RollRight) {
        torque.z -= ship_torque_nm;
    }
    torque
}

/// Reads `AccumulatedMouseMotion::delta` (per-render-frame, reset by Bevy each
/// frame) and adds it to the FixedUpdate-drained `MouseLookDelta` buffer.
/// Skips while `MouseLookSuppressFrames > 0` so cursor-warp motion after a
/// grab does not bleed into the buffer.
pub fn accumulate_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut buffer: ResMut<MouseLookDelta>,
    mut suppress: ResMut<MouseLookSuppressFrames>,
) {
    if suppress.0 > 0 {
        suppress.0 -= 1;
        return;
    }
    buffer.0 += mouse_motion.delta;
}

pub fn apply_torque(
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
    mut mouse_buffer: ResMut<MouseLookDelta>,
    mut ships: Query<(Forces, &ActionState<FlightAction>), With<PlayerShip>>,
) {
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    // Drain buffered mouse motion: subsequent FixedUpdate catch-up ticks in the
    // same frame see zero, so total angular impulse per render-frame's worth of
    // motion is independent of fixed-tick rate.
    let mouse = mouse_buffer.0;
    mouse_buffer.0 = Vec2::ZERO;
    for (mut forces, action_state) in &mut ships {
        let local_torque = ship_local_torque_vector(
            action_state,
            mouse.y,
            mouse.x,
            tuning.mouse_sensitivity,
            tuning.ship_torque_nm,
        );
        // apply_local_torque is a no-op for Vec3::ZERO (avoids waking sleeping bodies).
        forces.apply_local_torque(local_torque);
    }
}

/// Linear and angular acceleration to bleed velocity toward zero when
/// `state.active`. Returns (Vec3::ZERO, Vec3::ZERO) when inactive — the
/// early return covers the dampener-off case before any arithmetic.
/// Linear strength scales linear-velocity-opposing acceleration; angular
/// strength scales angular-velocity-opposing acceleration. Contributions
/// are independent (linear vs. angular axes do not couple).
pub fn dampener_acceleration(
    state: DampenerState,
    linear_velocity: Vec3,
    angular_velocity: Vec3,
    linear_strength: f32,
    angular_strength: f32,
) -> (Vec3, Vec3) {
    if !state.active {
        return (Vec3::ZERO, Vec3::ZERO);
    }
    (
        -linear_velocity * linear_strength,
        -angular_velocity * angular_strength,
    )
}

pub fn apply_dampener(
    tuning_assets: Res<Assets<TuningConfig>>,
    tuning_handle: Res<TuningHandle>,
    mut ships: Query<(Forces, &DampenerState), With<PlayerShip>>,
) {
    // PATTERN DEVIATION: Avian's apply_*_acceleration bypasses the mass/inertia
    // divisor; mathematically equivalent to applying force = -velocity * strength * mass
    // per the AC, but skips a redundant query of ComputedMass / ComputedAngularInertia.
    // apply_*_acceleration operate in world-space, matching forces.linear/angular_velocity()
    // — no local-frame transform needed (contrast: apply_thrust uses apply_local_force).
    let tuning = tuning_assets
        .get(tuning_handle.0.id())
        .cloned()
        .unwrap_or_default();
    for (mut forces, state) in &mut ships {
        let (linear_accel, angular_accel) = dampener_acceleration(
            *state,
            forces.linear_velocity(),
            forces.angular_velocity(),
            tuning.dampener_linear_strength,
            tuning.dampener_angular_strength,
        );
        // apply_*_acceleration are no-ops for Vec3::ZERO (avoids waking sleeping bodies).
        forces.apply_linear_acceleration(linear_accel);
        forces.apply_angular_acceleration(angular_accel);
    }
}

pub fn toggle_dampener(
    mut ships: Query<(&ActionState<FlightAction>, &mut DampenerState), With<PlayerShip>>,
) {
    for (action_state, mut dampener) in &mut ships {
        if action_state.just_pressed(&FlightAction::ToggleDampener) {
            dampener.active = !dampener.active;
            info!(
                "dampener {}",
                if dampener.active {
                    "engaged"
                } else {
                    "disengaged"
                }
            );
        }
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

    #[test]
    fn no_input_returns_zero_torque() {
        let v = ship_local_torque_vector(&no_input(), 0.0, 0.0, 1.0, 80.0);
        assert_eq!(v, Vec3::ZERO);
    }

    #[test]
    fn pitch_axis_value_maps_to_local_x_torque() {
        // pitch=5.0, sensitivity=2.0 → +X torque of 10.0 (positive mouse_y → +X → nose-down).
        let v = ship_local_torque_vector(&no_input(), 5.0, 0.0, 2.0, 80.0);
        assert!(
            (v - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn yaw_axis_value_maps_to_negative_local_y_torque() {
        // yaw=3.0, sensitivity=1.0 → -Y torque of 3.0 (positive mouse_x → -Y → yaw-right).
        let v = ship_local_torque_vector(&no_input(), 0.0, 3.0, 1.0, 80.0);
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
        let v = ship_local_torque_vector(&state, 0.0, 0.0, 1.0, 80.0);
        assert!(
            (v - Vec3::new(0.0, 0.0, 80.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn pitch_plus_roll_right_sums_components() {
        // Pitch contributes (2, 0, 0); RollRight contributes (0, 0, -80); sum = (2, 0, -80).
        let state = pressed(&[FlightAction::RollRight]);
        let v = ship_local_torque_vector(&state, 2.0, 0.0, 1.0, 80.0);
        assert!(
            (v - Vec3::new(2.0, 0.0, -80.0)).length() < 1e-5,
            "got {:?}",
            v
        );
    }

    #[test]
    fn roll_left_plus_roll_right_cancels_to_zero() {
        // Q+E held simultaneously: +Z and -Z roll contributions cancel exactly.
        // Behavioural note: the helper does not pick a "winner"; chord-input → no roll.
        let state = pressed(&[FlightAction::RollLeft, FlightAction::RollRight]);
        let v = ship_local_torque_vector(&state, 0.0, 0.0, 1.0, 80.0);
        assert_eq!(v, Vec3::ZERO);
    }

    #[test]
    fn dampener_inactive_returns_zero_acceleration() {
        // Inactive dampener with non-zero velocities returns zero — verifies the early-return gate.
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: false },
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
            2.0,
            3.0,
        );
        assert_eq!(lin, Vec3::ZERO);
        assert_eq!(ang, Vec3::ZERO);
    }

    #[test]
    fn dampener_active_zero_velocity_returns_zero_acceleration() {
        // Active dampener with zero velocities returns zero — verifies the no-op-quiet case.
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::ZERO,
            Vec3::ZERO,
            2.0,
            3.0,
        );
        assert_eq!(lin, Vec3::ZERO);
        assert_eq!(ang, Vec3::ZERO);
    }

    #[test]
    fn dampener_active_linear_velocity_returns_negative_proportional_acceleration() {
        // lin=(2,0,0), strength=2.0 → linear accel of (-4,0,0); angular zero (no coupling).
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::ZERO,
            2.0,
            3.0,
        );
        assert!(
            (lin - Vec3::new(-4.0, 0.0, 0.0)).length() < 1e-5,
            "got {:?}",
            lin
        );
        assert_eq!(ang, Vec3::ZERO);
    }

    #[test]
    fn dampener_active_angular_velocity_returns_negative_proportional_acceleration() {
        // ang=(0,3,0), strength=3.0 → angular accel of (0,-9,0); linear zero (no coupling).
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::ZERO,
            Vec3::new(0.0, 3.0, 0.0),
            2.0,
            3.0,
        );
        assert_eq!(lin, Vec3::ZERO);
        assert!(
            (ang - Vec3::new(0.0, -9.0, 0.0)).length() < 1e-5,
            "got {:?}",
            ang
        );
    }

    #[test]
    fn dampener_combines_linear_and_angular_independently() {
        // Both axes non-zero: linear strength scales linear only, angular strength scales angular only.
        let (lin, ang) = dampener_acceleration(
            DampenerState { active: true },
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            2.0,
            3.0,
        );
        assert!(
            (lin - Vec3::new(-2.0, -4.0, -6.0)).length() < 1e-5,
            "got {:?}",
            lin
        );
        assert!(
            (ang - Vec3::new(-12.0, -15.0, -18.0)).length() < 1e-5,
            "got {:?}",
            ang
        );
    }
}
