# Epic 5: Ship Subsystem State & Formal Save Schema

Formal Hull + Shields subsystems with regen mechanics. Shields regenerate after cooldown; Hull does not regen mid-run. Ship state readable at-a-glance. Save schema formalized with versioning for post-MVP expansion. Decoupled aim reticle system. M-alignment: M4. FRs covered: FR4, FR15. NFRs covered: NFR-R3, NFR-R4, NFR-U2, NFR-U3.

## Story 5.1: Formal HullHp + ShieldHp Components (Refactor from Health)

As a developer,
I want formal `HullHp` and `ShieldHp` components on PlayerShip, replacing the unified `Health` introduced in Epic 4,
So that FR15's distinct subsystem regen behaviours have a clean component boundary and asteroids/enemies can keep the simple `Health` model unchanged.

**Acceptance Criteria:**

**Given** `src/combat/health.rs` has a unified `Health { current: u32, max: u32 }` from Story 4.2
**When** Story 5.1 is implemented
**Then** two new components are added to the same module:
- `HullHp { current: u32, max: u32 }` — no regen
- `ShieldHp { current: u32, max: u32, regen_rate_per_sec: f32, regen_cooldown_sec: f32, last_damage_time_sec: f32 }`

**And** `Health` remains for asteroids and enemies

**Given** PlayerShip from Story 4.3 currently carries `Health`
**When** PlayerShip is updated
**Then** `Health` is removed from PlayerShip
**And** `HullHp { current: 3, max: 3 }` is added, sourced from `TuningConfig.player_hull_max: u32 = 3`
**And** `ShieldHp { current: 5, max: 5, regen_rate_per_sec: 1.0, regen_cooldown_sec: 3.0, last_damage_time_sec: 0.0 }` is added, sourced from `TuningConfig.player_shield_*`

**Given** the refactor is complete
**When** the compiler runs
**Then** no references to PlayerShip `Health` remain
**And** all prior damage-routing systems compile (routing logic is re-wired in Story 5.3)

**Given** asteroids and enemies from Epics 3–4
**When** they spawn
**Then** they continue to carry `Health` (shields/hull are player-only in Epic 5)

## Story 5.2: Shield Regen System + Damage Cooldown Tracking

As a player,
I want my shield to regenerate after a cooldown when I avoid damage,
So that I have a renewable defence layer per FR15, incentivizing tactical combat pacing.

**Acceptance Criteria:**

**Given** `src/combat/shield_regen.rs` is authored
**When** a regen system is registered in `FixedUpdate` within `CombatSystems`
**Then** for each entity with `ShieldHp`:
- If `shield.current < shield.max` AND `time.elapsed_sec - shield.last_damage_time_sec >= shield.regen_cooldown_sec`
- Then `shield.current += shield.regen_rate_per_sec * time.delta_sec()`, clamped to `shield.max`

**Given** the player takes shield damage (site is Story 5.3)
**When** the damage-routing system updates `shield.current`
**Then** it also updates `shield.last_damage_time_sec = time.elapsed_sec` (cross-story contract)

**Given** the player is undamaged for `regen_cooldown_sec` seconds
**When** the regen system runs each frame
**Then** `shield.current` increases at `regen_rate_per_sec` toward `shield.max`
**And** no regen occurs during the cooldown window

**Given** regen would push `current` above `max` in a single tick
**When** the clamp applies
**Then** `shield.current` saturates at `shield.max` (no overshoot)

## Story 5.3: Damage Routing — Shields Absorb First, Spill to Hull

As a player,
I want incoming damage to deplete my shields first, with spillover hitting my hull,
So that shields meaningfully protect me per FR15 and permadeath only triggers on hull-zero.

**Acceptance Criteria:**

**Given** Story 4.3's `ProjectileHitPlayer` event
**When** Story 5.3 updates the damage-application system
**Then** the system reads `ShieldHp` and `HullHp` on PlayerShip and applies damage in sequence:
- `shield_damage = min(incoming, shield.current)`
- `shield.current -= shield_damage`
- `remaining = incoming - shield_damage`
- `hull_damage = min(remaining, hull.current)`
- `hull.current -= hull_damage`

