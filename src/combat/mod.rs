//! CombatPlugin — owns weapon firing + projectile ballistics + projectile
//! lifecycle (FR9). Story 3.10 adds collision-driven damage events and
//! asteroid HP routing on top of the entity bundle established in 3.9.

pub mod components;
pub mod damage;
pub mod enemy;
pub mod enemy_ai;
pub mod health;
pub mod input;
pub mod projectiles;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::combat::damage::{
    AsteroidDestroyed, EnemyDestroyed, HullDepleted, ProjectileHitAsteroid, ProjectileHitEnemy,
    ProjectileHitPlayer,
};
use crate::combat::input::CombatAction;
use crate::flight::FlightSystems;
use crate::state::GameState;

pub struct CombatPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CombatSystems {
    Setup,
    EnemyAi,
    Fire,
    Lifecycle,
    EvaluateHits,
    ApplyDamage,
    CheckDeath,
}

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Setup ordering: PlayerShip must exist before combat insertion.
        // Setup lives on `OnTransition { MainMenu → Arena }` (NOT `OnEnter(Arena)`) so
        // Pause round-trip (Arena ↔ Paused) does not re-attach combat components.
        app.configure_sets(
            OnTransition {
                exited: GameState::MainMenu,
                entered: GameState::Arena,
            },
            (FlightSystems::Setup, CombatSystems::Setup).chain(),
        );
        app.add_plugins(InputManagerPlugin::<CombatAction>::default());

        // 3.10: register collision-driven damage events.
        app.add_message::<ProjectileHitAsteroid>();
        app.add_message::<AsteroidDestroyed>();
        // 4.2: register enemy-targeted damage events.
        app.add_message::<ProjectileHitEnemy>();
        app.add_message::<EnemyDestroyed>();
        // 4.3: register player-targeted damage + run-end events.
        app.add_message::<ProjectileHitPlayer>();
        app.add_message::<HullDepleted>();

        app.configure_sets(
            FixedUpdate,
            (
                CombatSystems::EnemyAi,
                CombatSystems::Fire,
                CombatSystems::Lifecycle,
                CombatSystems::EvaluateHits,
                CombatSystems::ApplyDamage,
                CombatSystems::CheckDeath,
            )
                .chain(),
        );
        app.add_systems(
            OnTransition {
                exited: GameState::MainMenu,
                entered: GameState::Arena,
            },
            (
                projectiles::attach_combat_to_player_ship,
                enemy::spawn_enemy_ship,
                damage::record_run_started_at,
            )
                .in_set(CombatSystems::Setup),
        );
        app.add_systems(
            FixedUpdate,
            (
                enemy_ai::apply_enemy_ai
                    .in_set(CombatSystems::EnemyAi)
                    .run_if(in_state(GameState::Arena)),
                projectiles::fire_primary_weapon
                    .in_set(CombatSystems::Fire)
                    .run_if(in_state(GameState::Arena)),
                enemy_ai::enemy_fire_weapon
                    .in_set(CombatSystems::Fire)
                    .run_if(in_state(GameState::Arena)),
                projectiles::tick_projectile_ttl
                    .in_set(CombatSystems::Lifecycle)
                    .run_if(in_state(GameState::Arena)),
                damage::detect_projectile_asteroid_hits
                    .in_set(CombatSystems::EvaluateHits)
                    .run_if(in_state(GameState::Arena)),
                damage::detect_projectile_enemy_hits
                    .in_set(CombatSystems::EvaluateHits)
                    .run_if(in_state(GameState::Arena)),
                damage::apply_asteroid_damage
                    .in_set(CombatSystems::ApplyDamage)
                    .run_if(in_state(GameState::Arena)),
                damage::apply_enemy_damage
                    .in_set(CombatSystems::ApplyDamage)
                    .run_if(in_state(GameState::Arena)),
                damage::detect_projectile_player_hits
                    .in_set(CombatSystems::EvaluateHits)
                    .run_if(in_state(GameState::Arena)),
                damage::apply_player_damage
                    .in_set(CombatSystems::ApplyDamage)
                    .run_if(in_state(GameState::Arena)),
                damage::check_player_death
                    .in_set(CombatSystems::CheckDeath)
                    .run_if(in_state(GameState::Arena)),
            ),
        );
    }
}
