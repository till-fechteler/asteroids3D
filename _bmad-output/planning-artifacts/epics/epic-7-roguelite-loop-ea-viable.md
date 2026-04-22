# Epic 7: Roguelite Loop (EA-Viable)

Meta-currency from runs spendable in an unlock shop for 8 permanent upgrades. "One more run" retention loop closed. Intel x86_64 macOS binary added alongside arm64 (universal, still unsigned — FR48 further deferred to E10 per Till 2026-04-22). Commercially viable as Itch.io release or Steam EA if pursued. M-alignment: M6 🏁. FRs covered: FR18, FR19, FR20, FR21, FR47 completion. NFR covered: NFR-R4.

## Story 7.1: Meta-Currency Display on MainMenu + PostRun `banked` Real Wiring

As a player,
I want to see my lifetime meta-currency on MainMenu and per-run banked amount on PostRun,
So that FR19 persistent progression is visible and Story 4.9's PostRun placeholder becomes real.

**Acceptance Criteria:**

**Given** SaveData.meta_currency exists from Story 4.6 (banked by Story 6.7)
**When** MainMenu renders (Story 4.7)
**Then** a "META: <meta_currency>" text Node is added to MainMenu layout (top-right or above title)
**And** the text updates reactively when SaveData.meta_currency changes

**Given** Story 4.9's PostRun shows "Salvage banked: 0" placeholder
**When** Story 7.1 wires the real values
**Then** PostRun summary reads:
- "Salvage banked this run: `<RunResult.salvage_banked>`"
- "Meta-currency total: `<SaveData.meta_currency>`"

**Given** Epic 6 Story 6.7 banks salvage
**When** Story 7.1 verifies the cross-story contract
**Then** `RunResult.salvage_banked` is populated on ALL run-end paths (TargetReached, HullDepleted, and Aborted with salvage_banked=0 for forfeit)

**Given** first launch (no runs completed)
**When** MainMenu is shown
**Then** display reads "META: 0" (always visible for consistency)

## Story 7.2: Unlock Definition Data Model + 8-Unlock Catalog

As a developer,
I want a data model for unlocks and a catalog of 8 definitions with exponential stacking costs,
So that FR21 has concrete purchasable content.

**Acceptance Criteria:**

**Given** `src/meta/unlocks.rs` is authored
**When** the data model is defined
**Then**:
- `UnlockDefinition { id: String, display_name: String, description: String, effect: UnlockEffect, stackable_max: u32, base_cost: u32 }`
- `UnlockEffect` enum: `HullMaxDelta(i32)`, `ShieldMaxDelta(i32)`, `ThrustMult(f32)`, `DetectionRangeMult(f32)`, `BoostRechargeMult(f32)`, `TractorRangeDelta(f32)`, `ShotCostMult(f32)`, `YieldCapturedMult(f32)`
- `stackable_max=1` = non-stackable; higher = cap

**Given** `src/meta/catalog.rs` is authored
**When** the catalog is defined
**Then** these 8 entries exist:
- `hull_plating` — `HullMaxDelta(1)` — max=3 — base=100
- `shield_capacitor` — `ShieldMaxDelta(2)` — max=3 — base=150
- `thruster_tuning` — `ThrustMult(1.2)` — max=1 — base=200
- `sensor_range` — `DetectionRangeMult(1.5)` — max=1 — base=250 (consumed by Epic 8 radar)
- `boost_recharge` — `BoostRechargeMult(1.5)` — max=1 — base=200
- `tractor_reach` — `TractorRangeDelta(20.0)` — max=2 — base=300
- `weapon_efficiency` — `ShotCostMult(0.8)` — max=1 — base=350
- `salvage_refinery` — `YieldCapturedMult(1.1)` — max=1 — base=400

**Given** exponential 1.5× stacking per Till's decision
**When** `cost_for_next_stack(&unlock_def, current_stacks: u32) -> u32` is called
**Then** returns `(base_cost as f32 * 1.5_f32.powi(current_stacks as i32)).round() as u32`
**And** returns `Err(MaxStacked)` when `current_stacks >= stackable_max`

**Given** SaveData.unlocked_upgrades was `Vec<String>` from Story 4.6
**When** Story 7.2 refactors representation
**Then** it becomes `HashMap<String, u32>` (id → stack_count) for cleaner stacking
**And** a `v2_to_v3` migration via Story 5.6's scaffold converts existing `Vec<String>` by counting duplicates
**And** SaveData.version bumps to 3

## Story 7.3: UnlockShop UI State + Access from MainMenu + PostRun

As a player,
I want to visit the UnlockShop from MainMenu or PostRun,
So that FR20 spend-between-runs and immediate-retention loops both work.

**Acceptance Criteria:**

**Given** `GameState` enum
**When** extended
**Then** `UnlockShop` variant is added

**Given** Story 4.7 MainMenu and Story 4.9 PostRun
**When** Story 7.3 extends both layouts
**Then** each adds a "Shop" button (MainMenu: placed between Settings and Credits; PostRun: alongside Retry and Main Menu)
**And** clicking either sets `NextState<GameState>` = `UnlockShop`
**And** a `ShopReturnTo(GameState)` resource is set to the originating state on entry

**Given** `src/ui/shop.rs` is authored
**When** OnEnter(UnlockShop) runs
**Then** UI shows:
- Header: "META: `<meta_currency>`" (reactive)
- Scrollable list of all 8 catalog entries
- For each entry: display_name, description, `current_stacks / stackable_max`, next_cost (or "MAX"), "Buy" button (disabled if insufficient funds or at max)
- "Back" button routes via `ShopReturnTo`

**And** all spawned entities carry a `ShopEntity` marker