**And** `shield.last_damage_time_sec = time.elapsed_sec` is set iff `shield_damage > 0`

**Given** damage is routed
**When** the system emits events
**Then** a `DamageApplied { entity: Entity, shield_damage: u32, hull_damage: u32 }` event is emitted

**Given** `HullHp.current` reaches 0
**When** the permadeath system from Story 4.3 observes
**Then** `HullDepleted` fires and transitions to `PostRun` (behaviour unchanged)

**Given** a 1-damage projectile hits a player with `shield.current = 5`, `hull.current = 3`
**When** damage resolves
**Then** `shield.current = 4`, `hull.current = 3`

**Given** a 5-damage projectile hits a player with `shield.current = 2`, `hull.current = 3`
**When** damage resolves
**Then** `shield.current = 0`, `hull.current = 0` (overkill) and `HullDepleted` fires

## Story 5.4: HUD Wiring for Shields + Hull — Instrument-Panel Styling

As a player,
I want shield and hull state as color-coded bars in my peripheral vision,
So that I can read ship status mid-combat without looking away from targets per NFR-U2 and NFR-U3.

**Acceptance Criteria:**

**Given** Story 3.11's HUD placeholders and Story 4.3's live-Hull text
**When** Story 5.4 replaces Shield + Hull fields
**Then** the placeholder texts are replaced with bar visuals
**And** each bar is a `bevy_ui` Node with a fill child sized via `Style.width = Val::Percent(current / max * 100)`

**Given** instrument-panel styling per architecture
**When** bars render
**Then**:
- Shield bar: cyan (`SemanticAccent::PlayerOwned`), subtle tick-mark decoration
- Hull bar: orange-red (`SemanticAccent::Hazard` or dedicated hull color), urgency palette
- Both bars have a thin toon-outline border
- Numeric value ("3 / 3") overlaid in small text adjacent

**Given** NFR-U2 (simultaneous visibility)
**When** PlayerShip is in Arena
**Then** both bars are visible without overlap or occlusion from ammo/salvage fields
**And** bars sit in instrument-panel corner layout (shield top-left, hull top-right)

**Given** NFR-U3 (at-a-glance)
**When** shield is 5/5 vs 2/5 vs 0/5
**Then** fill-percentage is visibly different under a 1-second glance from 60–80 cm at 1080p
**And** a playtest check confirms testers report ship state correctly in >90% of 1-second glances at random HP values

**Given** shields deplete to 0
**When** the bar hits empty
**Then** a subtle "shield-offline" visual cue triggers (brief flash or red warning chevron adjacent to bar) — audio cue is Epic 8

**Given** shields are regenerating
**When** the bar updates each frame
**Then** fill animates smoothly (no visible quantized jumps at default `regen_rate_per_sec = 1.0`)

## Story 5.5: Decoupled Aim Reticle — Hold-to-Enable

As a player,
I want to aim my weapons independently of ship heading by holding Right Mouse Button,
So that I can attack targets at oblique angles without realigning the whole ship per FR4.

**Acceptance Criteria:**

**Given** `CombatAction` from Story 3.9
**When** extended
**Then** `HoldDecoupledAim` is added bound to Right Mouse Button (hold-to-enable)

**Given** `src/combat/aim.rs` is authored
**When** `AimDirection { world_vector: Vec3 }` is defined on PlayerShip
**Then** `AimDirection` default-initializes to `ship_forward_vector` on spawn
**And** when `HoldDecoupledAim` transitions released→held, `AimDirection` is reset to `ship_forward_vector` (decoupled aim starts centered on current facing)

**Given** `HoldDecoupledAim` is held
**When** mouse-delta arrives
**Then** Story 3.7's ship-rotation pitch/yaw is suspended for that frame (no ExternalTorque on pitch/yaw)
**And** mouse deltas rotate `AimDirection.world_vector` around ship-local pitch/yaw axes at the same sensitivity as ship rotation
**And** roll (Q/E) continues to rotate the ship normally (roll is nonsensical for aim — decoupling is pitch/yaw only)

