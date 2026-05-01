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
}

pub fn default_input_map() -> InputMap<FlightAction> {
    InputMap::new([
        (FlightAction::ThrustForward, KeyCode::KeyW),
        (FlightAction::ThrustReverse, KeyCode::KeyS),
        (FlightAction::StrafeLeft, KeyCode::KeyA),
        (FlightAction::StrafeRight, KeyCode::KeyD),
        (FlightAction::ThrustUp, KeyCode::Space),
        (FlightAction::ThrustDown, KeyCode::ControlLeft),
    ])
}
