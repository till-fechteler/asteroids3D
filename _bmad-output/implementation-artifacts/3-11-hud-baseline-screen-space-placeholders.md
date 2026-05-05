# Story 3.11: HUD Baseline (Screen-Space Placeholders)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a player flying the Arena cockpit ship,
I want a HUD showing placeholder shields, hull, ammo, and salvage values in the four screen corners,
So that I learn where to look for tactical state before real values arrive in Epic 5 (Hull/Shield) and Epic 6 (Salvage currency) — closing Epic 3 / M2 First Playable per FR24.

## Acceptance Criteria

1. **Given** Bevy 0.18.1's `bevy_ui` UI tree must target a camera, and the default-UI-camera fallback algorithm at `bevy_ui-0.18.1/src/ui_node.rs:2927-2946` resolves to either (a) a camera carrying `IsDefaultUiCamera`, or (b) `cameras.iter().max_by_key(|(e, c, _)| (c.order, *e))` on the primary-window-targeting set
   **When** Story 3.11 introduces HUD UI nodes
   **Then** **NO** new `Camera2d` is spawned for the HUD; **NO** `IsDefaultUiCamera` marker is added; HUD nodes carry **NO** `UiTargetCamera` component
   **And** the cockpit `Camera3d` (spawned by `spawn_player_ship` at `src/flight/mod.rs:132-136`, implicit `order: 0`) IS the default UI camera target during Arena gameplay because it is the only camera entity targeting the primary window in `GameState::Arena`
   **And** when the `PausePlugin` overlay activates (Camera2d order: 1 at `src/pause/mod.rs:148-157`), Bevy's max-by-order fallback transparently re-targets HUD nodes to the pause Camera2d for the duration of `GameState::Paused`; HUD remains visually present (acceptable placeholder behavior — pause's BLACK clear obscures the 3D scene but HUD text continues to render on top, consistent with real-game pause overlays). NO code change required to support this; it is the algorithmic consequence of NOT setting `IsDefaultUiCamera` anywhere
   **And** the existing pause-overlay rendering is unchanged (pause overlay nodes also rely on the same default-UI-camera fallback; max-order tie between pause Camera2d and any future HUD Camera2d is **AVOIDED** by NOT introducing one)
   **Architecture rationale:** the alternative (spawn dedicated HUD Camera2d order: 1) collides with the pause Camera2d order: 1 slot — `cameras.max_by_key((order, entity))` would tie-break by Entity ID, producing nondeterministic UI camera target across runs. The deferred-work entry "Camera2d order:1 — no documented slot-reservation convention" (deferred-work.md:194) anticipates this exact collision; Story 3.11 SIDESTEPS the collision by not introducing a 2nd Camera2d. The deferred entry remains open for future Camera2d users (4.9 post-run screen, 9.2 photo-mode)

2. **Given** Story 3.5's `spawn_player_ship` (`src/flight/mod.rs:88-140`) and Story 3.3's `spawn_arena_zone` use `OnTransition { exited: MainMenu, entered: Arena }` (NOT `OnEnter(GameState::Arena)`) for setup, AND the existing cleanup at `src/arena/mod.rs:44-50` uses `OnTransition { exited: Arena, entered: MainMenu }` (NOT `OnExit(GameState::Arena)`), so the Arena ↔ Paused round-trip preserves all entities (`src/arena/mod.rs:1-6` documents this contract)
   **When** Story 3.11 wires HUD spawn + cleanup
   **Then** the `spawn_hud` system is registered on `OnTransition { exited: GameState::MainMenu, entered: GameState::Arena }` (NOT `OnEnter(GameState::Arena)` as the epic spec at `epics/epic-3-arena-flight-first-combat-first-playable.md:309` literally reads)
   **And** **NO** dedicated HUD-cleanup system is registered — HUD entities are dual-marked with both `HudEntity` AND `ArenaEntity`, so the existing `cleanup_on_exit::<ArenaEntity>` registered by `ArenaPlugin::build` (`src/arena/mod.rs:44-50`) on `OnTransition { exited: Arena, entered: MainMenu }` despawns HUD entities transitively (matches the `PlayerShip` + `ArenaEntity` dual-marker precedent at `src/flight/mod.rs:117-118`)
   **And** **NO** `OnEnter(GameState::Arena)` / `OnExit(GameState::Arena)` system is registered for HUD spawn/cleanup — using OnExit-based cleanup would despawn HUD on the Arena → Paused transition, breaking the round-trip preservation contract; this is the epic-spec-vs-implementation deviation — wording at `epics/epic-3-arena-flight-first-combat-first-playable.md:309,328` predates the OnTransition pattern that landed in Story 3.5 / 3.10. The deviation is documented in dev notes
   **And** Story 3.11 **DOES NOT** introduce a third direct consumer of `cleanup_on_exit::<T>` — HUD reuses ArenaEntity cleanup. The deferred-work entry "Generic-cleanup home re-evaluation now triggered (3rd consumer pending)" (deferred-work.md:186) is **UPDATED** by this story to note: "Story 3.11 chose dual-marker (HudEntity + ArenaEntity) over a 3rd direct cleanup_on_exit::<HudEntity> consumer; the entry stays open for the next direct consumer (likely 4.9 post-run screen or 9.2 photo-mode camera)"

3. **Given** architecture.md:592 places HUD source at `src/ui/hud.rs` AND existing `UiPlugin` at `src/ui/mod.rs:1-25` already follows a "submodule + plugin-extension" pattern (it currently registers `main_menu::spawn_main_menu` / `cleanup_main_menu` from the `main_menu` submodule)
   **When** Story 3.11 introduces the HUD module
   **Then** a new file `src/ui/hud.rs` is authored as a submodule; `pub mod hud;` is added to `src/ui/mod.rs` after the existing `pub mod main_menu;` line (alphabetical order: `hud`, `main_menu` — `hud` comes first)
   **And** **NO** separate `HudPlugin` struct/impl is introduced — the existing `UiPlugin::build` is extended to register HUD systems, matching the existing `main_menu` integration precedent. The epic-spec wording "`src/ui/hud.rs` is authored with a `HudPlugin`" at `epics/epic-3-arena-flight-first-combat-first-playable.md:308` is implementation-detail-leakage from the planning phase; the project's established pattern is "one Plugin per top-level feature module, submodules contribute systems/types" (see CombatPlugin owning `combat::damage` submodule from Story 3.10, FlightPlugin owning `flight::physics` + `flight::input` submodules)
   **And** the `UiPlugin::build` extension follows this exact additive structure (NEW lines marked `// 3.11`):
   ```rust
   impl Plugin for UiPlugin {
       fn build(&self, app: &mut App) {
           // [existing 3.1 main_menu lines unchanged]

           // 3.11: HUD spawn on MainMenu → Arena. Cleanup transitively via
           // cleanup_on_exit::<ArenaEntity> registered by ArenaPlugin (HUD
           // entities are dual-marked HudEntity + ArenaEntity).
           app.add_systems(
               OnTransition {
                   exited: crate::state::GameState::MainMenu,
                   entered: crate::state::GameState::Arena,
               },
               hud::spawn_hud,
           );
       }
   }
   ```
   **And** the existing `main_menu::spawn_main_menu` registration on `OnEnter(MainMenu)` is **NOT** modified (it is correctly OnEnter — MainMenu has no round-trip preservation concern)

