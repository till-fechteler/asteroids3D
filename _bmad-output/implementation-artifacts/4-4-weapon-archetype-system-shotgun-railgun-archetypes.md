# Story 4.4: Weapon Archetype System + Shotgun / Railgun Archetypes

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want three distinct weapon archetypes equipped in slots I can cycle between or direct-select,
So that I have tactical weapon choices per FR10 — the fourth of Epic 4 / M3 Itch.io stop-and-ship combat-loop stories (4.1 entity foundation → 4.2 AI alive → 4.3 hull + permadeath → **4.4 weapon variety**).

## Acceptance Criteria

1. **(`WeaponArchetypeStats` struct + per-archetype tuning fields — new types in `src/tuning/config.rs`)** A new `WeaponArchetypeStats` struct is added with Serde derives, holding the 5 per-archetype stats:
   ```rust
   #[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
   pub struct WeaponArchetypeStats {
       pub damage: u32,
       pub fire_rate_hz: f32,
       pub projectile_speed: f32,
       pub projectile_count: u32,
       pub spread_deg: f32,
   }
   ```
   **And** three `TuningConfig` fields are added with the established `#[serde(default = "...")]` per-field-default forward-compat pattern:
   ```rust
   #[serde(default = "default_weapon_pulse")]
   pub weapon_pulse: WeaponArchetypeStats,
   #[serde(default = "default_weapon_shotgun")]
   pub weapon_shotgun: WeaponArchetypeStats,
   #[serde(default = "default_weapon_railgun")]
   pub weapon_railgun: WeaponArchetypeStats,
   ```
   **And** three default-fn returns match the epic-4.4 spec literally:
   ```rust
   fn default_weapon_pulse() -> WeaponArchetypeStats {
       WeaponArchetypeStats { damage: 1, fire_rate_hz: 4.0, projectile_speed: 120.0, projectile_count: 1, spread_deg: 0.0 }
   }
   fn default_weapon_shotgun() -> WeaponArchetypeStats {
       WeaponArchetypeStats { damage: 1, fire_rate_hz: 1.5, projectile_speed: 80.0, projectile_count: 5, spread_deg: 15.0 }
   }
   fn default_weapon_railgun() -> WeaponArchetypeStats {
       WeaponArchetypeStats { damage: 5, fire_rate_hz: 0.5, projectile_speed: 300.0, projectile_count: 1, spread_deg: 0.0 }
   }
   ```
   **And** `TuningConfig::default()` is extended with `weapon_pulse: default_weapon_pulse(), weapon_shotgun: default_weapon_shotgun(), weapon_railgun: default_weapon_railgun()`.
   **And** `assets/config/tuning.ron` gets three new nested fields at the canonical default values:
   ```ron
   weapon_pulse:   (damage: 1, fire_rate_hz: 4.0, projectile_speed: 120.0, projectile_count: 1, spread_deg: 0.0),
   weapon_shotgun: (damage: 1, fire_rate_hz: 1.5, projectile_speed: 80.0, projectile_count: 5, spread_deg: 15.0),
   weapon_railgun: (damage: 5, fire_rate_hz: 0.5, projectile_speed: 300.0, projectile_count: 1, spread_deg: 0.0),
   ```
   **And** the existing 3 `tuning::config::tests` round-trip / RON-bytes / legacy-schema tests are extended with assertions for all three weapon fields (default-matches-RON test asserts all 5 sub-fields per archetype; RON-bytes test uses a distinct round-trip value per archetype; legacy-schema test asserts the per-field-default fallback). **NO** new test fns — assertion count grows by `3 archetypes × 5 sub-fields × 3 test fns = 45` across the 3 existing test fns.
   **And** the pre-existing `projectile_speed`, `projectile_fire_rate_hz`, `projectile_ttl_seconds` fields are **PRESERVED**: `projectile_speed` is still read by `enemy_fire_weapon` (`enemy_ai.rs:236`); `projectile_ttl_seconds` is still read by both player+enemy fire systems for projectile TTL. The Pulse archetype's `projectile_speed: 120.0` intentionally duplicates `tuning.projectile_speed: 120.0` for now — Pulse is the existing weapon, and the per-archetype tuning becomes the single source of truth for the player. `tuning.projectile_fire_rate_hz` becomes unused by the player path after this story; **NO** removal in 4.4 (still listed in `tuning.ron` for forward-compat / hot-reload-during-development convenience; flagged in Dev Notes as removable in a future cleanup).