**Given** `HoldDecoupledAim` is released
**When** the frame advances
**Then** Story 3.7's ship-rotation input resumes
**And** `AimDirection` persists at its last decoupled value until the next hold

**Given** the weapon-firing system from Story 3.9 / 4.4
**When** the player fires with `HoldDecoupledAim` held
**Then** projectiles spawn with `LinearVelocity = ship_velocity + aim_direction * projectile_speed`
**And** with `HoldDecoupledAim` NOT held
**Then** projectiles spawn along `ship_forward_vector` as before

**Given** `HoldDecoupledAim` held AND aim within the camera frustum
**When** the reticle HUD system runs
**Then** a reticle UI element renders at the screen-space projection of `ship_position + aim_direction * 100 m`
**And** the reticle shape conveys the active WeaponArchetype (shotgun: wider cone hint; pulse: medium crosshair; railgun: tight precision crosshair)

**Given** `HoldDecoupledAim` held AND aim projects off-screen
**When** the edge-indicator system runs
**Then** an arrow/edge-marker renders on the nearest screen edge toward the off-screen aim
**And** the indicator disappears when aim re-enters the frustum

**Given** `HoldDecoupledAim` NOT held
**When** the reticle system runs
**Then** a static ship-forward crosshair renders at screen-center (simple, WeaponArchetype-agnostic)
**And** no edge indicator is shown

## Story 5.6: Save Schema Migration Scaffold + Missing-Save Recovery Prompt

As a developer and as a player,
I want a save-schema migration path plus graceful recovery prompts when the save is missing or corrupt,
So that future schema changes don't brick existing users per NFR-R3/NFR-R4 and players never lose progression silently.

**Acceptance Criteria:**

**Given** `SaveData.version: u32` is currently 1 from Story 4.6
**When** Story 5.6 is implemented
**Then** `src/persistence/migration.rs` is authored with `migrate(raw: serde_json::Value, found_version: u32) -> Result<SaveData, MigrationError>`
**And** the function dispatches to per-version handlers (`fn v0_to_v1(v: serde_json::Value) -> serde_json::Value`, etc.)
**And** `current_version = 1` — no production migration active in Epic 5; scaffold exists for future Epics

**Given** a `#[cfg(test)]` synthetic v0 fixture (missing `version` field or missing a later-added field)
**When** the migration chain runs against it
**Then** the fixture upgrades cleanly to valid `SaveData` at `current_version = 1`
**And** the test lives in `src/persistence/migration_tests.rs`

**Given** a save file whose `version` is higher than current
**When** the load system encounters it
**Then** `LoadError::VersionTooNew { found, current }` is returned
**And** the app refuses to auto-clobber (does not create default over it)
**And** the recovery prompt is triggered

**Given** a `.initialized` sentinel file in the save dir distinguishes first-launch from existing-user
**When** the app starts
**Then**:
- No save AND no sentinel → **first launch**: default save created silently, sentinel written, proceed to MainMenu (preserves Story 4.6 first-launch UX)
- No save AND sentinel exists → **save-missing recovery prompt**
- Save unreadable (malformed JSON) → **save-corrupt recovery prompt**
- Save version-too-new → **version-too-new recovery prompt**

**Given** a recovery prompt is triggered
**When** `OnEnter(GameState::Loading)` proceeds
**Then** a modal `bevy_ui` Node shows: human-readable cause, a Yes button (accept default-save with "meta progression lost" warning), a Quit button (`AppExit::Success` no disk write)
**And** Yes:
- Missing-save: creates default, proceeds to MainMenu
- Corrupt / version-too-new: backs up the broken save to `<path>.corrupt-<UTC-timestamp>` first, then creates default, then proceeds

**And** Quit exits cleanly with no disk side-effects

**Given** a sentinel-write is interrupted
**When** the app relaunches
**Then** absence of sentinel is safe — worst case is first-launch UX shown a second time; no data loss beyond what the interrupted run had written

<!-- Epic 5 complete — 6 stories cover M4 subsystem formalization + save migration scaffold. Next epic to decompose: Epic 6 (Caravan Run Framework / M5). -->