4. **Given** the four placeholder fields per `epics/epic-3-arena-flight-first-combat-first-playable.md:314-321` are top-left "SHIELDS 100", top-right "HULL 100", bottom-left "AMMO ∞", bottom-right "SALVAGE 0", AND each must wire to a `HudPlaceholder { field: HudField }` component for later Epic-5/6 connection
   **When** the type vocabulary is authored at the top of `src/ui/hud.rs`
   **Then** the following types are defined, in this order:
   ```rust
   /// Marker for all HUD entities. Used both for granular HUD-only queries
   /// (Epic 5/6 placeholder-value updaters will Query<&mut Text, With<HudPlaceholder>>)
   /// and as a redundant safety marker — actual cleanup happens via the dual-marker
   /// ArenaEntity pattern + cleanup_on_exit::<ArenaEntity> in ArenaPlugin.
   #[derive(Component, Debug, Clone, Copy)]
   pub struct HudEntity;

   /// Identifies which tactical-state field a HUD text node represents. Wired up
   /// in Story 3.11 with static placeholder values; Epic 5 connects Shields/Hull
   /// to live ShieldHP/HullHP components; Epic 6 connects Salvage to the
   /// SalvageCurrency resource. Ammo remains "∞" through Epic 7 (pay-to-shoot
   /// economy replaces the ammo concept entirely per FR11).
   #[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
   pub enum HudField {
       Shields,
       Hull,
       Ammo,
       Salvage,
   }

   /// Companion component on each HUD value-text node. Future systems mutate
   /// the sibling Text component using HudField as the dispatch discriminant.
   /// Story 3.11 sets these once at spawn; Epic 5/6 will add update systems
   /// that re-write the Text content based on game state.
   #[derive(Component, Debug, Clone, Copy)]
   pub struct HudPlaceholder {
       pub field: HudField,
   }
   ```
   **And** **NO** `Default` derive on `HudField` (callers always specify the variant explicitly at spawn — same footgun-prevention rationale as `AsteroidHp` from Story 3.10 AC #4)
   **And** **NO** `Default` derive on `HudPlaceholder` (same reason — there is no semantically-correct default field)

5. **Given** the four corner labels + values are stable Epic-3 literals (string-table externalization is NFR-L3 work for Epic 4 per architecture.md:843, where the canonical `assets/strings/en.ron` lands; Epic 3 is too early — string table doesn't exist yet, confirmed by `ls assets/strings/` returning empty)
   **When** the placeholder content is captured as constants
   **Then** the following constants are defined at the top of `src/ui/hud.rs` after the `use` block AND before the type definitions:
   ```rust
   const SHIELDS_LABEL: &str = "SHIELDS 100";
   const HULL_LABEL: &str = "HULL 100";
   const AMMO_LABEL: &str = "AMMO ∞";
   const SALVAGE_LABEL: &str = "SALVAGE 0";

   const HUD_FONT_SIZE: f32 = 24.0;
   const HUD_CORNER_MARGIN_PX: f32 = 24.0;
   const HUD_TEXT_COLOR: Color = Color::srgb(0.85, 0.95, 1.0); // muted cyan-white per "scientific instrument panel" Design Philosophy
   ```
   **And** **EXACTLY** the strings above (no localization scaffolding, no f-string interpolation, no `format!`); FR28 "no tutorial text" is upheld trivially since these are tactical-state labels not tutorial text
   **And** the "∞" character is the literal Unicode U+221E INFINITY codepoint embedded in source — Bevy 0.18's default font (FiraSans-via-`default_font` feature, enabled in `Cargo.toml:8`) supports it. NO image-glyph fallback required
   **And** `HUD_TEXT_COLOR` uses `Color::srgb(0.85, 0.95, 1.0)` — a slightly cyan-tinted near-white that distinguishes the HUD from the pure-WHITE pause-overlay text (`src/pause/mod.rs:177`) and pure-WHITE main-menu title (`src/ui/main_menu.rs:39`); the visual distinction reinforces "scientific instrument panel" framing per architecture.md's Design Philosophy notes. NOT placed in `tuning.ron` because Epic 3 has no HUD-color tuning UX yet (post-MVP UI polish concern); reuse the constant when later HUD work needs the same accent
   **And** `HUD_FONT_SIZE: 24.0` is smaller than both the pause overlay (`48.0`, `src/pause/mod.rs:175`) and the MainMenu title (`96.0`, `src/ui/main_menu.rs:10`) — sized for at-a-glance reading per FR24 + NFR-A3, NOT for menu attention; NFR-A3 ("HUD legibility at 60–80 cm reading distance @ 1080p") is a playtest-validated audit concern in Story 10.5 — Story 3.11 establishes the placeholder slot, Story 10.5 tunes the size

6. **Given** `bevy_ui-0.18.1`'s flexbox-style layout uses `Node` with `position_type`, `top` / `left` / `right` / `bottom` `Val` fields for absolute positioning (per `bevy_ui-0.18.1/src/ui_node.rs:519-523, 1446-1453`), AND the four corner-anchored fields per the epic spec must NOT obstruct the central line of sight
   **When** the `spawn_hud` system is authored
   **Then** the system signature is exactly:
   ```rust
   pub fn spawn_hud(mut commands: Commands)
   ```
   **And** the system body spawns ONE root Node (full-window absolute, transparent, no children-blocking interaction surface) plus FOUR child Nodes (one per corner). The structural skeleton is:
   ```rust
   pub fn spawn_hud(mut commands: Commands) {
       commands
           .spawn((
               Node {
                   width: Val::Percent(100.0),
                   height: Val::Percent(100.0),
                   position_type: PositionType::Absolute,
                   ..default()
               },
               HudEntity,
               ArenaEntity, // dual-marker — see AC #2
           ))
           .with_children(|parent| {
               // top-left: SHIELDS
               parent.spawn((
                   Node {
                       position_type: PositionType::Absolute,
                       top: Val::Px(HUD_CORNER_MARGIN_PX),
                       left: Val::Px(HUD_CORNER_MARGIN_PX),
                       ..default()
                   },
                   HudEntity,
                   ArenaEntity,
                   HudPlaceholder { field: HudField::Shields },
                   Text::new(SHIELDS_LABEL),
                   TextFont { font_size: HUD_FONT_SIZE, ..default() },
                   TextColor(HUD_TEXT_COLOR),
               ));
               // top-right: HULL
               parent.spawn((
                   Node {
                       position_type: PositionType::Absolute,
                       top: Val::Px(HUD_CORNER_MARGIN_PX),
                       right: Val::Px(HUD_CORNER_MARGIN_PX),
                       ..default()
                   },
                   HudEntity,
                   ArenaEntity,
                   HudPlaceholder { field: HudField::Hull },
                   Text::new(HULL_LABEL),
                   TextFont { font_size: HUD_FONT_SIZE, ..default() },
                   TextColor(HUD_TEXT_COLOR),
               ));
               // bottom-left: AMMO
               parent.spawn((
                   Node {
                       position_type: PositionType::Absolute,
                       bottom: Val::Px(HUD_CORNER_MARGIN_PX),
                       left: Val::Px(HUD_CORNER_MARGIN_PX),
                       ..default()
                   },
                   HudEntity,
                   ArenaEntity,
                   HudPlaceholder { field: HudField::Ammo },
                   Text::new(AMMO_LABEL),
                   TextFont { font_size: HUD_FONT_SIZE, ..default() },
                   TextColor(HUD_TEXT_COLOR),
               ));
               // bottom-right: SALVAGE
               parent.spawn((
                   Node {
                       position_type: PositionType::Absolute,
                       bottom: Val::Px(HUD_CORNER_MARGIN_PX),
                       right: Val::Px(HUD_CORNER_MARGIN_PX),
                       ..default()
                   },
                   HudEntity,
                   ArenaEntity,
                   HudPlaceholder { field: HudField::Salvage },
                   Text::new(SALVAGE_LABEL),
                   TextFont { font_size: HUD_FONT_SIZE, ..default() },
                   TextColor(HUD_TEXT_COLOR),
               ));
           });
       info!("spawned HUD with 4 corner placeholders (Shields/Hull/Ammo/Salvage)");
   }
   ```
   **And** `position_type: PositionType::Absolute` is set on **both** the root Node AND each corner Node — root is absolute so it overlays the entire viewport without participating in any future flexbox flow; child Nodes are absolute so they anchor to the parent's edges via `top`/`left`/`right`/`bottom` independently of one another (no flex-row/column layout artefacts)
   **And** **NO** `BackgroundColor` component on any Node (transparent by default per `BackgroundColor::DEFAULT`; the 3D scene shows through the entire HUD root area — non-obstruction acceptance criterion satisfied trivially)
   **And** the root Node carries BOTH `HudEntity` AND `ArenaEntity`, AND each of the 4 corner child Nodes ALSO carries BOTH `HudEntity` AND `ArenaEntity` — redundant on children given the root will despawn recursively, BUT defensive: if a future system mutates Children to remove a HUD child without despawning, the orphan still gets cleaned up via the cleanup_on_exit::<ArenaEntity> sweep. Cost: 4 redundant Component inserts per Arena entry — negligible
   **And** the `info!` log line is exactly `"spawned HUD with 4 corner placeholders (Shields/Hull/Ammo/Salvage)"` (matches the project's existing `info!("spawned PlayerShip ...")` precedent at `src/flight/mod.rs:139`); used by AC #11's runtime smoke as a presence signal in `/tmp/story-3-11-run.log`

7. **Given** the import surface of `src/ui/hud.rs` must be minimal but complete
   **When** the use block is authored at the top of `src/ui/hud.rs` (immediately after the `//!` module doc-comment)
   **Then** the use block is exactly:
   ```rust
   use bevy::prelude::*;

   use crate::arena::ArenaEntity;
   ```
   **And** `bevy::prelude::*` imports `Component, Commands, Color, Node, Val, PositionType, Text, TextFont, TextColor, info!, default(), Plugin, App` (and the `Color::srgb` const-fn, plus the macros) — verified by precedent in `src/ui/main_menu.rs:1-7` which uses the same wildcard with the same set of types
   **And** `crate::arena::ArenaEntity` is the dual-marker imported from `src/arena/mod.rs:21` (`pub struct ArenaEntity`)
   **And** **NO** other crate-internal imports needed (HudEntity / HudField / HudPlaceholder are defined in this same file; `GameState` is referenced only in `src/ui/mod.rs`'s plugin registration, NOT in `hud.rs` itself; no `tuning` / `visual` / `combat` / `flight` deps)

8. **Given** the test surface for `src/ui/hud.rs` consists of: (a) the `HudField` enum's variant distinctness (footgun guard analogous to `pause::PauseInitiator` test at `src/pause/mod.rs:192-194`); (b) `HudPlaceholder` construction explicitness; (c) constant well-formedness checks; AND no system-level test (spawn_hud requires MinimalPlugins + state setup; integration testing is deferred per architecture.md:354)
   **When** the test block is authored
   **Then** a `#[cfg(test)] mod tests` block at the bottom of `src/ui/hud.rs` contains exactly these 4 tests:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn hud_field_variants_are_distinct() {
           // Guards against accidental enum-variant duplication during future
           // refactors (e.g., if Epic 5 adds Hull to HudField a second time,
           // this fails before the spawn loop creates two hull labels).
           assert_ne!(HudField::Shields, HudField::Hull);
           assert_ne!(HudField::Hull, HudField::Ammo);
           assert_ne!(HudField::Ammo, HudField::Salvage);
           assert_ne!(HudField::Shields, HudField::Salvage);
       }

       #[test]
       fn hud_placeholder_carries_specified_field() {
           // Round-trip explicit-construction guard (no Default derive).
           let p = HudPlaceholder { field: HudField::Salvage };
           assert_eq!(p.field, HudField::Salvage);
       }

       #[test]
       fn hud_font_size_smaller_than_pause_overlay() {
           // HUD is at-a-glance tactical state; pause overlay is attention-grabbing.
           // The relative sizing relationship is part of FR24 / NFR-A3 design intent
           // and a regression here would mean the HUD has been accidentally upgraded
           // to menu-grade prominence.
           const PAUSE_FONT_SIZE: f32 = 48.0; // mirror of src/pause/mod.rs:175
           const { assert!(HUD_FONT_SIZE < PAUSE_FONT_SIZE) };
       }

       #[test]
       fn hud_corner_labels_contain_expected_field_names() {
           // Lightweight contract test: the four label strings must mention the
           // four field semantic names. Catches accidental cross-wiring (e.g., a
           // refactor that swapped SHIELDS_LABEL and HULL_LABEL would still
           // compile but fail this test).
           assert!(SHIELDS_LABEL.contains("SHIELDS"));
           assert!(HULL_LABEL.contains("HULL"));
           assert!(AMMO_LABEL.contains("AMMO"));
           assert!(SALVAGE_LABEL.contains("SALVAGE"));
       }
   }
   ```
   **And** **NO** test for `spawn_hud` system itself (requires MinimalPlugins + Bevy UI plugin setup; deferred post-M3 per architecture.md:354 — the runtime smoke per AC #11 is the verification surface)
   **And** **NO** test for plugin registration in `UiPlugin::build` (same deferral)
   **And** Story 3.11 adds **4 net new test functions** in `src/ui/hud.rs`. Net post-3.11 test count: **49** (= 45 from end of 3.10 + 4 new in `src/ui/hud.rs`). AC #11 enforces N = 49 at verification time. The existing 1 test in `src/ui/main_menu.rs` is **NOT** modified

9. **Given** Story 3.11 closes a structural pre-existing gap noted in deferred-work.md: the `Generic-cleanup home re-evaluation now triggered (3rd consumer pending)` entry (deferred-work.md:186) which anticipated Story 3.11 as the 3rd direct consumer of `cleanup_on_exit::<T>` (after `<ArenaEntity>` from 3.2/3.3 and `<PauseOverlayEntity>` from 3.4)
   **When** Story 3.11 implementation completes
   **Then** the dev appends a status update to that deferred entry (preserve the original entry text, add a `> 📝 UPDATED 2026-05-XX by Story 3.11` blockquote underneath) along these lines:
   `> 📝 UPDATED 2026-05-XX by Story 3.11 — HUD did NOT introduce a 3rd direct consumer of cleanup_on_exit::<T>. HUD entities are dual-marked HudEntity + ArenaEntity, so existing cleanup_on_exit::<ArenaEntity> from ArenaPlugin handles them transitively. The decision-trigger for moving the generic to src/core/cleanup.rs remains the next direct consumer (likely Story 4.9 post-run-summary or 9.2 photo-mode).`
   **And** the deferred-work entry "Camera2d `order: 1` — no documented slot-reservation convention" (deferred-work.md:194) is **NOT** modified — Story 3.11 sidesteps the issue (no Camera2d added) but the underlying convention gap stays open for 4.9 post-run / 9.2 photo-mode
   **And** the deferred-work entry "Implicit cockpit Camera3d render order — no `order: 0` set" (deferred-work.md:214) is **NOT** modified — Story 3.11 doesn't introduce a 2nd camera, so the implicit order:0 stays implicit. The trigger for explicit ordering remains 4.9 / 9.2

10. **Given** the post-3.10 source baseline (45 tests passing per `cargo test 2026-05-05`; `cargo build --release` 0 warnings; `src/ui/mod.rs` = 25 lines; `src/ui/main_menu.rs` = 76 lines; NO `src/ui/hud.rs` exists; `tuning.ron` does not contain HUD fields; `assets/strings/` directory is empty)
    **When** Story 3.11 verification runs locally (per `feedback_full_build_output.md` discipline — exit-0 + tail is NOT proof; grep for `warning:|error:` per command, capture each to `/tmp/story-3-11-<command>.log`)
    **Then** **all six** of `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo build --release` produce **0** lines matching `grep -cE 'warning:|error:'` per their respective full-output logs
    **And** `cargo test` summary line reads `test result: ok. N passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` where **N = 49** (= 45 baseline + 4 new in `src/ui/hud.rs`)
    **And** the runtime smoke (Task 6 below) verifies all of (a)–(g) per AC #11
    **And** `git status --short` final set is **exactly**: `?? src/ui/hud.rs` (new file: types + constants + spawn_hud + 4 tests), `M src/ui/mod.rs` (M — `pub mod hud;` + new `OnTransition` registration in `UiPlugin::build`), `M _bmad-output/implementation-artifacts/sprint-status.yaml` (M — bookkeeping), `M _bmad-output/implementation-artifacts/deferred-work.md` (M — appended UPDATED note to "Generic-cleanup home" entry per AC #9), `?? _bmad-output/implementation-artifacts/3-11-hud-baseline-screen-space-placeholders.md` (?? at story-creation time, becomes M after dev flips Status to in-progress / review). **NO** entries under: `Cargo.toml` / `Cargo.lock` (no dep added — `Color::srgb`, `Node`, `Text`, `TextFont`, `TextColor`, `Val`, `PositionType` all already in `bevy::prelude` per Bevy 0.18.1 manifest; the `default_font` feature on bevy is already enabled at `Cargo.toml:8` so the literal "∞" Unicode glyph renders without any additional asset), `src/arena/**` (3.11 only IMPORTS ArenaEntity; doesn't modify arena code), `src/combat/**` (out of scope), `src/flight/**` (out of scope — preserves AC #2's pause-aware OnTransition discipline by NOT touching cockpit camera setup), `src/state.rs`, `src/pause/**` (NO Camera2d order changes — AC #1 sidesteps the collision), `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/tuning/**`, `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`

11. **Given** Story 3.11's runtime smoke is the integration test for the full HUD presentation chain `Loading → MainMenu → [Enter] → Arena spawn (ship + 17 asteroids + DirectionalLight + HUD root + 4 corner Nodes) → all 4 corner labels visible simultaneously` (per architecture.md:354 integration tests deferred post-M3, and the smoke precedent of Stories 3.6/3.7/3.8/3.9/3.10)
    **When** the dev runs the runtime smoke (`cargo run 2>&1 | tee /tmp/story-3-11-run.log`)
    **Then** the dev verifies all of:
    - **(a) HUD visible at Arena entry** — within ~2 s of pressing Enter on MainMenu (after splash), all four placeholder strings are simultaneously visible: `SHIELDS 100` top-left, `HULL 100` top-right, `AMMO ∞` bottom-left, `SALVAGE 0` bottom-right. Each ~24 px from the nearest corner. NO clipping at window edges. Exactly 1 occurrence of `info!("spawned HUD with 4 corner placeholders (Shields/Hull/Ammo/Salvage)")` in `/tmp/story-3-11-run.log`. NO `WARN`/`ERROR` lines from `bevy_ui` (specifically NO "Two or more Entities with IsDefaultUiCamera" — confirms AC #1's no-IsDefaultUiCamera choice)
    - **(b) HUD does NOT obstruct central line of sight** — the central ~80% of the screen (where asteroids are visible per Story 3.5's 50 m line-of-sight precondition) shows the asteroid field cleanly. NO HUD background/fill overlapping the central area (AC #6 transparent default). At least 3 asteroids visible between the four corner labels
    - **(c) HUD persists through ship-flight + combat actions** — fly forward (W) for 5 seconds, strafe left/right (A/D), pitch with mouse, roll with Q/E, fire LMB (single shot, then held burst), destroy 1+ asteroid → all four placeholder values remain **STATIC** (`SHIELDS 100` does NOT decrement on collision; `HULL 100` does NOT decrement; `AMMO ∞` does NOT change; `SALVAGE 0` does NOT increment when an asteroid dies). Real wiring is Epic 5 (Hull/Shield) + Epic 6 (Salvage). The static behavior is intentional per `epics/epic-3-arena-flight-first-combat-first-playable.md:323-325`
    - **(d) HUD survives Pause round-trip (Esc)** — press Esc to pause → "PAUSED — Esc to resume" appears centered + the pause overlay's BLACK clear obscures the 3D scene. The 4 HUD placeholder labels remain rendered (acceptable: pause Camera2d order:1 wins the default-UI-camera vote per AC #1, HUD nodes re-target to it, render on top of pause's BLACK clear). Press Esc again to resume → 3D scene returns + HUD remains in its 4 corners. NO HUD entity respawn (no second `info!("spawned HUD ...")` log line emerges; total occurrences across smoke = 1)
    - **(e) HUD survives focus-loss round-trip (Cmd-Tab on macOS / Alt-Tab Win/Linux)** — alt-tab away from window → game pauses (silent — no overlay since `PauseInitiator::FocusLoss`); HUD presumably re-targets to nothing (window-not-focused, render is suspended) — verify no panic. Alt-tab back → game resumes; HUD re-renders intact in all 4 corners. Total `info!("spawned HUD ...")` count remains 1
    - **(f) Quit cleanly during Arena** — close window while in Arena → no panic; tracing log file (under user-log-dir) ends cleanly with no stack trace; no `ERROR` / `panic` lines in the smoke log
    - **(g) No Bevy UI / camera ambiguity warnings across the entire smoke** — `grep -cE 'WARN.*camera|ambiguous.*camera|UiTargetCamera|IsDefaultUiCamera' /tmp/story-3-11-run.log` outputs `0` confirming the no-Camera2d / no-IsDefaultUiCamera choice doesn't trip any of Bevy's UI-camera-resolution warnings

## Tasks / Subtasks

- [x] **Task 1: Author `src/ui/hud.rs` — types, constants, spawn_hud system, 4 unit tests** (AC: #1, #4, #5, #6, #7, #8)
  - [x] Create new file `src/ui/hud.rs`. Author top-down in this order:
    1. Module doc-comment:
       ```rust
       //! HUD baseline (FR24) — screen-space corner placeholders for Shields / Hull / Ammo / Salvage.
       //! Renders via Bevy 0.18's default-UI-camera fallback on the cockpit Camera3d during Arena;
       //! re-targets to the pause Camera2d during Paused (transparent algorithmic consequence).
       //! Placeholder values are Epic-3 static; Epic 5 wires Shields/Hull, Epic 6 wires Salvage.
       ```
    2. Use block per AC #7 verbatim.
    3. Constants per AC #5 verbatim.
    4. Type definitions per AC #4 verbatim (HudEntity, HudField, HudPlaceholder).
    5. `spawn_hud` system per AC #6 verbatim.
    6. `#[cfg(test)] mod tests { ... }` block with 4 tests per AC #8 verbatim.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-3-11-check-task1.log; grep -cE 'warning:|error:' /tmp/story-3-11-check-task1.log` should output `0` only AFTER Task 2's `pub mod hud;` lands. Defer the green-check expectation to end of Task 2.

- [x] **Task 2: Extend `src/ui/mod.rs` — `pub mod hud;` + spawn_hud OnTransition registration** (AC: #2, #3)
  - [x] Open `src/ui/mod.rs`. Add `pub mod hud;` at the top of the module list (alphabetical: `hud` before `main_menu` — final ordering: `pub mod hud;\npub mod main_menu;`).
  - [x] In `impl Plugin for UiPlugin::build`, AFTER the existing `main_menu::cleanup_main_menu` registration block (the last `.add_systems(...)` block in the function), add:
    ```rust
    .add_systems(
        OnTransition {
            exited: crate::state::GameState::MainMenu,
            entered: crate::state::GameState::Arena,
        },
        hud::spawn_hud,
    );
    ```
    Note: the existing `app.add_systems(...)` chain ends at the `cleanup_main_menu` registration with a `;`. Replace that terminal `;` with `.add_systems(...)`-then-`;` to extend the call chain. OR, equivalently, end the existing chain and start a new statement `app.add_systems(OnTransition {...}, hud::spawn_hud);`. Both are idiomatic; choose whichever reads cleaner with `cargo fmt`.
  - [x] **Verify post-edit:** `cargo check 2>&1 | tee /tmp/story-3-11-check-task2.log; grep -cE 'warning:|error:' /tmp/story-3-11-check-task2.log` should output `0`. If a `dead_code` warning fires on `HudPlaceholder.field` (because Story 3.11 only WRITES the field; no reader yet — Epic 5/6 readers haven't landed), apply the project-precedent `#[allow(dead_code, reason = "...")]` pattern from Story 3.10's `AsteroidDestroyed.asteroid` (per `src/combat/damage.rs` and Story 3.10 dev notes Item 2). Reason text: `"HudPlaceholder.field is read by Epic 5 (Shields/Hull update systems) and Epic 6 (Salvage update system); Story 3.11 wires the placeholder slot only."`. Same allow may be needed for `HudEntity` if no Query references it post-spawn — the field-version is `#[allow(dead_code, reason = "...")]` on the struct.

- [x] **Task 3: Update `_bmad-output/implementation-artifacts/deferred-work.md` — annotate the cleanup-3rd-consumer entry per AC #9** (AC: #9)
  - [x] Open `_bmad-output/implementation-artifacts/deferred-work.md`. Find the entry "Generic-cleanup home re-evaluation now triggered (3rd consumer pending)" at line 186.
  - [x] Append a `> 📝 UPDATED 2026-05-XX by Story 3.11 — ...` blockquote BELOW the existing entry (preserve the original text). Exact wording at impl time, but matching AC #9's content: HUD chose dual-marker (HudEntity + ArenaEntity) over a 3rd direct cleanup_on_exit::<HudEntity>; trigger remains the next direct consumer.

- [x] **Task 4: Verification gates — all 6 cargo commands clean** (AC: #10)
  - [x] Run each command in sequence; capture FULL output (NOT just exit code or tail) per `feedback_full_build_output.md`:
    ```bash
    cargo check                                         2>&1 | tee /tmp/story-3-11-check.log
    cargo build                                         2>&1 | tee /tmp/story-3-11-build.log
    cargo test                                          2>&1 | tee /tmp/story-3-11-test.log
    cargo clippy --all-targets -- -D warnings           2>&1 | tee /tmp/story-3-11-clippy.log
    cargo fmt --all -- --check                          2>&1 | tee /tmp/story-3-11-fmt.log
    cargo build --release                               2>&1 | tee /tmp/story-3-11-release.log
    ```
  - [x] For EACH log: `grep -cE 'warning:|error:' /tmp/story-3-11-<cmd>.log` must output `0`. If non-zero, fix and re-run from the failing command. NO partial-pass shortcuts.
  - [x] `cargo test` log MUST contain `49 passed` AND `0 failed`. Confirm the literal line `test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (or accept a less specific variant: confirm `49 passed` AND `0 failed`).

- [x] **Task 5: Runtime smoke — full HUD presentation chain validation** (AC: #10, #11)
  - [x] Till manually executes `cargo run 2>&1 | tee /tmp/story-3-11-run.log` and verifies scenarios (a)–(g) per AC #11. Bevy app smoke requires interactive input (Enter on MainMenu, flight controls, Esc, Cmd-Tab) — LLM cannot execute. All preconditions for the smoke must be met (cargo gates clean per Task 4).
  - [x] `grep -c 'spawned HUD with 4 corner placeholders' /tmp/story-3-11-run.log` outputs **1** after a single Arena entry; **1** after a Pause round-trip (HUD does not respawn); increments by 1 per actual MainMenu→Arena transition (none expected in Epic 3 smoke since Arena→MainMenu transition isn't user-triggerable yet).
  - [x] `grep -cE 'WARN.*camera|ambiguous.*camera|IsDefaultUiCamera' /tmp/story-3-11-run.log` outputs **0**.
  - [x] `grep -cE 'panic|backtrace|FATAL' /tmp/story-3-11-run.log` outputs **0**.

- [x] **Task 6: Sprint status bookkeeping** (AC: #10)
  - [x] After Task 4 + Task 5 confirmed green, update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - Find development_status[`3-11-hud-baseline-screen-space-placeholders`].
    - Flip `backlog` → `ready-for-dev` at story-creation time (this is done by the create-story workflow — see workflow.md step 6).
    - Dev workflow flips `ready-for-dev` → `in-progress` → `review` → `done` per the standard lifecycle.
    - Status flip `in-progress` → `review` is performed once Till's manual smoke (Task 5) is confirmed.

## Dev Notes

### Relevant architecture patterns and constraints

- **Plugin boundaries** (architecture.md:643-658) — UiPlugin owns HUD UI nodes. NO new plugin (HudPlugin) introduced; the project's "one Plugin per top-level feature module" pattern is preserved. Cross-plugin: UiPlugin reads ArenaEntity from arena module (one-line type import only — `crate::arena::ArenaEntity`); architecture-compliant since this is a public type-vocabulary item, NOT internal Resource/Component mutation.
- **OnTransition vs OnEnter/OnExit** (precedent: src/flight/mod.rs:45-58, src/arena/mod.rs:23-50) — Epic-3 spawn/cleanup uses `OnTransition { from, to }` to preserve entities across Pause round-trips. Story 3.11's HUD inherits this pattern; the epic-spec wording at `epics/epic-3-arena-flight-first-combat-first-playable.md:309,328` uses `OnEnter`/`OnExit` which predates the OnTransition convention.
- **Default-UI-camera fallback** (`bevy_ui-0.18.1/src/ui_node.rs:2927-2946`) — `cameras.iter().max_by_key(|(e, c, _)| (c.order, *e))` resolves UI target. NOT introducing a 2nd Camera2d sidesteps the order:1 collision deferred-work entry. Confirmed by reading bevy_ui-0.18.1 source.
- **Past-tense event naming** (architecture.md:324) — N/A for Story 3.11; no events introduced (all systems are spawn-once on transition).
- **No magic numbers** (architecture.md:463) — `100`, `0`, `∞` are Epic-3-MVP placeholder literals matching epic spec verbatim, NOT TuningConfig fields. Future Story 5.4 (HUD wiring for shields/hull) will introduce live-value Query-based update systems; those values come from ShieldHP / HullHP components, not tuning.
- **String externalization (NFR-L3)** — string-table externalization is Epic 4+ work (architecture.md:843); Epic 3 ships hardcoded `&'static str` constants. Future Story 4.x string-table introduction will migrate these constants; Story 3.11 establishes the slot.

### Source tree components to touch

| File | Change | LOC delta (estimate) |
|------|--------|---------------------|
| `src/ui/hud.rs` | NEW: types + constants + spawn_hud + 4 tests | +160 |
| `src/ui/mod.rs` | Add `pub mod hud;`; extend UiPlugin::build with OnTransition spawn_hud registration | +9 |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Append UPDATED note to cleanup-3rd-consumer entry | +1 line |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | bookkeeping (status field) | +0 net |

NO changes expected in: `src/arena/**`, `src/combat/**`, `src/flight/**`, `src/state.rs`, `src/pause/**`, `src/visual/**`, `src/splash.rs`, `src/logging.rs`, `src/main.rs`, `src/tuning/**`, `assets/**`, `Cargo.toml`, `Cargo.lock`, `.github/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

### Testing standards summary

- **Unit tests only** for Story 3.11 (architecture.md:354 — integration tests deferred post-M3). Pure-logic / type-vocabulary checks are first-class targets.
- 4 tests in `src/ui/hud.rs`: HudField variant distinctness; HudPlaceholder construction; HUD font size sanity; corner-label semantic content.
- NO test for `spawn_hud` system (requires MinimalPlugins + state setup; same deferral as Stories 3.5–3.10).
- Runtime smoke (Task 5) covers the full system-level chain that integration tests would verify: MainMenu→Arena→HUD-visible→pause-roundtrip→focus-loss→clean-quit.
- Test count post-3.11: **49** (= 45 baseline + 4 net new). AC #10 enforces.

### Project Structure Notes

- **Alignment with unified project structure:** `src/ui/hud.rs` is exactly where architecture.md:592 places it. `src/ui/mod.rs` already exists with the submodule + plugin-extension pattern from Story 3.1.
- **Detected variances:**
  - Epic-spec wording ("HudPlugin", "OnEnter/OnExit") deviates from established project patterns ("extend UiPlugin", "OnTransition"). The implementation follows the project pattern; deviations from epic-spec wording are documented inline in AC #2 / AC #3 with rationale tied to the Pause round-trip preservation contract.
  - Epic-spec did not anticipate the Bevy 0.18 default-UI-camera fallback algorithm; AC #1 takes the simpler "no Camera2d" path verified against `bevy_ui-0.18.1/src/ui_node.rs:2927-2946`.
- **Architecture compliance:** plugin boundaries respected (UiPlugin owns HUD; arena consumes only via type import); OnTransition pattern preserves Pause round-trip; no new Camera2d means no order-slot convention pressure (deferred-work entry stays open for future work).

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md#Story-3.11] — story spec (Acceptance Criteria source)
- [Source: _bmad-output/planning-artifacts/architecture.md#UI-Menu-Debug-Architecture] — bevy_ui screen-space HUD strategy (line 226-235)
- [Source: _bmad-output/planning-artifacts/architecture.md#Project-Directory-Structure] — `src/ui/hud.rs` location (line 592)
- [Source: _bmad-output/planning-artifacts/architecture.md#Communication-Patterns] — system ordering / state-transition cleanup conventions (line 408-420)
- [Source: _bmad-output/implementation-artifacts/3-10-projectile-asteroid-collision-damage.md] — precedent for spawn-tuple expansions, dual-marker components, dev-note structure
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:186] — Generic-cleanup home re-evaluation entry (closed-by-non-introduction in this story)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:194] — Camera2d order:1 convention gap (sidestepped by no-Camera2d choice)
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:214] — Implicit cockpit Camera3d order entry (sidestepped by no-2nd-camera choice)
- [Source: bevy_ui-0.18.1/src/ui_node.rs:2865-2946] — UiTargetCamera + IsDefaultUiCamera + DefaultUiCamera::get fallback algorithm
- [Source: bevy_ui-0.18.1/src/ui_node.rs:519-523, 1446-1453] — PositionType + absolute-positioning Val fields
- [Source: src/ui/main_menu.rs] — bevy_ui Node + Text + TextFont + TextColor pattern precedent
- [Source: src/pause/mod.rs:148-180] — Camera2d order:1 + UI overlay precedent (the precedent we are explicitly NOT replicating per AC #1)
- [Source: src/flight/mod.rs:88-140] — OnTransition spawn pattern + dual-marker component precedent (PlayerShip + ArenaEntity)
- [Source: src/arena/mod.rs:23-58] — cleanup_on_exit::<T> generic + OnTransition cleanup precedent

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (1M context)

### Debug Log References

- `/tmp/story-3-11-check.log` — `cargo check` (0 warning|error)
- `/tmp/story-3-11-build.log` — `cargo build` (0 warning|error)
- `/tmp/story-3-11-test.log` — `cargo test` (`test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`)
- `/tmp/story-3-11-clippy.log` — `cargo clippy --all-targets -- -D warnings` (0 warning|error)
- `/tmp/story-3-11-fmt.log` — `cargo fmt --all -- --check` (0 warning|error, no diff)
- `/tmp/story-3-11-release.log` — `cargo build --release` (0 warning|error, finished in ~4m21s)

### Completion Notes List

- **Task 1 (`src/ui/hud.rs`)** — authored verbatim per AC #4–#8: module doc, `use bevy::prelude::*` + `use crate::arena::ArenaEntity`, four `&'static str` corner labels, `HUD_FONT_SIZE` / `HUD_CORNER_MARGIN_PX` / `HUD_TEXT_COLOR` constants, `HudEntity` marker, `HudField` enum (Shields/Hull/Ammo/Salvage), `HudPlaceholder { field: HudField }` companion, `spawn_hud(mut commands: Commands)` system with one absolute-positioned root `Node` (full-window, transparent) plus four corner child `Node`s carrying `(HudEntity, ArenaEntity, HudPlaceholder, Text, TextFont, TextColor)` tuples, `info!("spawned HUD with 4 corner placeholders (Shields/Hull/Ammo/Salvage)")` log line, and `#[cfg(test)] mod tests` block with the 4 tests prescribed in AC #8. NO `Default` derive on `HudField` or `HudPlaceholder` (footgun-prevention per AC #4 / Story 3.10 precedent).
- **Dead-code allow on `HudPlaceholder.field`** — applied preemptively per Task 2 verify-step guidance: `#[allow(dead_code, reason = "HudPlaceholder.field is read by Epic 5 (Shields/Hull update systems) and Epic 6 (Salvage update system); Story 3.11 wires the placeholder slot only.")]`. Sole `#[allow]` introduced; `HudEntity` did NOT need the attribute — its construction in `spawn_hud` tuples plus the `Component` derive trait-impl is sufficient to satisfy `cargo clippy --all-targets -- -D warnings`.
- **Task 2 (`src/ui/mod.rs`)** — added `pub mod hud;` BEFORE `pub mod main_menu;` (alphabetical); extended `UiPlugin::build` AFTER the existing `main_menu::cleanup_main_menu` registration with a fresh `app.add_systems(OnTransition { exited: GameState::MainMenu, entered: GameState::Arena }, hud::spawn_hud)` statement (chose the new-statement form over chained `.add_systems(...)` for readability — both are idiomatic per Task 2 sub-bullet). NO HUD-cleanup system registered (HUD entities are dual-marked `HudEntity + ArenaEntity`; existing `cleanup_on_exit::<ArenaEntity>` from `ArenaPlugin` handles them). NO Camera2d / `IsDefaultUiCamera` / `UiTargetCamera` introduced — relies on Bevy 0.18.1's default-UI-camera fallback (`bevy_ui-0.18.1/src/ui_node.rs:2927-2946`) onto the cockpit `Camera3d` during `GameState::Arena` and onto the pause `Camera2d` during `GameState::Paused` — all per AC #1.
- **Task 3 (`deferred-work.md`)** — appended `> 📝 UPDATED 2026-05-05 by Story 3.11 — ...` blockquote under the "Generic-cleanup home re-evaluation now triggered" entry per AC #9. Original entry text preserved; entry stays open for the next direct consumer (4.9 / 9.2). The Camera2d-`order:1` and implicit-Camera3d-`order:0` entries deliberately NOT modified — Story 3.11 sidesteps both by not introducing a 2nd camera.
- **Task 4 (cargo gates)** — all 6 commands tee'd to `/tmp/story-3-11-*.log`; `grep -cE 'warning:|error:'` returns `0` for each; `cargo test` reports `49 passed; 0 failed` (= 45 baseline + 4 new) precisely matching AC #10 expected count. `cargo build --release` succeeded in ~4m21s (re-link of the full project; expected on first release build of the session).
- **Task 5 (runtime smoke)** — pending Till's manual run (`cargo run 2>&1 | tee /tmp/story-3-11-run.log`). LLM cannot execute the interactive smoke (Enter on MainMenu, flight controls, Esc, Cmd-Tab). All preconditions for the smoke (Task 4 cargo gates) are green, so Till can run it directly.
- **Task 6 (sprint-status)** — flipped `ready-for-dev` → `in-progress` at story start; final flip `in-progress` → `review` is gated on Till's Task 5 confirmation (matches the convention from Stories 3.5–3.10 where the dev hands off to user smoke before review).
- **Architecture-compliance notes** — (a) plugin boundary respected: `UiPlugin` owns the HUD systems, `arena` exposes only the `ArenaEntity` type via public re-export at `src/arena/mod.rs:21`; no cross-plugin Resource/Component mutation. (b) `OnTransition` discipline preserved per the post-3.9 pause-roundtrip preservation contract — HUD is registered on `OnTransition { exited: MainMenu, entered: Arena }` (NOT `OnEnter(Arena)` as the epic-spec wording at `epics/epic-3-arena-flight-first-combat-first-playable.md:309` literally reads). (c) No `tuning.ron` mutation, no `assets/strings/` introduction, no Cargo dep additions — consistent with AC #10's expected file-set.

### File List

- **NEW** `src/ui/hud.rs` — module doc, use block, 4 string + 3 numeric/color constants, `HudEntity` marker, `HudField` enum, `HudPlaceholder { field }`, `spawn_hud` system, `#[cfg(test)] mod tests` (4 tests). 198 lines.
- **MODIFIED** `src/ui/mod.rs` — added `pub mod hud;` line; extended `UiPlugin::build` with `OnTransition { exited: MainMenu, entered: Arena }` registration of `hud::spawn_hud`. +12 lines.
- **MODIFIED** `_bmad-output/implementation-artifacts/deferred-work.md` — appended `📝 UPDATED 2026-05-05 by Story 3.11` blockquote under the "Generic-cleanup home re-evaluation now triggered (3rd consumer pending)" entry. +2 lines.
- **MODIFIED** `_bmad-output/implementation-artifacts/sprint-status.yaml` — flipped `3-11-hud-baseline-screen-space-placeholders` from `ready-for-dev` → `in-progress`; final flip to `review` upon Till's Task 5 confirmation. +0 net lines (in-place value change).
- **MODIFIED** `_bmad-output/implementation-artifacts/3-11-hud-baseline-screen-space-placeholders.md` — Status, Tasks/Subtasks checkboxes, Dev Agent Record, File List, Change Log updated by this dev workflow run.

## Change Log

| Date       | Author       | Change                                                                                                                                            |
|------------|--------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| 2026-05-05 | Amelia (Dev) | Authored `src/ui/hud.rs` (HUD types + constants + `spawn_hud` + 4 tests); extended `UiPlugin` with `OnTransition` HUD spawn; updated deferred-work.md cleanup-3rd-consumer entry; all 6 cargo gates green (49 tests passing); Till confirmed runtime smoke (a)–(g) per AC #11 — Status flipped to `review`. |
| 2026-05-05 | Code Review  | 3-layer review (Blind Hunter + Edge Case Hunter + Acceptance Auditor). 0 decision-needed, 0 patch, 5 deferred, 8 dismissed. Story → done. |

## Review Findings

- [x] [Review][Defer] `spawn_hud` has no duplicate-guard — repeated MainMenu→Arena transitions will stack HUD entities when Story 4.7 wires round-trips [src/ui/hud.rs:52] — deferred, forward-compat concern
- [x] [Review][Defer] `cleanup_on_exit` uses `despawn()` not `despawn_recursive()` — all future HUD descendants must be individually `ArenaEntity`-tagged or they leak [src/arena/mod.rs:56] — deferred, pre-existing pattern gap
- [x] [Review][Defer] No ordering constraint between `spawn_hud` and `spawn_player_ship` — safe now but Epic 5 HUD update systems must add `.after(FlightSystems::Setup)` [src/ui/mod.rs:29-35] — deferred, Epic 5 concern
- [x] [Review][Defer] HUD root 100%×100% absolute node has no pointer-event passthrough guard — will block mouse clicks on future mouse-driven UI stories [src/ui/hud.rs:53-62] — deferred, harmless in Epic 3
- [x] [Review][Defer] `sprint-status.yaml` has `last_updated` duplicated in comment block and document body — pre-existing structural redundancy [sprint-status.yaml] — deferred, pre-existing
