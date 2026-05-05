//! FlightAction enum + default keyboard bindings (FR1 keyboard input → FR2 6-DOF translation).

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum FlightAction {
    ThrustForward,
    ThrustReverse,
    StrafeLeft,
    StrafeRight,
    ThrustUp,
    ThrustDown,
    #[actionlike(Axis)]
    Pitch,
    #[actionlike(Axis)]
    Yaw,
    RollLeft,
    RollRight,
    ToggleDampener,
}

pub fn default_input_map() -> InputMap<FlightAction> {
    InputMap::new([
        (FlightAction::ThrustForward, KeyCode::KeyW),
        (FlightAction::ThrustReverse, KeyCode::KeyS),
        (FlightAction::StrafeLeft, KeyCode::KeyA),
        (FlightAction::StrafeRight, KeyCode::KeyD),
        (FlightAction::ThrustUp, KeyCode::Space),
        (FlightAction::ThrustDown, KeyCode::ControlLeft),
        (FlightAction::RollLeft, KeyCode::KeyQ),
        (FlightAction::RollRight, KeyCode::KeyE),
        (FlightAction::ToggleDampener, KeyCode::KeyX),
    ])
    .with_axis(FlightAction::Pitch, MouseMoveAxis::Y)
    .with_axis(FlightAction::Yaw, MouseMoveAxis::X)
}
