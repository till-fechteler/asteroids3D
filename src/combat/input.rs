//! CombatAction enum + default mouse binding (FR9 primary fire).

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum CombatAction {
    FirePrimary,
}

pub fn default_input_map() -> InputMap<CombatAction> {
    InputMap::new([(CombatAction::FirePrimary, MouseButton::Left)])
}