2. **(`WeaponArchetype` enum + pure helper — new types in `src/combat/weapons.rs`)** A new file `src/combat/weapons.rs` is authored (NOT in `combat/components.rs` — the file is referenced by name in architecture.md:567 "`weapons.rs`: 3 prefab weapon archetypes, firing systems"). Contents:
   ```rust
   /// Player-equippable weapon archetype. Stats per variant are loaded from
   /// `TuningConfig` via `stats_from`. Variants are exhaustive — `Pulse` is the
   /// 3.9 baseline behavior; `Shotgun` and `Railgun` are new in 4.4. Future
   /// post-MVP crafting (C#6) will replace this prefab enum with a composable
   /// modules system (per PRD 173).
   #[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
   pub enum WeaponArchetype {
       Pulse,
       Shotgun,
       Railgun,
   }

   impl WeaponArchetype {
       /// Pure lookup: returns the canonical stats for this archetype from the
       /// tuning resource. The match is exhaustive — adding a variant forces
       /// the call sites to be updated (compile-time forward-compat).
       pub fn stats_from(self, tuning: &TuningConfig) -> WeaponArchetypeStats {
           match self {
               WeaponArchetype::Pulse => tuning.weapon_pulse,
               WeaponArchetype::Shotgun => tuning.weapon_shotgun,
               WeaponArchetype::Railgun => tuning.weapon_railgun,
           }
       }
   }
   ```
   **And** `WeaponArchetype` is a `Component` (the loadout stores it directly inside slots — see AC #3) AND a value type (used by `stats_from` and the cycle/select logic). No separate marker component.
   **And** **NO** `Default` derive on `WeaponArchetype` — a silent default would mask which archetype was selected (e.g., a fresh `WeaponArchetype::default()` would be Pulse implicitly, which is correct today but masks future variant additions). Loadout slots are explicitly constructed; see AC #3.
   **And** the `#[allow(dead_code, reason = "...")]` block on `EnemyShip::Standard` at `src/combat/enemy.rs:33-40` mentions "Story 4.4 weapon archetypes may further extend" — Story 4.4 does **NOT** extend `EnemyShip` (enemies keep their single weapon profile per Story 4.2; weapon archetypes are player-side only). The `#[allow]` block on `EnemyShip` is left unchanged; the comment text becomes stale but rewriting it is out-of-scope (it's a forward-compat note, not a load-bearing claim).

3. **(`WeaponLoadout` component + Default impl — new type in `src/combat/weapons.rs`)** A new `WeaponLoadout` component is added alongside `WeaponArchetype`:
   ```rust
   /// Player weapon loadout: up to 3 equipped slots + the active-slot index.
   /// `slots` uses `Option<WeaponArchetype>` so partial loadouts (1 or 2 weapons
   /// equipped) and "empty slot 3" are representable — relevant for Epic 7 unlock
   /// shop where slots are gradually unlocked. Story 4.4 ships all 3 slots filled.
   #[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
   pub struct WeaponLoadout {
       pub slots: [Option<WeaponArchetype>; 3],
       pub active_slot: usize,
   }

   impl Default for WeaponLoadout {
       fn default() -> Self {
           Self {
               slots: [
                   Some(WeaponArchetype::Pulse),
                   Some(WeaponArchetype::Shotgun),
                   Some(WeaponArchetype::Railgun),
               ],
               active_slot: 0,
           }
       }
   }

   impl WeaponLoadout {
       /// Returns the currently-active archetype, or `None` if the active slot
       /// is empty. Slot indexing is bounds-checked: `active_slot >= 3` returns
       /// `None` (programmer-error guard — set_active enforces the invariant).
       pub fn active(&self) -> Option<WeaponArchetype> {
           self.slots.get(self.active_slot).copied().flatten()
       }

       /// Cycles to the next non-empty slot (wrapping). If all slots are empty
       /// returns without mutation (impossible in 4.4 but defensive for Epic 7
       /// "unlock shop took my last weapon" edge case). Indices are walked
       /// modulo 3; at most 3 probes before bail-out.
       pub fn cycle_next(&mut self) {
           for offset in 1..=3 {
               let candidate = (self.active_slot + offset) % 3;
               if self.slots[candidate].is_some() {
                   self.active_slot = candidate;
                   return;
               }
           }
       }

       /// Direct-select slot N (0-indexed). No-op if N >= 3 or slot N is empty.
       pub fn set_active(&mut self, slot: usize) {
           if slot < 3 && self.slots[slot].is_some() {
               self.active_slot = slot;
           }
       }
   }
   ```
   **And** `Default` IS derived/impl'd on `WeaponLoadout` — this is the explicit project-default loadout (3 weapons, slot 0 active). The Default-derive lint guard (project pattern of NO-Default on Health/Projectile/EnemyShip etc.) does NOT apply here because `WeaponLoadout`'s default IS the canonical first-playable loadout, not a meaningless zero-state. Mirrors `DampenerState::default()` and `PrimaryWeaponCooldown::default()` precedents (both impl Default and are spawned via `..default()`).
   **And** the cycle-skip-empty + bounds-check logic prevents future-Epic-7 footguns ("unlock shop emptied slot 2; cycle from slot 1 must skip to slot 3, not panic"). Tested per AC #11.

4. **(`CombatAction` extension — `src/combat/input.rs`)** The existing `CombatAction` enum is extended:
   ```rust
   #[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
   pub enum CombatAction {
       FirePrimary,
       CycleWeapon,
       SelectSlot1,
       SelectSlot2,
       SelectSlot3,
   }
   ```
   **And** `default_input_map()` is extended:
   ```rust
   pub fn default_input_map() -> InputMap<CombatAction> {
       InputMap::new([
           // Map-tuple constructor for KeyCode bindings (one-input-per-action).
           // Mouse + key inputs live in InputKind-heterogenous bindings, so
           // FirePrimary lives in a separate `.insert(...)` chain below.
       ])
       .insert(CombatAction::FirePrimary, MouseButton::Left)
       .insert(CombatAction::CycleWeapon, KeyCode::Tab)
       .insert(CombatAction::SelectSlot1, KeyCode::Digit1)
       .insert(CombatAction::SelectSlot2, KeyCode::Digit2)
       .insert(CombatAction::SelectSlot3, KeyCode::Digit3)
       .build() // returns InputMap<CombatAction>
   }
   ```
   **And** the binding choice for `CycleWeapon` is **`KeyCode::Tab`**, NOT `KeyCode::KeyQ` as the epic-4.4 spec line 132 suggests — **rationale:** `FlightAction::RollLeft` is bound to `KeyCode::KeyQ` at `src/flight/input.rs:31`. Leafwing-input-manager processes both action enums concurrently; pressing Q would fire both `RollLeft` AND `CycleWeapon` every press, producing a Roll+weapon-cycle combo on every Q press — unusable. **Tab** is unbound across both action enums, near the WSAD cluster for thumb-roll-by-pinky reach, and a familiar weapon-cycle binding from established shooters (Titanfall 2, Doom). The epic-spec wording "default binding: Q, **or** 1/2/3 direct-select" reads as two alternative proposals; this story selects "1/2/3 direct-select" + a non-conflicting cycle key as the resolution.
   **And** the `leafwing_input_manager` API used is `InputMap::new(<empty>).insert(action, input).build()` because mixing `MouseButton` + `KeyCode` inputs in a single `InputMap::new(...)` constructor call requires heterogeneous-input typing that the `.insert()` chain handles cleanly. **Verify at compile time:** if leafwing's current API does not expose `.build()` or `.insert()` exactly as above, the dev may use the equivalent `InputMap::default().with(action, input)` pattern — semantics are identical (one mouse-button + four keyboard-keys, all bound).
   **And** **NO** changes to `FlightAction` or `src/flight/input.rs` — weapon controls live entirely inside `CombatAction`. The Q/E roll bindings stay as Story 3.7 established them.

5. **(`PrimaryWeaponCooldown` semantics extension — `src/combat/components.rs`)** The `PrimaryWeaponCooldown` component is **NOT** modified structurally — it remains `{ remaining: f32 }`. The semantic meaning is extended in the existing doc-comment:
   ```rust
   /// Per-ship primary-weapon rate-limit state. `remaining` counts seconds
   /// until the next shot is permitted (regardless of archetype — the cooldown
   /// is a single shared timer, NOT per-archetype). Cycling to a different
   /// archetype does NOT reset the cooldown — the player cannot dodge a slow
   /// archetype's cooldown by cycling away then back. Cooldown duration on
   /// each fire is computed from the active archetype's `fire_rate_hz`
   /// (Story 4.4) — was `tuning.projectile_fire_rate_hz` (Story 3.9).
   /// Default `0.0` so the first `FirePrimary` press fires instantly.
   #[derive(Component, Debug, Clone, Copy, Default, PartialEq)]
   pub struct PrimaryWeaponCooldown {
       pub remaining: f32,
   }
   ```
   **And** the shared-cooldown design choice is **deliberate** — per-archetype cooldowns would let the player rapid-cycle (e.g., Pulse → Shotgun → Railgun) to fire all three weapons in <0.3 s, which trivializes the railgun's slow fire rate. Shared cooldown is the standard "weapon hotswap" pattern from established shooters (Halo, Destiny). Tested by AC #11 (post-fire cooldown == active archetype's `1/fire_rate_hz`, and a cycle-then-fire DOES respect the residual cooldown).
   **And** the existing `primary_weapon_cooldown_default_is_zero` test at `src/combat/components.rs:38-41` is **PRESERVED** unchanged.

6. **(`PlayerShip` spawn-tuple extension + nested-tuple grouping — `src/flight/mod.rs::spawn_player_ship`)** The existing flat 15-component spawn tuple is restructured into a 3-subtuple nested-group, per the Bevy 0.18 `Bundle` derive arity cap of 15 (per Story 4.3 Dev Notes "Bundle-arity 15 cap"). Adding `WeaponLoadout::default()` brings arity to 16 — at the cap, requires nested-grouping:
   ```rust
   commands
       .spawn((
           (
               PlayerShip,
               ArenaEntity,
               Mesh3d(ship_mesh),
               MeshMaterial3d(ship_material),
               Transform::from_xyz(0.0, 0.0, 0.0),
               outline,
           ),
           (
               RigidBody::Dynamic,
               Collider::sphere(2.0),
               LinearVelocity(Vec3::ZERO),
               AngularVelocity(Vec3::ZERO),
               CollisionEventsEnabled,
           ),
           (
               Health {
                   current: tuning.player_hull_max,
                   max: tuning.player_hull_max,
               },
               default_input_map(),
               ActionState::<FlightAction>::default(),
               DampenerState::default(),
               WeaponLoadout::default(),
           ),
       ))
       .with_children(|parent| {
           parent.spawn((
               Camera3d::default(),
               CockpitCamera,
               Transform::from_xyz(0.0, 0.6, 0.5),
           ));
       });
   ```
   **And** Bevy auto-flattens nested tuples up to 15 per level — three sub-tuples of (6, 5, 5) components each is well under the cap and matches the `enemy.rs:71-98` precedent (4 sub-tuples of (7, 4, 5, 2)).
   **And** the spawn `info!` log line is unchanged.
   **And** the existing component set (PlayerShip, ArenaEntity, Mesh3d, MeshMaterial3d, Transform, outline, RigidBody, Collider, LinearVelocity, AngularVelocity, CollisionEventsEnabled, Health, default_input_map, ActionState<FlightAction>, DampenerState) is **PRESERVED unchanged** — only ADDITIVE plus the structural nesting.
   **And** the new component `WeaponLoadout::default()` is added to the third sub-tuple (component-set #3 of the 3-group nest), alongside the other state-style components (Health, DampenerState) — grouping by domain: visual/structural / physics / gameplay-state.

7. **(`use crate::combat::weapons::WeaponLoadout` — `src/flight/mod.rs`)** The new `WeaponLoadout` import is added to `src/flight/mod.rs::use` block alongside `crate::combat::health::Health`:
   ```rust
   use crate::combat::health::Health;
   use crate::combat::weapons::WeaponLoadout;
   ```
   **And** the import is grouped with the other `crate::combat::*` imports (alphabetical).
   **And** **NO** other imports are added — `WeaponArchetype` is not directly named in `spawn_player_ship` (it's hidden inside `WeaponLoadout::default()`).

8. **(`fire_primary_weapon` refactor — `src/combat/projectiles.rs`)** The existing `fire_primary_weapon` system is refactored to read the active archetype's stats and spawn N projectiles with per-archetype damage/speed/spread:
   ```rust
   pub fn fire_primary_weapon(
       time: Res<Time>,
       tuning_assets: Res<Assets<TuningConfig>>,
       tuning_handle: Res<TuningHandle>,
       mut commands: Commands,
       mut meshes: ResMut<Assets<Mesh>>,
       mut materials: ResMut<Assets<ToonMaterial>>,
       mut ships: Query<
           (
               &ActionState<CombatAction>,
               &Transform,
               &LinearVelocity,
               &mut PrimaryWeaponCooldown,
               &WeaponLoadout,
           ),
           With<PlayerShip>,
       >,
   )
   ```
   System body responsibilities (in order):
   - **Cold-start fallback** — `tuning = tuning_assets.get(...).cloned().unwrap_or_default()` (existing pattern, unchanged).
   - **Cooldown tick** — `cooldown.remaining = (cooldown.remaining - dt).max(0.0)` (existing pattern, unchanged).
   - **Active-archetype lookup** — `let Some(archetype) = loadout.active() else { continue; };` (skip the fire path if active slot is empty — Epic 7 forward-compat).
   - **Stats lookup** — `let stats = archetype.stats_from(&tuning);`.
   - **Damage-validation guard** — `if stats.damage == 0 { warn!("zero-damage archetype {:?} suppressed (would despawn projectiles harmlessly)", archetype); continue; }` — closes deferred-work.md:289 (zero-damage projectile silently despawning); also guards against future tuning.ron corruption.
   - **Projectile-count-validation guard** — `if stats.projectile_count == 0 { warn!("zero-projectile archetype {:?} suppressed", archetype); continue; }` — defensive against tuning corruption.
   - **Fire-gate** — `if action.pressed(&CombatAction::FirePrimary) && cooldown.remaining <= 0.0 { ... }` — existing pattern.
   - **Spread-fan computation** — `let directions = spread_forwards(*transform.forward(), *transform.up(), stats.projectile_count, stats.spread_deg);` (pure helper; AC #9).
   - **Per-direction spawn** — for each direction in the fan: spawn 1 projectile with that direction. The existing per-shot mesh+material allocation pattern is preserved (deferred-work.md:246 known — Shotgun's 5-projectile fire ESCALATES the asset-allocation pattern; flagged in Dev Notes, NOT addressed in 4.4; Story 10.1/10.2 performance pass scope).
   - **Cooldown set** — `cooldown.remaining = 1.0 / stats.fire_rate_hz.max(f32::EPSILON);` (uses `stats.fire_rate_hz`, NOT `tuning.projectile_fire_rate_hz`).
   - **Info log** — `info!("fired {:?}: {} projectiles at fire_rate={:.2}Hz (next-fire {:.2}s)", archetype, stats.projectile_count, stats.fire_rate_hz, cooldown.remaining);` — replaces the old `info!("fired projectile at velocity=...")` log; the per-shot velocity log is dropped (Shotgun would emit 5 lines per trigger which floods logs).

   Per-projectile spawn body (called once per direction in the fan):
   ```rust
   let spawn_pos = transform.translation + direction * PROJECTILE_SPAWN_OFFSET;
   let velocity = projectile_initial_velocity(ship_velocity.0, direction, stats.projectile_speed);

   // Per-archetype visual differentiation (placeholder-grade per epic-4.4 AC line 146):
   //   - Pulse:   radius = PROJECTILE_RADIUS (0.2 m, baseline)
   //   - Shotgun: radius = PROJECTILE_RADIUS (0.2 m, same as Pulse — 5 of them suffices for visual distinction)
   //   - Railgun: radius = PROJECTILE_RADIUS * 1.5 (0.3 m, beefier projectile signals high-damage)
   let radius = match archetype {
       WeaponArchetype::Pulse | WeaponArchetype::Shotgun => PROJECTILE_RADIUS,
       WeaponArchetype::Railgun => PROJECTILE_RADIUS * 1.5,
   };
   let projectile_mesh = meshes.add(Sphere::new(radius).mesh().ico(2).expect("..."));
   let projectile_material = materials.add(ToonMaterial { tint: color_for(SemanticAccent::Neutral).into(), ..default() });

   commands.spawn((
       Projectile {
           ttl: tuning.projectile_ttl_seconds,
           damage: stats.damage,
       },
       ArenaEntity,
       Mesh3d(projectile_mesh),
       MeshMaterial3d(projectile_material),
       Transform::from_translation(spawn_pos),
       RigidBody::Dynamic,
       Collider::sphere(radius),
       LinearVelocity(velocity),
       CollisionLayers::new([GameLayer::Projectile], [GameLayer::Asteroid, GameLayer::Enemy]),
       CollisionEventsEnabled,
   ));
   ```
   **And** `Projectile.damage` is now sourced from `stats.damage` (was hardcoded `1` at `projectiles.rs:116`) — **closes deferred-work.md:279** (projectile damage hardcoded).
   **And** **NO** changes to `tick_projectile_ttl` (lifecycle is archetype-agnostic).
   **And** **NO** changes to `attach_combat_to_player_ship` (cooldown / input-map attachment is archetype-agnostic).
   **And** `SemanticAccent::Neutral` for projectile tint is **PRESERVED** — Story 4.5 will sweep all player-projectiles to `SemanticAccent::PlayerOwned`, per the deferred-work.md:208 hand-off ("Story 3.9 extends the same deferral to Projectile entities"). 4.4 does NOT pre-empt 4.5's retro-tint sweep.

9. **(`spread_forwards` pure helper — `src/combat/weapons.rs`)** A new pure helper computes the N projectile-forward directions for a fan-spread:
   ```rust
   /// Pure helper: distribute `count` projectile-forward unit vectors symmetrically
   /// across a horizontal fan of total angular width `2 * spread_deg` around the
   /// ship-forward axis, rotating around the ship-up axis.
   ///
   /// Edge cases:
   /// - `count == 0` returns empty vec (caller should guard; AC #8 enforces).
   /// - `count == 1` returns `[forward]` (no spread, regardless of spread_deg).
   /// - `spread_deg == 0.0` returns `count` copies of `forward` (degenerate fan).
   /// - `up` collinear with `forward` (cockpit pointed straight up/down) falls
   ///   back to `Vec3::Y` as the rotation axis. Matches the enemy_ai.rs:159-163
   ///   look_at degenerate-up guard pattern.
   ///
   /// For `count > 1` the angles linspace from `-spread_deg` to `+spread_deg`:
   /// e.g., count=5, spread=15° → angles = [-15, -7.5, 0, +7.5, +15].
   /// Symmetric around forward; sums to zero offset; deterministic (no RNG).
   pub fn spread_forwards(forward: Vec3, up: Vec3, count: u32, spread_deg: f32) -> Vec<Vec3> {
       if count == 0 {
           return Vec::new();
       }
       if count == 1 {
           return vec![forward];
       }
       let rotation_axis = if forward.normalize_or_zero().dot(up.normalize_or_zero()).abs() > 1.0 - 1e-4 {
           Vec3::Y
       } else {
           up.normalize_or_zero()
       };
       (0..count)
           .map(|i| {
               let t = i as f32 / (count - 1) as f32; // 0.0 .. 1.0
               let angle_deg = -spread_deg + 2.0 * spread_deg * t;
               Quat::from_axis_angle(rotation_axis, angle_deg.to_radians()) * forward
           })
           .collect()
   }
   ```
   **And** the helper returns `Vec<Vec3>` (NOT `[Vec3; N]` for fixed N) so call sites can iterate variably-sized fans without const-generic plumbing.
   **And** the rotation axis is the **ship's up axis** (NOT global Y) — a player who rolls 90° still gets a horizontal-relative-to-cockpit fan (matches first-person shooter intuition: the shotgun spreads horizontally across what the player sees, not horizontally in world space).
   **And** `Quat::from_axis_angle(...) * forward` is the canonical Bevy rotation pattern; vectorization is irrelevant at N=5.
   **And** the helper lives in `src/combat/weapons.rs`, NOT `projectiles.rs` (per architecture.md:567 "weapons.rs: 3 prefab weapon archetypes, **firing systems**" — pure helpers for archetype dispatch belong in weapons.rs).

10. **(Cycle / direct-select systems — `src/combat/weapons.rs`)** Two new FixedUpdate systems handle the weapon-selection input. Both gated by `in_state(GameState::Arena)`:
    ```rust
    /// Reads `CycleWeapon` action presses; advances loadout active_slot via
    /// `WeaponLoadout::cycle_next`. `just_pressed` (not `pressed`) — Tab held
    /// fires once per press, not per tick. Logs the new active archetype.
    pub fn cycle_active_weapon(
        mut ships: Query<&mut WeaponLoadout, With<PlayerShip>>,
        action_state: Single<&ActionState<CombatAction>, With<PlayerShip>>,
    ) {
        if !action_state.just_pressed(&CombatAction::CycleWeapon) {
            return;
        }
        for mut loadout in &mut ships {
            let prev = loadout.active_slot;
            loadout.cycle_next();
            if let Some(now_active) = loadout.active() {
                info!("weapon cycle: slot {} → slot {} ({:?})", prev, loadout.active_slot, now_active);
            }
        }
    }

    /// Reads `SelectSlot1`/`SelectSlot2`/`SelectSlot3` actions; calls
    /// `WeaponLoadout::set_active(0|1|2)`. just_pressed semantics same as cycle.
    pub fn select_active_weapon(
        mut ships: Query<&mut WeaponLoadout, With<PlayerShip>>,
        action_state: Single<&ActionState<CombatAction>, With<PlayerShip>>,
    ) {
        const SLOT_KEYS: [(CombatAction, usize); 3] = [
            (CombatAction::SelectSlot1, 0),
            (CombatAction::SelectSlot2, 1),
            (CombatAction::SelectSlot3, 2),
        ];
        for (action, slot) in SLOT_KEYS {
            if action_state.just_pressed(&action) {
                for mut loadout in &mut ships {
                    let prev = loadout.active_slot;
                    loadout.set_active(slot);
                    if prev != loadout.active_slot {
                        if let Some(now_active) = loadout.active() {
                            info!("weapon select: slot {} → slot {} ({:?})", prev, loadout.active_slot, now_active);
                        }
                    }
                }
            }
        }
    }
    ```
    **And** both systems use `just_pressed` semantics — Tab held does NOT spam-cycle every FixedUpdate tick (would result in a cycle every ~17 ms, unusable). Digit keys 1/2/3 held also do NOT spam-re-select.
    **And** **NO** archetype-related side-effects on cycle/select other than `WeaponLoadout` mutation + info log: cooldowns are NOT reset (per AC #5), HUD updates land in Epic 10 polish.
    **And** the `for mut loadout in &mut ships` loop is a single-iteration-in-practice loop (the player has exactly 1 ship); the loop form is preserved for symmetry with other combat systems (`fire_primary_weapon` uses the same pattern).
    **And** `Single<&ActionState<CombatAction>, With<PlayerShip>>` is used because Cycle/Select is a player-only input — `Single<>` panics if zero or multiple matches (correct in Arena state; impossible to have 0 or 2 PlayerShips).

11. **(`CombatPlugin::build` — system registrations — `src/combat/mod.rs`)** Three additions:
    - `mod weapons;` declaration alongside the existing module list (`pub mod weapons;` so other modules can import).
    - Two new FixedUpdate system registrations alongside the existing Fire-set systems:
      ```rust
      weapons::cycle_active_weapon
          .in_set(CombatSystems::Fire)
          .run_if(in_state(GameState::Arena)),
      weapons::select_active_weapon
          .in_set(CombatSystems::Fire)
          .run_if(in_state(GameState::Arena)),
      ```
    - Both new systems are placed in `CombatSystems::Fire` (NOT a new set) — they mutate `WeaponLoadout.active_slot` BEFORE `fire_primary_weapon` reads it. Within `CombatSystems::Fire`, system-set siblings have no ordering constraint between them — but cycle/select and fire **DO** need ordering (cycle must run before fire-on-same-tick to take effect). **Fix:** add explicit chain: cycle/select before fire. Implementation:
      ```rust
      (
          weapons::cycle_active_weapon,
          weapons::select_active_weapon,
          projectiles::fire_primary_weapon,
      )
          .chain()
          .in_set(CombatSystems::Fire)
          .run_if(in_state(GameState::Arena)),
      ```
      AND remove the old standalone `projectiles::fire_primary_weapon.in_set(...)` line. The 3-system chain replaces it.
    - **NO** changes to the existing `CombatSystems` enum (no new variant needed — Fire is the right set).
    - **NO** changes to `enemy_ai::enemy_fire_weapon` registration (enemy is archetype-agnostic in 4.4).
    **And** import additions in `src/combat/mod.rs`:
    ```rust
    // No new imports needed — weapons::* is referenced via path.
    ```
    (Hint: only `pub mod weapons;` is added; system functions are reached via `weapons::cycle_active_weapon` path.)

12. **(`src/combat/projectiles.rs` import additions)** New `use` statements:
    ```rust
    use crate::combat::weapons::{WeaponArchetype, WeaponLoadout, spread_forwards};
    ```
    **And** the existing `crate::combat::input::{CombatAction, default_input_map}` import is **PRESERVED unchanged**.

13. **(`src/combat/weapons.rs` import block)** The new file's `use` block at the top:
    ```rust
    use bevy::prelude::*;
    use leafwing_input_manager::prelude::*;

    use crate::combat::input::CombatAction;
    use crate::flight::PlayerShip;
    use crate::tuning::config::{TuningConfig, WeaponArchetypeStats};
    ```
    **And** `WeaponArchetypeStats` is imported from `crate::tuning::config` (it lives in tuning, not weapons — tuning owns the data shape; weapons owns the archetype enum + dispatch).

14. **(Tests — pure-helper + invariant coverage — `src/combat/weapons.rs::tests`)** A new `#[cfg(test)] mod tests { ... }` block at the bottom of `src/combat/weapons.rs` with **8 net new tests** covering all pure helpers and the WeaponLoadout invariants:

    ```rust
    #[test]
    fn weapon_archetype_variants_are_distinct() {
        // Mirrors DeathCause::variants_distinct (damage.rs:429). Guards future variant additions.
        assert_ne!(WeaponArchetype::Pulse, WeaponArchetype::Shotgun);
        assert_ne!(WeaponArchetype::Shotgun, WeaponArchetype::Railgun);
        assert_ne!(WeaponArchetype::Pulse, WeaponArchetype::Railgun);
    }

    #[test]
    fn weapon_archetype_stats_from_returns_correct_archetype_data() {
        let tuning = TuningConfig::default();
        assert_eq!(WeaponArchetype::Pulse.stats_from(&tuning).damage, 1);
        assert_eq!(WeaponArchetype::Shotgun.stats_from(&tuning).projectile_count, 5);
        assert_eq!(WeaponArchetype::Railgun.stats_from(&tuning).damage, 5);
        assert!((WeaponArchetype::Shotgun.stats_from(&tuning).spread_deg - 15.0).abs() < 1e-5);
    }

    #[test]
    fn weapon_loadout_default_is_three_full_slots_slot_zero_active() {
        let loadout = WeaponLoadout::default();
        assert_eq!(loadout.slots, [Some(WeaponArchetype::Pulse), Some(WeaponArchetype::Shotgun), Some(WeaponArchetype::Railgun)]);
        assert_eq!(loadout.active_slot, 0);
        assert_eq!(loadout.active(), Some(WeaponArchetype::Pulse));
    }

    #[test]
    fn weapon_loadout_cycle_next_wraps_around_three_full_slots() {
        let mut loadout = WeaponLoadout::default();
        loadout.cycle_next(); // 0 → 1
        assert_eq!(loadout.active_slot, 1);
        loadout.cycle_next(); // 1 → 2
        assert_eq!(loadout.active_slot, 2);
        loadout.cycle_next(); // 2 → 0 (wraps)
        assert_eq!(loadout.active_slot, 0);
    }

    #[test]
    fn weapon_loadout_cycle_next_skips_empty_slots() {
        // Epic 7 unlock-shop forward-compat: slot 1 empty; cycle from slot 0 should land on slot 2.
        let mut loadout = WeaponLoadout {
            slots: [Some(WeaponArchetype::Pulse), None, Some(WeaponArchetype::Railgun)],
            active_slot: 0,
        };
        loadout.cycle_next();
        assert_eq!(loadout.active_slot, 2, "cycle should skip empty slot 1");
        loadout.cycle_next();
        assert_eq!(loadout.active_slot, 0, "cycle from 2 should wrap to 0 (skipping 1)");
    }

    #[test]
    fn weapon_loadout_set_active_ignores_empty_slot_and_oob() {
        let mut loadout = WeaponLoadout {
            slots: [Some(WeaponArchetype::Pulse), None, Some(WeaponArchetype::Railgun)],
            active_slot: 0,
        };
        loadout.set_active(1); // empty slot — no-op
        assert_eq!(loadout.active_slot, 0);
        loadout.set_active(2); // populated slot — activates
        assert_eq!(loadout.active_slot, 2);
        loadout.set_active(99); // OOB — no-op
        assert_eq!(loadout.active_slot, 2);
    }

    #[test]
    fn spread_forwards_count_one_returns_single_forward() {
        // count=1 → no spread regardless of spread_deg.
        let dirs = spread_forwards(Vec3::NEG_Z, Vec3::Y, 1, 30.0);
        assert_eq!(dirs.len(), 1);
        assert!((dirs[0] - Vec3::NEG_Z).length() < 1e-5);
    }

    #[test]
    fn spread_forwards_count_five_is_symmetric_around_forward() {
        // count=5, spread=15° → angles [-15, -7.5, 0, +7.5, +15]; middle is forward; outer pairs symmetric.
        let dirs = spread_forwards(Vec3::NEG_Z, Vec3::Y, 5, 15.0);
        assert_eq!(dirs.len(), 5);
        // Middle (index 2) is the unrotated forward.
        assert!((dirs[2] - Vec3::NEG_Z).length() < 1e-5, "middle direction = {:?}", dirs[2]);
        // Endpoints are equidistant from forward (symmetric fan).
        let left_offset = (dirs[0] - Vec3::NEG_Z).length();
        let right_offset = (dirs[4] - Vec3::NEG_Z).length();
        assert!((left_offset - right_offset).abs() < 1e-5, "fan asymmetric: left={} right={}", left_offset, right_offset);
        // Each direction is a unit vector (rotation preserves length).
        for (i, d) in dirs.iter().enumerate() {
            assert!((d.length() - 1.0).abs() < 1e-5, "dirs[{}] not unit: {:?}", i, d);
        }
    }
    ```

    **`src/tuning/config.rs::tests` (extended, NO new fns):** Each of the 3 existing tests gains 15 assertions (3 archetypes × 5 sub-fields). The `tuning_config_deserializes_from_ron_bytes` byte-string is extended with `weapon_pulse: (damage: 2, fire_rate_hz: 6.0, projectile_speed: 150.0, projectile_count: 2, spread_deg: 5.0), weapon_shotgun: (damage: 3, fire_rate_hz: 2.5, projectile_speed: 100.0, projectile_count: 7, spread_deg: 20.0), weapon_railgun: (damage: 10, fire_rate_hz: 1.0, projectile_speed: 400.0, projectile_count: 1, spread_deg: 0.0)`. Round-trip values intentionally distinct from defaults.

    **NO** new tests in `src/combat/components.rs` (PrimaryWeaponCooldown semantics unchanged; existing `primary_weapon_cooldown_default_is_zero` test preserved).
    **NO** new tests in `src/combat/projectiles.rs` (the refactored `fire_primary_weapon` is ECS-bound; pure helpers `projectile_initial_velocity` tests preserved unchanged).
    **NO** new tests in `src/combat/input.rs` (binding-table validation is exercised by the runtime smoke; no in-test ECS world setup).
    **NO** new tests in `src/flight/mod.rs` (spawn-tuple structural change is integration-level; existing flight tests are pure-helper tests; AC #15 runtime smoke verifies).

    **Net new test functions across the codebase: +8** (8 in weapons.rs; 0 elsewhere; assertions added to 3 existing tuning fns). Net post-4.4 test count: **63 + 8 = 71**. AC #14 enforces.

15. **(Verification gates — all 6 cargo commands clean — per `feedback_full_build_output.md`)** Per the project's full-output discipline (exit-0 + tail is NOT proof; full output captured per command and grep'd for `warning:|error:`):
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-4-4-check.log
    cargo build                                         2>&1 | tee /tmp/story-4-4-build.log
    cargo test                                          2>&1 | tee /tmp/story-4-4-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-4-4-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-4-4-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-4-4-release.log
    ```
    **And** **all six** logs produce **0** lines matching `grep -cE 'warning:|error:'`.
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 71** (= 63 baseline at end of 4.3 + 8 net new per AC #14).

16. **(File set — `git status --short` final)** Final set is **exactly**:
    - `M src/flight/mod.rs` (M — restructure PlayerShip spawn-tuple into 3-subtuple nested grouping; add `WeaponLoadout::default()` to component-state subtuple; add `crate::combat::weapons::WeaponLoadout` import)
    - `M src/combat/mod.rs` (M — `pub mod weapons;` declaration; replace standalone `projectiles::fire_primary_weapon` registration with 3-system chain `(cycle, select, fire).chain()` in CombatSystems::Fire)
    - `M src/combat/projectiles.rs` (M — refactor `fire_primary_weapon` body for active-archetype dispatch + spread fan + per-archetype damage / speed / projectile-count / mesh-radius; closes deferred-work.md:279 + :289; add weapons-module imports)
    - `M src/combat/components.rs` (M — extend `PrimaryWeaponCooldown` doc-comment to document shared-cooldown semantics across archetypes)
    - `M src/combat/input.rs` (M — extend `CombatAction` enum with `CycleWeapon`, `SelectSlot1`, `SelectSlot2`, `SelectSlot3`; extend `default_input_map` with 4 new bindings)
    - `M src/tuning/config.rs` (M — `WeaponArchetypeStats` struct + 3 archetype-stat fields with per-field serde defaults + Default impl + 3 default fns; tests extended)
    - `M assets/config/tuning.ron` (M — `weapon_pulse` / `weapon_shotgun` / `weapon_railgun` nested-struct fields added)
    - `?? src/combat/weapons.rs` at create-time (? → M after dev fills body) — NEW FILE: `WeaponArchetype` enum + `stats_from` + `WeaponLoadout` component + Default impl + `cycle_next` / `set_active` / `active` methods + `spread_forwards` pure helper + `cycle_active_weapon` / `select_active_weapon` systems + 8 tests
    - `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — `4-4-...: backlog → ready-for-dev → in-progress → review → done`, `last_updated`)
    - `M _bmad-output/implementation-artifacts/deferred-work.md` (M — close entries at line 279 + 289; add new entry escalating line 246 per-shot-allocation severity for Shotgun's 5×-rate)
    - `?? _bmad-output/implementation-artifacts/4-4-weapon-archetype-system-shotgun-railgun-archetypes.md` at story-creation time (becomes M after dev flips Status / fills Dev Agent Record / Change Log)

    **NO** entries under: `Cargo.toml` / `Cargo.lock` (no dep added — `Quat`, `Vec3` already in scope via Bevy prelude), `src/combat/damage.rs` (damage routing is archetype-agnostic — `Projectile.damage` flows through unchanged), `src/combat/enemy.rs` / `src/combat/enemy_ai.rs` / `src/combat/health.rs` (enemy archetype is single-variant per Story 4.2), `src/arena/**`, `src/ui/**` (HUD weapon-display lands in Epic 10 polish), `src/flight/input.rs` (Q stays on RollLeft), `src/persistence/**`, `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/state.rs`, `src/pause/**`, `assets/meshes/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

17. **(Runtime smoke — full archetype-cycle-and-fire chain)** After AC #15 cargo gates green, Till manually executes `cargo run 2>&1 | tee /tmp/story-4-4-run.log` and verifies:
    - **(a) Default loadout fires Pulse on entry** — Press Enter on MainMenu; once in Arena, hold LMB. Projectiles fire at ~4 Hz with damage=1 (asteroid destroyed in 1 hit — matches 3.10 baseline). `grep -c "fired Pulse: 1 projectiles" /tmp/story-4-4-run.log ≥ 1`.
    - **(b) Tab cycles to Shotgun** — Press Tab once; HUD has no weapon-name display in 4.4 (polish item), so verify via log: `grep -c "weapon cycle: slot 0 → slot 1 (Shotgun)" /tmp/story-4-4-run.log == 1`.
    - **(c) Shotgun fires 5 projectiles per trigger** — Hold LMB; first shot spawns 5 projectiles in a horizontal fan. Visually verify: 5 distinct projectile-spheres emit from the ship's nose; the outer two are at ~±15° from ship-forward. Fire rate is slower (~1.5 Hz; 5-shot burst once per ~0.67s). `grep -c "fired Shotgun: 5 projectiles" /tmp/story-4-4-run.log ≥ 1`.
    - **(d) Digit 3 direct-selects Railgun** — Press `3` key; `grep -c "weapon select: slot 1 → slot 2 (Railgun)" /tmp/story-4-4-run.log == 1`. Fire: 1 large projectile at high speed (~300 m/s, visually faster than Pulse); damage=5 (one-shot kills the 2-HP enemy from Story 4.2). Cooldown ~2 s before next fire.
    - **(e) Digit 1 direct-selects Pulse** — Press `1` key; cycles back to slot 0. `grep -c "weapon select: slot 2 → slot 0 (Pulse)" /tmp/story-4-4-run.log == 1`.
    - **(f) Tab held does not spam-cycle** — Hold Tab for 1 second; `grep -c "weapon cycle" /tmp/story-4-4-run.log` increases by exactly **1** (one just_pressed firing, NOT 60+ FixedUpdate spams).
    - **(g) Cooldown is shared across archetypes (no hot-swap-fire bypass)** — Fire Railgun (cooldown 2s); within that 2s window, press Tab to cycle to Pulse and hold LMB. **Pulse does NOT fire immediately** — cooldown is shared. After ~2s, Pulse fires. `grep -c "fired Pulse" /tmp/story-4-4-run.log` confirms the fire happens only after the Railgun cooldown elapsed.
    - **(h) Per-archetype visual differentiation** — Pulse and Shotgun share a 0.2 m sphere mesh (Shotgun is differentiated by the 5-projectile fan, not by per-projectile shape). Railgun spawns a 0.2 m × 1.0 m capsule (lance) rotated along the flight axis — clearly distinct in silhouette from the spheres, especially when viewed off-axis or chased toward an off-center asteroid. Initial 1.5× sphere-radius approach was tested and rejected by Till on 2026-05-11 as not visibly distinguishable from cockpit perspective at Railgun's 300 m/s speed; capsule form is the placeholder-grade replacement. Verify by eyeball: a fired Railgun projectile looks elongated, the Pulse/Shotgun projectiles look round.
    - **(i) Roll Q + cycle Tab are independent** — Press Q (ship rolls left per FlightAction::RollLeft) WITHOUT triggering a weapon cycle. `grep -c "weapon cycle" /tmp/story-4-4-run.log` is unchanged after pressing Q.

18. **(Pre-flight: NO out-of-scope work.)** Story 4.4 explicitly does NOT:
    - Add HUD weapon-name / weapon-icon display — Epic 10 polish (Story 10.8 UI polish pass).
    - Add per-archetype projectile colors or distinct meshes beyond radius — Epic 10 visual polish.
    - Add per-archetype SFX — Epic 8 (audio plugin) wires weapon-fire sounds.
    - Add `SemanticAccent::PlayerOwned` retroactive tinting to projectiles — Story 4.5 (per deferred-work.md:208).
    - Cache projectile mesh+material handles to address per-shot allocation churn — deferred per deferred-work.md:246, escalated by 4.4 (Shotgun's 5×-rate), still deferred to Epic 10 Story 10.1/10.2.
    - Wire `salvage_banked` cost-per-shot ("pay-to-shoot" FR11) — Epic 6 Story 6.9.
    - Refactor `enemy_ai::enemy_fire_weapon` to use archetype dispatch — enemies stay single-archetype per Story 4.2; archetype is a player-only construct in 4.4.
    - Add weapon-equipped-but-no-active-slot UI feedback — Epic 7 unlock-shop scope.
    - Validate tuning.ron weapon-stat ranges (e.g., negative speed, fire_rate_hz==0) — the `f32::EPSILON` floor in cooldown-set is the only guard; comprehensive tuning-input validation is deferred per deferred-work.md:222/228 to a tuning-hardening pass.
    - Migrate `tuning.projectile_fire_rate_hz` removal (becomes unused by player path) — leave field in tuning.ron and TuningConfig for forward-compat; if dev-cycle hot-reload tweaks reference it, the no-op nature does no harm. A future cleanup story may remove it.
    - Add a "weapon UI overlay" or "weapon select wheel" — out-of-scope; first-playable polish.

## Tasks / Subtasks

- [x] **Task 1 — Tuning extension: WeaponArchetypeStats + 3 fields** (AC: #1)
  - [x] Add `WeaponArchetypeStats` struct with Serde derives + 5 fields to `src/tuning/config.rs`
  - [x] Add 3 fields (`weapon_pulse`, `weapon_shotgun`, `weapon_railgun`) with `#[serde(default = "...")]` annotations
  - [x] Add 3 default fns returning the canonical Pulse/Shotgun/Railgun stat values per AC #1 literal
  - [x] Extend `TuningConfig::default()` with the new fields
  - [x] Extend the 3 existing `tuning::config::tests` with +15 assertions each (3 archetypes × 5 sub-fields); update the round-trip byte-string with distinct round-trip values
  - [x] Add `weapon_pulse`, `weapon_shotgun`, `weapon_railgun` nested-struct fields to `assets/config/tuning.ron`
  - [x] Run `cargo test tuning::config::tests` — all 3 tests pass with the +45 assertions total

- [x] **Task 2 — Create `src/combat/weapons.rs` module skeleton** (AC: #2, #3, #13)
  - [x] Author new file `src/combat/weapons.rs` with the use block per AC #13
  - [x] Add `pub mod weapons;` declaration to `src/combat/mod.rs` alongside existing modules
  - [x] Add `WeaponArchetype` enum with 3 variants (`Pulse`, `Shotgun`, `Railgun`) + `Component` + `Debug` + `Clone` + `Copy` + `PartialEq` + `Eq` derives
  - [x] Add `WeaponArchetype::stats_from(self, tuning: &TuningConfig) -> WeaponArchetypeStats` pure helper with exhaustive match
  - [x] Add `WeaponLoadout` component with `slots: [Option<WeaponArchetype>; 3]` + `active_slot: usize`
  - [x] Implement `WeaponLoadout::default()` returning `[Pulse, Shotgun, Railgun]` at slot 0
  - [x] Implement `WeaponLoadout::active(&self) -> Option<WeaponArchetype>` (bounds-checked)
  - [x] Implement `WeaponLoadout::cycle_next(&mut self)` (skips empty slots; bails after 3 probes)
  - [x] Implement `WeaponLoadout::set_active(&mut self, slot: usize)` (no-op on OOB or empty)
  - [x] Verify `cargo check` passes

- [x] **Task 3 — Pure helper `spread_forwards`** (AC: #9, #14)
  - [x] Implement `spread_forwards(forward: Vec3, up: Vec3, count: u32, spread_deg: f32) -> Vec<Vec3>` in `src/combat/weapons.rs`
  - [x] Cover edge cases: count==0 → empty, count==1 → [forward], collinear forward/up → fallback Vec3::Y axis
  - [x] Use `Quat::from_axis_angle(rotation_axis, angle_deg.to_radians()) * forward` per Bevy convention
  - [x] Use linspace from `-spread_deg` to `+spread_deg` for count > 1 (e.g., count=5, spread=15 → [-15, -7.5, 0, 7.5, 15])

- [x] **Task 4 — Cycle / direct-select systems** (AC: #10)
  - [x] Extend `CombatAction` enum at `src/combat/input.rs` with `CycleWeapon`, `SelectSlot1`, `SelectSlot2`, `SelectSlot3`
  - [x] Extend `default_input_map()` with 5 bindings: LMB=FirePrimary, Tab=CycleWeapon, Digit1/2/3=SelectSlot1/2/3
  - [x] **Resolve Q binding conflict:** verify `KeyCode::KeyQ` is NOT used as `CycleWeapon` (would collide with `FlightAction::RollLeft` at `flight/input.rs:31`); use `KeyCode::Tab` instead
  - [x] Verify the leafwing-input-manager API for mixing MouseButton + KeyCode in one InputMap — likely `InputMap::new(<empty>).insert(...).insert(...).build()` or `InputMap::default().with(...).with(...)`. If the exact chain differs from AC #4 code-shape, use the equivalent that compiles
  - [x] Implement `cycle_active_weapon(mut ships: Query<&mut WeaponLoadout, With<PlayerShip>>, action_state: Single<&ActionState<CombatAction>, With<PlayerShip>>)` with `just_pressed` semantics
  - [x] Implement `select_active_weapon(...)` with the `SLOT_KEYS: [(CombatAction, usize); 3]` constant + per-slot loop

- [x] **Task 5 — `fire_primary_weapon` refactor** (AC: #5, #8)
  - [x] Extend the `fire_primary_weapon` query tuple with `&WeaponLoadout` (5th field)
  - [x] Read active archetype via `loadout.active()`; skip the fire path if `None`
  - [x] Look up archetype stats via `archetype.stats_from(&tuning)`
  - [x] Add zero-damage and zero-projectile guards with `warn!` logs (closes deferred-work.md:289)
  - [x] Replace single-projectile spawn with a fan-loop over `spread_forwards(...)` results
  - [x] Set `Projectile.damage = stats.damage` (closes deferred-work.md:279)
  - [x] Set per-projectile `radius` per archetype (Pulse=Shotgun=0.2, Railgun=0.3)
  - [x] Use `stats.fire_rate_hz` for cooldown-set (was `tuning.projectile_fire_rate_hz`)
  - [x] Replace per-shot velocity log with a single per-trigger `info!("fired {:?}: {} projectiles ...")` log

- [x] **Task 6 — CombatPlugin wiring** (AC: #11)
  - [x] Add `pub mod weapons;` declaration to `src/combat/mod.rs`
  - [x] Replace the standalone `projectiles::fire_primary_weapon` registration with the 3-system chain `(weapons::cycle_active_weapon, weapons::select_active_weapon, projectiles::fire_primary_weapon).chain().in_set(CombatSystems::Fire).run_if(in_state(GameState::Arena))`
  - [x] Verify the new chain replaces (NOT augments) the old standalone fire registration
  - [x] Verify `cargo build` passes — set-graph wiring is structurally valid

- [x] **Task 7 — PlayerShip spawn-tuple restructure** (AC: #6, #7)
  - [x] Add `use crate::combat::weapons::WeaponLoadout;` to `src/flight/mod.rs` imports
  - [x] Restructure the existing 15-component flat tuple in `spawn_player_ship` into a 3-subtuple nested group: (visual: 6 components) + (physics: 5 components) + (state: 5 components — including the new `WeaponLoadout::default()`)
  - [x] Verify the `with_children` block for the cockpit Camera3d is unchanged
  - [x] Verify the spawn `info!` log line is unchanged
  - [x] `cargo build` — verifies the nested-tuple grouping compiles within the Bevy 0.18 Bundle-derive arity cap

- [x] **Task 8 — PrimaryWeaponCooldown doc-comment update** (AC: #5)
  - [x] Update the doc-comment block at `src/combat/components.rs:18-22` to document shared-cooldown semantics across archetypes (cycling does NOT reset the cooldown)
  - [x] No code changes; struct shape and Default derive unchanged

- [x] **Task 9 — Tests** (AC: #14)
  - [x] Add 8 new tests in `src/combat/weapons.rs::tests` per AC #14 literal:
    - `weapon_archetype_variants_are_distinct`
    - `weapon_archetype_stats_from_returns_correct_archetype_data`
    - `weapon_loadout_default_is_three_full_slots_slot_zero_active`
    - `weapon_loadout_cycle_next_wraps_around_three_full_slots`
    - `weapon_loadout_cycle_next_skips_empty_slots`
    - `weapon_loadout_set_active_ignores_empty_slot_and_oob`
    - `spread_forwards_count_one_returns_single_forward`
    - `spread_forwards_count_five_is_symmetric_around_forward`
  - [x] Run `cargo test` — total = 71 (63 baseline + 8 net new), all pass

- [x] **Task 10 — Verification gates** (AC: #15)
  - [x] Run all 6 cargo commands (check / build / test / clippy / fmt / release) with `2>&1 | tee /tmp/story-4-4-{name}.log`
  - [x] `grep -cE 'warning:|error:' /tmp/story-4-4-*.log` returns 0 for all 6 logs
  - [x] Fix any clippy or fmt issues **at root** — do NOT add `#[allow(...)]` without a reasoned justification per the project's full-output discipline

- [x] **Task 11 — Runtime smoke** (AC: #17) — *Executed by Till 2026-05-11*
  - [x] `cargo run` walkthrough; scenarios (a)–(g), (i) green on first pass
  - [x] Scenario (h) initially failed (1.5× sphere not visibly distinguishable); fixed in-session with Capsule3d lance + Y-axis rotation; re-smoke (h) green

- [x] **Task 12 — Close deferred-work entries** (closes existing entries 279 + 289; flags 246 escalation)
  - [x] In `_bmad-output/implementation-artifacts/deferred-work.md`, append `> **✅ CLOSED 2026-MM-DD by Story 4.4** — projectile damage is now sourced from the active archetype's `stats.damage` (Pulse=1, Shotgun=1, Railgun=5); no longer hardcoded.` to the entry at line 279
  - [x] Append a similar closure note to the entry at line 289 — zero-damage guard in `fire_primary_weapon` warn!-logs and suppresses the fire path
  - [x] Add a new deferred-work entry: "Per-shot mesh+material allocation severity escalated by Shotgun" — under "Observed during: 4-4-weapon-archetype-system dev/review", noting that Shotgun's 5-projectiles-per-trigger multiplies the allocation rate by 5× vs the 3.9 baseline; same resolution path as line 246 (shared-handle resource); same scope (Epic 10 polish pass); still deferred

## Dev Notes

### Architectural Anchors

**Weapon-archetype data shape (architecture.md:567).** The `combat/weapons.rs` file is named in the architecture's directory plan as "3 prefab weapon archetypes, firing systems." Story 4.4 is the first occupant. Future post-MVP (C#6) crafting will replace this prefab enum with a composable modules system — `WeaponArchetype` enum becomes a sealed prefab catalog; loadout slots store `Weapon` (composable) instead. Out of MVP scope.

**Component-composition discipline (architecture.md:74, :460, :471).** `WeaponLoadout` carries only slot data + active-slot index; cooldown stays on a separate `PrimaryWeaponCooldown` component. NO god-struct like `Weapon { slots, cooldown, archetype, fire_rate, damage }`. The cycle/select systems mutate `WeaponLoadout` only; the fire system mutates `PrimaryWeaponCooldown` only.

**System-set chain over `.after()` (architecture.md:413-416).** The cycle/select-before-fire ordering MUST be inside `CombatSystems::Fire` via `.chain()`, NOT `.after(fire_primary_weapon)`. Renaming `fire_primary_weapon` to `fire_weapon_from_archetype` (hypothetical Epic 7 cleanup) would break the `.after(...)` form silently; the `.chain()` form is rename-safe.

**Tuning-config field design (architecture.md:357-358).** Runtime-tunable gameplay values live in tuning.ron, NOT compile-time consts. The 3 archetype stat-blocks (Pulse/Shotgun/Railgun) ALL go in tuning.ron per this pattern. Pre-existing `tuning.projectile_speed` / `tuning.projectile_fire_rate_hz` are kept (still read by enemy weapon path), but the player path is now archetype-driven — the legacy `tuning.projectile_fire_rate_hz` becomes unused by the player but stays in tuning.ron until a future cleanup.

**Forward-compat tuning-config pattern (Story 4.2 / 4.3 precedent).** Per-field `#[serde(default = "fn_name")]` on every new field. Story 4.4 introduces a new pattern variant: `#[serde(default = "...")]` on a STRUCT-typed field (`weapon_pulse: WeaponArchetypeStats`). This still composes with the existing pattern — the default fn returns a fully-constructed `WeaponArchetypeStats`. Tested by the legacy-schema test which constructs a partial RON omitting the weapon fields entirely.

**Input-action enum extension pattern (Story 3.7 precedent).** `FlightAction` enum was extended with `Pitch` / `Yaw` / `RollLeft` / `RollRight` / `ToggleDampener` across Stories 3.6–3.8 (single enum, growing variant set). Story 4.4 mirrors this for `CombatAction`. The leafwing-input-manager dispatcher handles N-way actions in one ActionState.

### Code-Reuse Discipline (LLM Wheel-Reinvention Prevention)

**REUSE — DO NOT DUPLICATE:**
- `projectile_initial_velocity(ship_velocity, forward, projectile_speed)` from `src/combat/projectiles.rs:64` — pure helper for muzzle-velocity composition; reused per-fan-direction in the refactored `fire_primary_weapon`.
- `PrimaryWeaponCooldown` from `src/combat/components.rs:21-24` — kept as the shared rate-limit component. Semantics extension only (doc-comment); structure unchanged.
- `Projectile { ttl, damage }` from `src/combat/components.rs:12-16` — already supports variable `damage` via the existing field; Story 4.4 actually USES the field instead of hardcoding `1`.
- `apply_damage` from `src/combat/damage.rs:162` — saturating-sub, archetype-agnostic; Railgun's `damage=5` flows through unchanged (already covered by the existing `apply_damage_overdamage_clamps_at_zero` test).
- `default_input_map()` pattern from both `flight/input.rs:23` and `combat/input.rs:11` — extended in-place rather than refactored.
- `ToonMaterial { tint: color_for(SemanticAccent::Neutral).into() }` material setup — unchanged per archetype; visual differentiation is mesh-radius only (Epic 10 polish owns full per-archetype visuals).
- The nested-tuple spawn pattern from `src/combat/enemy.rs:71-98` — established in Story 4.1 / 4.2 for >15-arity bundles; 4.4 applies it to PlayerShip for the first time.
- Cold-start `tuning_assets.get(...).cloned().unwrap_or_default()` pattern from `projectiles.rs:89-92` — preserved unchanged in the refactored `fire_primary_weapon`.

**DO NOT REINVENT:**
- A new `PrimaryWeaponCooldown` per archetype — shared cooldown is deliberate (AC #5 rationale).
- A new fire system per archetype — single `fire_primary_weapon` dispatches via `archetype.stats_from(&tuning)`.
- A new "weapon component" hierarchy — `WeaponLoadout` + `WeaponArchetype` enum is the only structure needed.
- Per-archetype mesh/material caches — known performance concern (deferred-work.md:246); Epic 10 polish-pass scope; would conflate this story's scope.
- Per-archetype SFX — Epic 8 (audio plugin) integration; no audio in 4.4.
- Per-archetype HUD display — Epic 10 polish.
- Random spread within a cone — deterministic linspace fan is sufficient and testable per pure-helper discipline.
- A separate `CombatSystems::WeaponSelect` set — the cycle/select systems live in `CombatSystems::Fire` with intra-set chain ordering.

### Previous Story Intelligence (Story 4.3 Learnings)

**Pattern that worked: pure-helper-first.** Story 4.3 extracted `apply_damage`, `next_ai_state` (4.2), `projectile_initial_velocity` (3.9) — all callable without ECS world setup, all tested in isolation. **Apply to 4.4:** `WeaponArchetype::stats_from`, `WeaponLoadout::active/cycle_next/set_active`, `spread_forwards` are all pure helpers — 8 of the 8 new tests are pure-helper / invariant tests with no Bevy world setup required.

**Bundle-arity 15 cap (Bevy 0.18).** Story 4.3 Dev Notes explicitly flagged: "PlayerShip currently has 13 components in the spawn tuple. Adding 2 brings it to 15 — at the edge but still under the cap." Story 4.4 ADDS one more (`WeaponLoadout`) → arity 16 → MUST use nested-tuple grouping. Mirrors enemy.rs:71-98 (4 sub-tuples). AC #6 prescribes the 3-subtuple split: visual / physics / state.

**Cold-start tuning fallback.** Stories 4.2 / 4.3 confirmed `tuning_assets.get(...).cloned().unwrap_or_default()` is the pattern. `fire_primary_weapon` already uses it (line 89-92); refactor preserves it.

**No-Default-derive discipline.** Story 4.3 added `DeathCause` / `RunResult` / `RunStartedAt` with NO Default. **Story 4.4 selectively allows Default:** `WeaponLoadout::default()` IS the canonical first-playable loadout — meaningful default, not a silent zero-state. `WeaponArchetype` does NOT impl Default (would mask explicit construction intent). The discipline is: Default-derive is OK when the default has explicit gameplay meaning; reject Default for placeholder structs where any-variant is wrong.

**Clippy `never_loop` pitfall.** Story 4.3's `check_player_death` originally used `for event in depleted.read() { ...; return; }` — clippy flagged. Refactored to `if let Some(event) = depleted.read().next() { ... }`. **Apply to 4.4:** the cycle/select systems use `for slot in ... { if just_pressed { ... } }` loops — no early return, just iteration. The `for offset in 1..=3` loop in `cycle_next` uses early `return` from a mutating function — NOT clippy-flagged (early return from a method is idiomatic).

**Q-binding conflict with FlightAction::RollLeft.** Story 3.7 bound Q/E to roll. The epic-4.4 spec suggests Q for CycleWeapon. **Apply to 4.4:** detect the conflict explicitly in the AC; pick Tab for cycle. Document the deviation in the Change Log so future stories don't accidentally re-introduce Q-cycle.

### Cross-Story Dependencies

**Depends on (must be done before 4.4):**
- 3.5 (PlayerShip spawn) — extended in AC #6 / Task 7.
- 3.9 (weapon firing baseline) — `fire_primary_weapon` and `PrimaryWeaponCooldown` are refactored in AC #8.
- 3.10 (projectile-asteroid damage routing) — `Projectile.damage` is now per-archetype (was hardcoded `1`).
- 4.2 (Health component, EnemyProjectile marker, GameLayer enum) — Railgun's `damage=5` flows through `apply_enemy_damage` which uses `saturating_sub` (one-shots a 2-HP enemy).
- 4.3 (PlayerShip Hull + permadeath) — orthogonal but the spawn-tuple restructure (AC #6) builds on 4.3's `Health` addition.

**Blocks (cannot proceed without 4.4):**
- (None within Epic 4) — 4.5, 4.6, 4.7, 4.8, 4.9, 4.10 are all orthogonal to weapon archetypes.
- Epic 7 unlock-shop (Stories 7.2–7.5) — extends `WeaponLoadout::slots` semantics (slots can be empty until unlocked); the `Option<WeaponArchetype>` shape is forward-compat for that.
- Epic 6 pay-to-shoot (Story 6.9) — debits salvage on each fire; the archetype-aware `fire_primary_weapon` is the hook point.

**Independent (parallel post-4.4):**
- 4.5 (SemanticAccent retro-tint) — visual-layer only.
- 4.6 (PersistencePlugin) — orthogonal.
- 4.7 (Title screen full), 4.8 (Settings), 4.9 (PostRun summary), 4.10 (Release workflow) — all orthogonal.

### Source Tree Components Touched (per architecture.md:564-570)

```
src/
├── combat/
│   ├── components.rs        # M: PrimaryWeaponCooldown doc-comment extension (shared-cooldown semantics)
│   ├── input.rs             # M: CombatAction enum + default_input_map extension (4 new actions + bindings)
│   ├── mod.rs               # M: pub mod weapons; declaration; cycle/select/fire chain in CombatSystems::Fire
│   ├── projectiles.rs       # M: fire_primary_weapon refactor for active-archetype dispatch + spread fan
│   └── weapons.rs           # NEW: WeaponArchetype enum + WeaponLoadout component + cycle/select systems + spread_forwards helper + 8 tests
├── flight/
│   └── mod.rs               # M: PlayerShip spawn-tuple restructure (3-subtuple nest); WeaponLoadout::default() add
└── tuning/
    └── config.rs            # M: WeaponArchetypeStats struct + 3 archetype-stat fields + Default impl + 3 default fns + tests extension

assets/
└── config/
    └── tuning.ron           # M: 3 nested weapon_pulse/shotgun/railgun fields
```

### Testing Standards Summary

**From the project's established pattern (Stories 3.6–4.3):**
- Pure-helper-first: extract testable fns before ECS-bound systems. AC #14's 8 tests are 100% pure-helper / invariant tests; no Bevy world setup required.
- No-Default-derive guard tests for types where Default would mask intent: AC #14's `weapon_archetype_variants_are_distinct` mirrors `DeathCause::variants_are_distinct` (4.3) and `HudField::variants_distinct` (3.11).
- Tuning round-trip extension for every new field: AC #1's 45 net new assertions are spread across the existing 3 tuning tests (default-matches-RON, RON-bytes-deserializes, legacy-schema-falls-back). Pattern from Stories 2.4, 3.6–3.10, 4.2, 4.3.
- Tests co-located in `mod tests { ... }` at the bottom of each module — never in `tests/` directory.
- `cargo test` full output captured + `grep -cE 'warning:|error:'` = 0, per `feedback_full_build_output.md`.

### Known Risks / Watch-Outs

**Bundle-arity 16 with nested grouping.** AC #6's 3-subtuple split (6+5+5 = 16 total) compiles within Bevy 0.18's 15-arity-per-tuple cap because the outer wrapping tuple is `(tuple_6, tuple_5, tuple_5)` — outer arity 3, each inner ≤6. If a future story adds a 17th component, re-evaluate the grouping (e.g., split state into state+input subtuples).

**Q vs Tab cycle binding.** Already mitigated by AC #4 — Tab is chosen for CycleWeapon. **Watch out:** if a future story rebinds Tab for menu navigation or any other use, the cycle binding needs to migrate. Tab is conventionally a UI-nav key, so this is a plausible future conflict (e.g., Story 4.7 title-screen full FR36 menu binding). The conflict is OOB for 4.7's MainMenu state (different state, leafwing dispatches separately by state-active input maps), but worth re-evaluating during 4.7 development.

**Shotgun 5-allocation rate.** Each Shotgun trigger spawns 5 projectiles = 5 mesh+material allocations. At 1.5 Hz fire rate over a 60-second engagement → ~450 allocations per minute. Pulse at 4 Hz → 240/min — Shotgun is 2× the rate. Asset handles are GC'd on projectile despawn (Bevy strong-handle ref-counting). **No leak**, but allocation churn is real. Deferred to Epic 10 polish (cached shared-handle resource). Flagged as escalation of deferred-work.md:246 in Task 12.

**Hot-reload via tuning.ron.** Changing `weapon_pulse: (damage: 1, ...)` to `(damage: 2, ...)` in `assets/config/tuning.ron` while the game is running triggers Bevy's asset hot-reload — `tuning_assets.get(...).cloned()` returns the new config on the next FixedUpdate tick. The active archetype's stats refresh transparently; the loadout / cycle/select state is unaffected (lives on the player entity, not in tuning). **Tested implicitly** by AC #15's `cargo test` (TuningConfig deserializes from arbitrary RON bytes).

**Pre-existing `CombatSystems::Fire` lacks `.after(FlightSystems::ApplyForces)`** — same deferred-work.md (around 3.9 review) entry. Story 4.4 does NOT close this; the velocity-staleness window stays at 1 FixedUpdate tick. Negligible at archetype-fire-rates 0.5–4 Hz.

**`leafwing-input-manager` API drift.** AC #4's `InputMap::new(<empty>).insert(...).build()` may not match the current 0.18-compatible leafwing API exactly. The dev should check `Cargo.toml` for the pinned leafwing version, then match the InputMap construction syntax to that version. The dev has discretion on the exact code shape (Task 4 line 4); the AC's intent is "5 bindings, none of which use Q".

**Project context: PrimaryWeaponCooldown semantics doc-only change risk.** Task 8 modifies only the doc-comment, not the struct shape. **Watch out:** if a future story (e.g., Epic 5 ShieldHP arrives) adds per-archetype cooldowns, the doc-comment becomes load-bearing — devs must understand 4.4's shared-cooldown decision before adding per-archetype state. The Dev Notes "Architectural Anchors" section + the doc-comment make this explicit; review-stage Edge Case Hunter should verify.

### Project Structure Notes

**Alignment with unified project structure:**
- `WeaponArchetype` + `WeaponLoadout` live in `src/combat/weapons.rs` per architecture.md:567's prescription (`weapons.rs: 3 prefab weapon archetypes, firing systems`). NOT in `combat/components.rs` (which holds the legacy generic markers like `Asteroid`, `Projectile`, `PrimaryWeaponCooldown`).
- `WeaponArchetypeStats` lives in `src/tuning/config.rs` because it's a deserialized-from-tuning.ron data shape, not a combat-domain runtime construct. The `combat/weapons.rs` module imports it.
- Cycle/select systems live in `src/combat/weapons.rs` alongside the components they mutate — domain co-location.
- `pub mod weapons;` declaration in `src/combat/mod.rs` is alphabetical-after the existing module list (component, damage, enemy, enemy_ai, health, input, projectiles, weapons).
- File creation: 1 new file (`src/combat/weapons.rs`). Story 4.2 added 2 files (`enemy_ai.rs` + `health.rs`); Story 4.3 added 0 files. Story 4.4 adds 1 new file.

**Detected variances (intentional, with rationale):**
- Epic-4.4 spec line 132 suggests Q for CycleWeapon; Story 4.4 binds Tab instead (AC #4 rationale). Documented in the Dev Notes and Change Log.
- Epic-4.4 spec line 134 says "an `info!` log records the new active archetype" — this story emits the log only on actual transitions (not on Tab-press into an empty slot), via the `if prev != loadout.active_slot` guard in `select_active_weapon` and an implicit guard via `loadout.active()` returning `Some` in `cycle_active_weapon`. Minor refinement, in the spirit of the AC.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-4-enemies-alive-stop-ship-itchio-prototype.md#Story 4.4] — Acceptance criteria (epic 4 spec lines 111-146)
- [Source: _bmad-output/planning-artifacts/architecture.md#Combat-Module-Plan] — `combat/weapons.rs` named as the archetype home (line 567)
- [Source: _bmad-output/planning-artifacts/architecture.md#Combat-Plugin-Publishes] — `CombatPlugin` publishes weapon-fire-related events; archetype dispatch lives inside CombatPlugin (line 648)
- [Source: _bmad-output/planning-artifacts/architecture.md#Component-Composition-Anti-Patterns] — god-struct prohibition (lines 460-461)
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming-Conventions] — PascalCase Components / past-tense Events / SCREAMING_SNAKE_CASE consts (lines 322-326)
- [Source: _bmad-output/planning-artifacts/architecture.md#Tuning-Convention] — Runtime-tunable values live in tuning.ron (lines 357-358)
- [Source: _bmad-output/planning-artifacts/architecture.md#System-Ordering] — `.chain()` over `.after(specific_fn)` (lines 413-416)
- [Source: _bmad-output/planning-artifacts/architecture.md#FR-Mapping] — FR9-FR10 weapons → `src/combat/weapons.rs` (line 681)
- [Source: _bmad-output/planning-artifacts/prd.md#FR10] — "Player ship equips up to 3 weapons drawn from a pool of 3 prefab archetypes" (line 512)
- [Source: _bmad-output/planning-artifacts/prd.md#Concept-Decision-C6] — "3 prefab weapon archetypes (no crafting UI in MVP per C#6 staged rollout)" (line 144)
- [Source: _bmad-output/implementation-artifacts/4-3-hull-component-permadeath-postrun-state.md] — Spawn-tuple arity-15 cap; nested-tuple grouping pattern; pure-helper testing discipline
- [Source: _bmad-output/implementation-artifacts/4-2-enemy-ai-state-machine-detect-pursue-attack.md] — Health component + EnemyProjectile marker (used by Railgun overkill case); nested-tuple precedent in enemy.rs spawn
- [Source: _bmad-output/implementation-artifacts/3-9-weapon-firing-projectile-ballistics.md] — `fire_primary_weapon` / `attach_combat_to_player_ship` / `PrimaryWeaponCooldown` baseline (refactored here)
- [Source: _bmad-output/implementation-artifacts/3-10-projectile-asteroid-collision-damage.md] — `Projectile.damage` field + `apply_damage` saturating-sub (Railgun's damage=5 flows through unchanged)
- [Source: _bmad-output/implementation-artifacts/3-7-flight-input-3-axis-rotation-pitch-yaw-roll.md] — Q/E roll bindings (informs Q-conflict resolution at AC #4)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:246] — Per-shot mesh+material allocation churn (escalated by Shotgun's 5×-rate; still deferred to Epic 10)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:279] — Projectile damage hardcoded to 1 (CLOSED by Story 4.4)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:289] — Zero-damage projectile silently despawns (CLOSED by Story 4.4 with zero-damage guard)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:208] — SemanticAccent::PlayerOwned retroactive re-tint (Story 4.5 scope; 4.4 keeps `Neutral` tint)
- [Source: src/combat/projectiles.rs:64-70] — `projectile_initial_velocity` (reused per-fan-direction in refactored fire system)
- [Source: src/combat/projectiles.rs:72-140] — `fire_primary_weapon` baseline body (refactored by AC #8)
- [Source: src/combat/components.rs:12-24] — `Projectile { ttl, damage }` + `PrimaryWeaponCooldown` (extended doc-comment in AC #5)
- [Source: src/combat/input.rs:6-13] — `CombatAction` enum + `default_input_map` (extended in AC #4)
- [Source: src/combat/damage.rs:162-164] — `apply_damage` saturating-sub (archetype-agnostic; flows Railgun's damage=5 through unchanged)
- [Source: src/combat/enemy.rs:69-98] — Nested-tuple spawn pattern (template for AC #6 PlayerShip restructure)
- [Source: src/combat/enemy_ai.rs:159-163] — Degenerate-up guard for `look_at` (template for `spread_forwards` collinear-up guard in AC #9)
- [Source: src/flight/mod.rs:91-148] — `spawn_player_ship` (restructured by AC #6 + #7)
- [Source: src/flight/input.rs:6-37] — `FlightAction` enum + Q/E roll bindings (informs Q-conflict avoidance)
- [Source: src/tuning/config.rs:15-49, :111-113] — Per-field `#[serde(default = "...")]` pattern (template for AC #1)
- [Memory: feedback_full_build_output.md] — `cargo check` exit-0 + tail is NOT proof; grep for `warning:|error:` (governs AC #15)
- [Memory: feedback_compact_review_style.md] — Compact single-line review answers (governs review interaction style)
- [Memory: feedback_scope_focus.md] — Core quality + learning leverage; archetype dispatch via Component+exhaustive-match is the core ECS learning hook
- [Memory: feedback_bmad_commit_order.md] — feat+bmad commits + push happen AFTER code review, not before

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Opus 4.7, 1M context) — bmad-dev-story workflow

### Debug Log References

- `cargo check` → `/tmp/story-4-4-check.log` (0 warning/error lines)
- `cargo build` → `/tmp/story-4-4-build.log` (0 warning/error lines)
- `cargo test` → `/tmp/story-4-4-test.log` (0 warning/error lines; `test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`)
- `cargo clippy --all-targets -- -D warnings` → `/tmp/story-4-4-clippy.log` (0 warning/error lines)
- `cargo fmt --all -- --check` → `/tmp/story-4-4-fmt.log` (0 warning/error lines, exit 0)
- `cargo build --release` → `/tmp/story-4-4-release.log` (0 warning/error lines)

### Completion Notes List

- ✅ **All 18 ACs implemented per spec** with two minor deviations documented below.
- ✅ **71 tests passing** (63 baseline + 8 net new in `combat::weapons::tests`: variants_distinct, stats_from_correct, loadout_default_three_full_slots, cycle_next_wraps, cycle_next_skips_empty, set_active_ignores_empty_and_oob, spread_count_one, spread_count_five_symmetric).
- ✅ **All 6 cargo gates green** per `feedback_full_build_output.md` discipline (full-output capture + `grep -cE 'warning:|error:'` = 0 for all 6 logs).
- ✅ **Two deferred-work entries closed** (line 279 projectile-damage-hardcoded; line 289 zero-damage-silent-despawn). Resolution notes appended in place per `4-3` precedent.
- ✅ **New deferred-work entry added** under "Observed during: 4-4-weapon-archetype-system dev (2026-05-11)" — escalation of pre-existing per-shot mesh+material allocation severity (Shotgun 5×-rate). Same resolution path as parent entry at line 246; same Epic 10 polish-pass scope.

- ⚠️ **Deviation from AC #4 code-shape (leafwing API):** AC #4 prescribed `InputMap::new(<empty>).insert(...).insert(...).build()`. Actual leafwing-input-manager 0.20 API has `InputMap::new(iter)` requiring same-typed buttons, OR `InputMap::default().with(action, button)` builder chain returning Self. Switched to the latter (verified at `~/.cargo/registry/.../leafwing-input-manager-0.20.0/src/input_map.rs:161`). Semantics identical: 1 mouse-button + 4 keyboard-key bindings, all wired. No `.build()` step needed (`.with` returns Self directly).
- ⚠️ **Deviation from AC #10 code-shape (Single<> vs Query<>):** AC #10 prescribed `Single<&ActionState<CombatAction>, With<PlayerShip>>` separate from the loadout query. Refactored to a single combined `Query<(&mut WeaponLoadout, &ActionState<CombatAction>), With<PlayerShip>>` — equivalent semantics for the player-only single-entity case, avoids any potential `Single<>`-vs-`Query<>` borrow-graph complexity, mirrors the `fire_primary_weapon` query shape. Cycle/select systems still run once per tick with just_pressed semantics; behavior identical.

- 📌 **Clippy `collapsible_if` fix during fmt/clippy sweep:** The original `if prev != ... { if let Some(now_active) = ... }` nested form in `select_active_weapon` was flagged by clippy's `collapsible-if` lint (denied by `-D warnings`). Refactored to the let-chain form `if prev != ... && let Some(now_active) = ...` which compiles on stable Rust (let-chain stabilized in 1.88; project pins Rust 1.94.1). Functionally equivalent.
- 📌 **`tuning.projectile_fire_rate_hz` unused by player path post-4.4:** Field is still read by `enemy_fire_weapon`'s cooldown calculation? NO — verified via grep — `enemy_fire_weapon` reads `tuning.enemy_fire_rate_hz`. So `projectile_fire_rate_hz` is now READ only by 3 tests. Added `#[allow(dead_code, reason = "...")]` per AC #1's preservation directive (kept for hot-reload-during-development convenience and forward-compat with pre-4.4 tuning.ron). A future cleanup story may remove.
- 📌 **PlayerShip spawn-tuple arity now exactly 16** via nested 3-subtuple grouping (visual:6 + physics:5 + state:5). Future stories adding a 17th component will need to re-evaluate the subtuple split.

- ✅ **Task 11 (Runtime smoke) executed by Till on 2026-05-11; all 9 scenarios green.** Scenarios (a)–(g) and (i) passed first walkthrough; scenario (h) initially failed (1.5× sphere-radius not visibly distinguishable at Railgun's 300 m/s speed from cockpit perspective). Post-smoke fix landed in same dev session: Railgun projectile mesh+collider switched from `Sphere::new(0.3)` to `Capsule3d::new(0.2, 1.0)` (lance shape) with `Quat::from_rotation_arc(Vec3::Y, direction)` rotation to align long axis with flight direction. Re-smoke (h) green after fix. All 6 cargo gates remain green post-fix.

### File List

**M (modified) — implementation:**
- `src/tuning/config.rs` — add `WeaponArchetypeStats` struct + 3 archetype-stat fields (`weapon_pulse`/`weapon_shotgun`/`weapon_railgun`) with per-field serde-default annotations; add 3 default fns; extend `TuningConfig::default()`; `#[allow(dead_code, reason)]` on `projectile_fire_rate_hz`; +45 assertions across the 3 existing tuning tests
- `src/combat/input.rs` — extend `CombatAction` enum with `CycleWeapon`, `SelectSlot1`, `SelectSlot2`, `SelectSlot3`; rewrite `default_input_map` to leafwing 0.20 `InputMap::default().with(...).with(...)` chain pattern
- `src/combat/projectiles.rs` — refactor `fire_primary_weapon` body: extend query with `&WeaponLoadout`, archetype-dispatch via `loadout.active()` + `archetype.stats_from(&tuning)`, zero-damage / zero-projectile-count warn-and-continue guards (closes deferred-work :289), spread-fan loop via `spread_forwards`, per-archetype mesh+collider+rotation tuple (Pulse/Shotgun = 0.2 m sphere, Railgun = 0.2 m × 1.0 m capsule rotated along flight axis via `Quat::from_rotation_arc(Vec3::Y, direction)`), `Projectile.damage = stats.damage` (closes deferred-work :279), per-trigger info log (was per-projectile); new `RAILGUN_CAPSULE_RADIUS` + `RAILGUN_CAPSULE_LENGTH` consts; new `crate::combat::weapons::{WeaponArchetype, WeaponLoadout, spread_forwards}` imports
- `src/combat/mod.rs` — `pub mod weapons;` declaration; replace standalone `projectiles::fire_primary_weapon` Fire-set registration with 3-system `.chain()` of `(weapons::cycle_active_weapon, weapons::select_active_weapon, projectiles::fire_primary_weapon)` so cycle/select apply before fire on the same tick
- `src/combat/components.rs` — extend `PrimaryWeaponCooldown` doc-comment to document shared-cooldown semantics across archetypes (cycling does NOT reset cooldown); struct shape unchanged
- `src/flight/mod.rs` — restructure PlayerShip spawn-tuple into 3-subtuple nested group (visual:6 + physics:5 + state:5); add `WeaponLoadout::default()` to state subtuple; add `crate::combat::weapons::WeaponLoadout` import
- `assets/config/tuning.ron` — add 3 nested `weapon_pulse` / `weapon_shotgun` / `weapon_railgun` fields with canonical default values

**?? (added) — implementation:**
- `src/combat/weapons.rs` — NEW FILE: `WeaponArchetype` enum (3 variants: Pulse/Shotgun/Railgun) + `stats_from` exhaustive-match dispatch; `WeaponLoadout` component (`[Option<WeaponArchetype>; 3]` slots + `active_slot` index) with `Default` impl (`[Pulse, Shotgun, Railgun]` at slot 0), `active`, `cycle_next` (skips-empty, wraps), `set_active` (OOB-bounded, empty-slot-noop) methods; `spread_forwards` pure helper (deterministic linspace fan, count==0/1/collinear-up edge cases); `cycle_active_weapon` + `select_active_weapon` FixedUpdate systems with just-pressed semantics; 8 tests

**M (modified) — bookkeeping:**
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `4-4-...: ready-for-dev → in-progress → review`; `last_updated` bumped
- `_bmad-output/implementation-artifacts/deferred-work.md` — close entries 279 + 289 with `✅ CLOSED 2026-05-11 by Story 4.4` notes; add new entry under "Observed during: 4-4-weapon-archetype-system dev (2026-05-11)" escalating per-shot mesh allocation severity for Shotgun's 5× rate
- `_bmad-output/implementation-artifacts/4-4-weapon-archetype-system-shotgun-railgun-archetypes.md` — flip `Status: ready-for-dev → in-progress → review`; check off Tasks 1–10, 12 (Task 11 deferred to user); fill Dev Agent Record sections; add Change Log entry

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-05-11 | bmad-create-story (auto) | Initial story creation; status: ready-for-dev. Notable design decisions in ACs: (a) Tab binding for CycleWeapon (NOT Q per epic-spec line 132 — Q collides with FlightAction::RollLeft); (b) `WeaponArchetypeStats` substruct in `TuningConfig` (vs flat per-stat fields) — nested-RON pattern, first use of struct-typed serde-default in this project; (c) shared `PrimaryWeaponCooldown` across archetypes (NOT per-archetype) — prevents hot-swap-fire bypass; (d) PlayerShip spawn-tuple restructure to 3-subtuple nest (Bevy 0.18 Bundle arity-15 cap; matches enemy.rs precedent); (e) closes deferred-work.md:279 (damage hardcoded) + :289 (zero-damage guard); escalates :246 (mesh allocation churn) without closing. |
| 2026-05-11 | bmad-dev-story (Opus 4.7) | Implemented all 18 ACs; 71 tests passing (63 baseline + 8 net new); all 6 cargo gates green; 2 deferred-work entries closed; new escalation entry added; status → review. Deviations from spec documented in Completion Notes: (a) leafwing 0.20 API uses `InputMap::default().with(...)` chain (AC #4 prescribed `.insert().build()` which doesn't match 0.20's surface); (b) cycle/select systems use combined `Query<>` instead of separate `Single<>` (AC #10 prescription) — equivalent semantics for the player-only case; (c) `select_active_weapon` uses let-chain `if a && let Some(b) = ...` form (clippy `collapsible_if` fix during gate sweep); (d) `tuning.projectile_fire_rate_hz` carries `#[allow(dead_code, reason)]` per AC #1's preservation directive. PlayerShip spawn-tuple arity now 16 (visual:6 + physics:5 + state:5 nested grouping). Task 11 runtime smoke deferred to Till's manual verification per AC #17. |
| 2026-05-11 | bmad-dev-story (Opus 4.7) | **Runtime-smoke (h) fix:** Till's smoke walkthrough surfaced that the original 1.5× sphere-radius differentiation (Railgun = 0.3 m vs Pulse/Shotgun = 0.2 m) was not visibly distinguishable from the cockpit at the Railgun's 300 m/s speed. Replaced sphere-radius differentiation with capsule-lance shape: Railgun now spawns a 0.2 m × 1.0 m `Capsule3d` mesh + matching `Collider::capsule`, rotated so the capsule's Y-axis aligns with the flight direction via `Quat::from_rotation_arc(Vec3::Y, direction)`. Spawn-clearance margin stays positive (0.3 m). Pulse/Shotgun unchanged (0.2 m sphere). All 6 cargo gates re-run, still green; 71 tests pass. AC #17(h) text updated to reflect the capsule design. |

### Review Findings

- [x] [Review][Patch] `damage==0`/`count==0` guard does not reset cooldown — `warn!` fires every FixedUpdate tick while FirePrimary held with a zero-stats archetype (cooldown stays at 0, gate re-passes every tick) [`src/combat/projectiles.rs:226-236`] **FIXED 2026-05-11:** added `cooldown.remaining = 1.0 / stats.fire_rate_hz.max(f32::EPSILON)` before each guard's `continue`.
- [x] [Review][Patch] `spread_forwards` fallback rotation axis `Vec3::Y` is degenerate when `forward ≈ Vec3::Y` — all Shotgun pellets fire in the same direction (reachable via 6DOF flight pointing straight up/down) [`src/combat/weapons.rs:119`] **FIXED 2026-05-11:** replaced `Vec3::Y` with cross-product perpendicular (`forward.cross(Vec3::X)` or `.cross(Vec3::Z)` fallback).
- [x] [Review][Patch] `tuning.ron` has no comment explaining that `projectile_fire_rate_hz` is superseded by per-archetype `weapon_*.fire_rate_hz` — latent tuner confusion [`assets/config/tuning.ron`] **FIXED 2026-05-11:** added two-line RON comment above `projectile_fire_rate_hz`.
- [x] [Review][Defer] No upper bound on `stats.projectile_count` — corrupted `tuning.ron` with large value triggers entity storm per-tick [`src/combat/projectiles.rs`] — deferred, tuning-input validation explicitly deferred per spec AC #18
- [x] [Review][Defer] Simultaneous Tab+digit or digit+digit input: undocumented but deterministic slot-selection behavior [`src/combat/weapons.rs:135-179`] — deferred, non-crashing edge case; behavior is implementation-defined
- [x] [Review][Defer] `spread_forwards` with `forward==Vec3::ZERO` and `up==Vec3::ZERO` produces NaN rotation [`src/combat/weapons.rs:106-130`] — deferred, unreachable via Avian `RigidBody::Dynamic` physics
- [x] [Review][Defer] Negative `fire_rate_hz` in `tuning.ron` freezes weapon at ~8.5×10⁶s cooldown with no log [`src/tuning/config.rs:71`, `src/combat/projectiles.rs:192`] — deferred, tuning-input validation explicitly deferred per spec AC #18
- [x] [Review][Defer] `cycle_next` and `set_active` hardcode literal `3` instead of `self.slots.len()` [`src/combat/weapons.rs:77-91`] — deferred, 3-slot design is fixed by type; no current bug
- [x] [Review][Defer] `WeaponLoadout::active_slot` and `slots` are fully public with no setter-enforced invariants [`src/combat/weapons.rs:48-49`] — deferred, intentional pub design for Epic 7 forward-compat per spec
- [x] [Review][Defer] `info!` log reports `stats.projectile_count` (tuning value) not `directions.len()` (actual spawn count) [`src/combat/projectiles.rs:309`] — deferred, values are equal in current code; theoretical divergence on future `spread_forwards` changes
