//! CombatAction enum + default bindings.
//! Story 3.9: FirePrimary (LMB). Story 4.4: CycleWeapon (Tab) +
//! SelectSlot1/2/3 (Digits 1/2/3) for weapon-archetype switching (FR10).
//!
//! Tab is chosen for CycleWeapon (NOT Q per epic-4.4 spec line 132) because
//! Q is already bound to `FlightAction::RollLeft` (`flight/input.rs:31`);
//! reusing Q would fire both actions every press.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum CombatAction {
    FirePrimary,
    CycleWeapon,
    SelectSlot1,
    SelectSlot2,
    SelectSlot3,
}

pub fn default_input_map() -> InputMap<CombatAction> {
    InputMap::default()
        .with(CombatAction::FirePrimary, MouseButton::Left)
        .with(CombatAction::CycleWeapon, KeyCode::Tab)
        .with(CombatAction::SelectSlot1, KeyCode::Digit1)
        .with(CombatAction::SelectSlot2, KeyCode::Digit2)
        .with(CombatAction::SelectSlot3, KeyCode::Digit3)
}