**Given** OnExit(UnlockShop)
**When** cleanup runs
**Then** `ShopEntity`-marked entities are despawned
**And** `ShopReturnTo` resource is removed

## Story 7.4: Purchase Flow — Validate + Deduct + Event + Save

As a player,
I want "Buy" to validate funds, deduct cost, apply the stack, and save,
So that FR20 transactions are atomic and persistent.

**Acceptance Criteria:**

**Given** the shop UI from Story 7.3
**When** the player clicks "Buy" for an unlock at cost C
**Then** the purchase system:
- Validates `SaveData.meta_currency >= C` (else abort silently, subtle cost-text flash)
- Validates `current_stacks < stackable_max` (else abort)
- Deducts `SaveData.meta_currency -= C`
- Increments stack count in `SaveData.unlocked_upgrades` HashMap
- Emits `UnlockPurchased { id: String, stack_after: u32, cost_paid: u32 }` event
- Calls `save(&save_data)`

**Given** the save write succeeds
**When** UI re-renders
**Then** META header updates reactively
**And** the purchased unlock's stack count and next cost update
**And** input debounce (100 ms cooldown or `Interaction::Pressed`-once) prevents double-purchase

**Given** the save write fails
**When** the purchase system detects the error
**Then** in-memory SaveData changes are rolled back (currency and stack reverted)
**And** an inline error "Purchase failed: could not save. Try again." is shown
**And** no `UnlockPurchased` event is emitted

**Given** the `UnlockPurchased` event is emitted
**When** Story 7.5's system observes
**Then** downstream effects-wiring fires per Story 7.5

## Story 7.5: Unlock Effects Wiring — Runtime TuningConfig Overlay

As a player,
I want my purchased unlocks to actually improve my ship in the next run,
So that FR21 upgrades have tangible gameplay effect.

**Acceptance Criteria:**

**Given** SaveData.unlocked_upgrades (HashMap<String, u32>) from Story 7.2
**When** OnEnter(Caravan) OR OnEnter(Arena) fires
**Then** an `apply_unlock_effects` system runs that builds a `RuntimeTuning` resource from `TuningConfig` + unlock overlays:
- For each `(id, stacks)`, look up effect, apply N times:
  - `HullMaxDelta(d)` × N → `effective_player_hull_max = base + d*N`
  - `ShieldMaxDelta(d)` × N → similarly
  - `ThrustMult(m)` → `effective_ship_thrust_newtons = base * m^N`
  - `DetectionRangeMult(m)` → `effective_enemy_detection_range = base * m^N`
  - `BoostRechargeMult(m)` → `effective_boost_recharge_rate = base * m^N`
  - `TractorRangeDelta(d)` → `effective_tractor_range = base + d*N`
  - `ShotCostMult(m)` → `effective_shot_cost = max(1, (base as f32 * m.powi(N)).floor() as u32)`
  - `YieldCapturedMult(m)` → `effective_yield_captured_<size> = (base as f32 * m.powi(N)).floor() as u32`

**Given** gameplay systems previously read `TuningConfig` directly
**When** Story 7.5 is merged
**Then** all ship/enemy/tractor/shot/yield consumers are updated to read from `RuntimeTuning` instead
**And** `RuntimeTuning` rebuilds on each state-entry (newly-purchased unlocks take effect next run, not mid-run)

**Given** no unlocks purchased
**When** `RuntimeTuning` is built
**Then** it exactly matches `TuningConfig` (identity operation — regression safety)

**Given** the Retry loop (PostRun → Shop → Back → Retry)
**When** a new Caravan run starts
**Then** the newly-purchased unlock is reflected (verified via playtest)

**Given** `hull_plating` purchased 3 times (100 + 150 + 225 = 475 salvage)
**When** next run starts
**Then** `effective_player_hull_max = 3 + 1*3 = 6`
**And** 4th purchase attempt is rejected (at stackable_max=3)

## Story 7.6: macOS Universal Binary — Intel x86_64 + arm64

As a player on an Intel Mac,
I want a native-speed macOS binary,
So that FR47's Apple Silicon + Intel x86_64 commitment is fulfilled without emulation.

**Acceptance Criteria:**

**Given** Epic 4 Story 4.10's release workflow produces `macos-arm64` only
**When** Story 7.6 extends the macOS job
**Then** the workflow adds `rustup target add x86_64-apple-darwin` + a second `cargo build --release --target x86_64-apple-darwin` step

**Given** both architectures are built
**When** the universal-binary step runs
**Then** `lipo -create -output asteriods3D target/release/asteriods3D target/x86_64-apple-darwin/release/asteriods3D` produces a combined Mach-O
**And** `lipo -info asteriods3D` reports both architectures in the CI log
**And** a CI check asserts both slices are present

**Given** the universal binary is staged
**When** packaging runs
**Then** the ZIP is renamed to `asteriods3D-macos-universal-<version>.zip` (replaces prior `macos-arm64` ZIP)

**Given** both Intel and arm64 Mac users download the ZIP
**When** they launch
**Then** each CPU runs its native slice at native performance
**And** no Rosetta emulation occurs on either architecture (verified via Activity Monitor or `file` output check)

**Given** the binary is still unsigned per Till's decision (FR48 deferred E7 → E10 / M9 Polish)
**When** users run it
**Then** the right-click-Open Gatekeeper workaround from Story 4.10's runbook still applies
**And** the runbook is updated: "universal binary, unsigned; signing is Epic 10 / M9"

<!-- Epic 7 complete — 6 stories deliver M6 Roguelite Loop + macOS universal binary. FR48 further deferred to Epic 10. Next epic to decompose: Epic 8 (Perception — Sensors & Spatial Audio / M7). -->
