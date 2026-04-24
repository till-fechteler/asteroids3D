# Story 1.6: GameState Enum with Bevy States Skeleton

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want a `GameState` enum registered with Bevy's `States` API,
So that future plugins can hook `OnEnter`/`OnExit`/`in_state()` scheduling from M1 onward without retrofit.

## Acceptance Criteria

1. **Given** `src/state.rs` is created
   **When** `GameState` is defined with variants `Loading`, `MainMenu`, `Arena`, `Caravan`, `PostRun`, `PhotoMode`, `Paused`
   **Then** it derives `States`, `Default` (default = `Loading`), `Debug`, `Clone`, `Eq`, `PartialEq`, `Hash`

2. **Given** `App::init_state::<GameState>()` is called in `main.rs`
   **When** the app starts
   **Then** `State<GameState>::get()` returns `GameState::Loading` on first frame

3. **Given** a debug system registered on `OnEnter(GameState::Loading)` emits an `info!` log
   **When** the app starts
   **Then** the log contains the expected `"entered Loading"` line
   **And** no further state transitions happen automatically in this story (the transition to `MainMenu` is Story 1.7)

4. **Given** a bundled deferred item from Story 1.5 review (`deferred-work.md:50`)
   **When** `fn main()` is touched for GameState wiring
   **Then** its signature becomes `fn main() -> AppExit`
   **And** `App::run()`'s return value is propagated (Bevy 0.18's `App::run() -> AppExit` is the crash-signal surface; discarding it hides `AppExit::Error(_)`)

## Tasks / Subtasks

- [x] **Task 1: Create `src/state.rs` with `GameState` enum** (AC: #1)
  - [x] New file `src/state.rs`.
  - [x] Define `pub enum GameState` with variants `Loading, MainMenu, Arena, Caravan, PostRun, PhotoMode, Paused`, in that order.
  - [x] Annotate `Loading` with `#[default]`.
  - [x] Derive attribute: `#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]`.
  - [x] Import path: `use bevy::prelude::*;` suffices — `States` is re-exported by prelude in Bevy 0.18.
  - [x] Keep a short `//!` module doc (≤2 lines) describing the file's purpose. Do NOT include story/change-log references (Bevy convention; matches 1.5 review-patch BH8).

- [x] **Task 2: Add debug system `log_loading_entered`** (AC: #3)
  - [x] In `src/state.rs`, add `pub fn log_loading_entered()` that emits `info!("entered Loading")`.
  - [x] Signature: zero params (no `Commands`, no queries). The ONLY side-effect is the log line.
  - [x] Exact log string must be `entered Loading` (literal, no trailing punctuation) so a `grep 'entered Loading'` verification matches unambiguously.

- [x] **Task 3: Wire state + debug system into `main.rs`** (AC: #2, #3, #4)
  - [x] Add `mod state;` declaration at top of `src/main.rs` (single-binary layout — no `lib.rs` scaffolded yet; see "File Structure Requirements").
  - [x] Import the types/system: `use state::{GameState, log_loading_entered};` (or `use state::*;`).
  - [x] Change `fn main()` signature to `fn main() -> AppExit`.
  - [x] Use builder-style chain: `App::new().add_plugins(DefaultPlugins).init_state::<GameState>().add_systems(OnEnter(GameState::Loading), log_loading_entered).run()`.
  - [x] Return the `AppExit` value from `run()` (omit the trailing `;` or explicitly `return` it).
  - [x] Update the `//!` doc in `main.rs` to reflect: "asteroids3D — app entry point. Registers DefaultPlugins and GameState." (≤2 lines; no story reference per 1.5 review-patch BH8.)

- [x] **Task 4: Unit test for Default variant** (AC: #1)
  - [x] In `src/state.rs`, add a `#[cfg(test)] mod tests { … }` block with one test: `assert_eq!(GameState::default(), GameState::Loading);`.
  - [x] This is the first unit test in the project. It does NOT construct an `App` (no wgpu init, CI-safe on headless runners). The `msrv-check` job's CI gap for `--all-targets` (deferred-work.md:21) remains deferred — do not fix in this story.

- [x] **Task 5: Local verification sweep** (AC: #1, #2, #3, #4)
  - [x] `cargo check 2>&1 | tee /tmp/story-1-6-check.log; grep -E 'warning:|error:' /tmp/story-1-6-check.log` — expect zero hits (MEMORY: `feedback_full_build_output.md` — exit-0 + tail is NOT proof of correctness; grep explicitly).
  - [x] `cargo build 2>&1 | tee /tmp/story-1-6-build.log; grep -E 'warning:|error:' /tmp/story-1-6-build.log` — expect zero hits.
  - [x] `cargo test 2>&1 | tee /tmp/story-1-6-test.log; grep -E 'warning:|error:|FAILED' /tmp/story-1-6-test.log` — expect 1 test passed, zero warnings/errors/FAILED.
  - [x] `cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-1-6-clippy.log; grep -E 'warning:|error:' /tmp/story-1-6-clippy.log` — expect zero hits.
  - [x] `cargo fmt --all -- --check` — exit 0, no diff output.
  - [x] `cargo run 2>&1 | tee /tmp/story-1-6-run.log &` — wait ~12s for window, closed manually by user.
  - [x] `grep 'entered Loading' /tmp/story-1-6-run.log` — expect exactly one hit.
  - [x] `grep -E 'AdapterInfo|backend:' /tmp/story-1-6-run.log` — expect `backend: Metal` on macOS (parity evidence with Story 1.5's Adapter log).
  - [x] Capture the full `grep` outputs into Dev Agent Record → Debug Log References. Don't just say "clean" — paste line counts.

- [x] **Task 6: Scope guardrails — verify nothing else drifted** (AC: #3)
  - [x] `git status --short` — expect exactly: `src/main.rs` (M), `src/state.rs` (untracked). `Cargo.toml` and `Cargo.lock` MUST be untouched (no new deps in this story).
  - [x] `grep -nE 'NextState|MainMenu' src/` — expect hits ONLY in `state.rs`'s enum declaration (the `MainMenu` *variant*). No `NextState<GameState>(GameState::MainMenu)` mutation. That transition belongs to Story 1.7.
  - [x] `grep -n 'SplashConfig\|LoadingStateEntity' src/` — expect zero hits. Those markers belong to Story 1.7.
  - [x] No `bevy_ui` text nodes spawned. Window remains empty (default Bevy render) — splash-screen UI is Story 1.7.
  - [x] `.gitignore` untouched (the `.claude/scheduled_tasks.lock` defer remains open; see deferred-work.md:51).

- [x] **Task 7: Commit + CI observation** (AC: #1, #2, #3, #4)
  - [x] Stage only source artifacts: `git add src/main.rs src/state.rs`.
  - [x] Commit message (follow the 1.1–1.5 convention: single-line subject, no `Co-Authored-By` trailer, `feat:` prefix):
        `feat: GameState enum + Bevy States skeleton (Story 1.6)`
  - [x] Push to `origin/master`.
  - [x] Observe `gh run watch` or `gh run list --branch master --limit 1` — expect all 4 jobs (build×3 OS + msrv-check) ✅.
  - [x] Capture full CI log: `gh run view <run-id> --log | grep -E 'warning:|error:' | wc -l` — expect 0.
  - [x] BMad bookkeeping commit follows after CI green: story file + `sprint-status.yaml` status flip `ready-for-dev → in-progress → review` (mirrors 1.4/1.5 two-commit pattern).

## Dev Notes

### Why this story exists

Stories 1.1–1.5 established: Cargo + plugins compile (1.1, 1.2), toolchain + lint + format are pinned (1.3), CI is green on 3 OSes (1.4), a window opens on all 3 OSes (1.5). Story 1.6 installs the **state machine backbone** that every future gameplay plugin hangs off. From M1 onward, plugins will register systems via `OnEnter(GameState::X)` / `OnExit(GameState::X)` / `in_state(GameState::Y)` scheduling — the enum MUST exist before any of those registrations can compile. Epic 1's M0 completion criterion explicitly calls out "States skeleton" as a M0-Hello-Bevy deliverable. [Source: architecture.md:294; epic-1-*.md:124-143]

This story also **bundles one deferred item from Story 1.5's review**: `fn main() -> AppExit`. Bevy 0.18's `App::run()` returns an `AppExit` that carries `AppExit::Error(_)` on crash; discarding it lets a crashed Bevy loop report exit 0. `deferred-work.md:50` assigns the fix to Story 1.6 explicitly because 1.6 is the next story to touch `main.rs`. Bundling avoids a second `main.rs`-only commit. [Source: `_bmad-output/implementation-artifacts/deferred-work.md:50`]

### Context inherited from Stories 1.1–1.5

| Fact | Value | Source |
|---|---|---|
| Rust toolchain | `1.94.1` (stable, pinned) | `rust-toolchain.toml:5` |
| MSRV | `1.89` (CI-verified) | `Cargo.toml:5`; `.github/workflows/ci.yml` |
| Bevy | `0.18` (resolved `0.18.1`) | `Cargo.toml:8`; `docs/plugin-compatibility.md` |
| Package name | `asteroids3D` | `Cargo.toml:2` |
| Current `src/main.rs` body | 8 lines: module doc + `use bevy::prelude::*;` + `fn main() { App::new().add_plugins(DefaultPlugins).run(); }` | Post-Story-1.5 |
| CI workflow | `.github/workflows/ci.yml` — 3-OS build matrix + msrv-check, all 4 green on `8096ff0` | `.github/workflows/ci.yml` |
| Commit convention | Single-line subject; `feat:` / `chore:` / `docs:` / `fix:` / `ci:` / `bmad:` prefixes; NO `Co-Authored-By` trailer | `git log --oneline -n 15` |
| Local dev machine | macOS 26.4.1 / arm64 (Apple Silicon, M-series) | Prior story Debug Logs |
| Zero tests exist currently | Story 1.6 adds the first (`default == Loading`) | `cargo test` output in 1-5 DAR |

### Bevy 0.18 `States` API — what the dev agent must know

**Re-export:** `States` trait + `NextState<T>` + `State<T>` + `OnEnter` / `OnExit` schedules are all in `bevy::prelude`. `use bevy::prelude::*;` is sufficient.

**Derive contract:** `#[derive(States)]` requires `Debug + Clone + PartialEq + Eq + Hash + Default` on the same type — the `StatesPlugin` uses these bounds to register the state in the world and route transitions. This is why AC #1's derive list looks long — it's not gold-plating, it's the exact trait set `States` depends on. Skip any of them and `#[derive(States)]` won't compile.

**Registration:** `app.init_state::<GameState>()` — a single call. It:
1. Inserts `State<GameState>` = `State::default()` (which is `State(Loading)` thanks to the `Default` derive + `#[default]` attribute).
2. Inserts `NextState<GameState>::default()` (an `Option`-like wrapper currently `None`).
3. Configures the `StateTransition` schedule to run.
4. Registers the `OnEnter(GameState::Loading)` set — so the `log_loading_entered` system fires on the first `StateTransition` tick.

**First-frame timing:** `OnEnter(Loading)` runs during the first `StateTransition` schedule, which Bevy schedules immediately after `Startup` and before the first `Update`. So `info!("entered Loading")` lands in the log before any gameplay-relevant frame logic — verifiable via `grep` on `cargo run` output.

**Anti-pattern reminders (architecture.md:418-420):**
- Never mutate `State<GameState>` directly. Use `NextState<GameState>` (not in this story's scope anyway — 1.7 owns the first transition).
- `OnEnter`/`OnExit` systems must be idempotent and must not assume prior state.
- State-scoped entity cleanup uses marker components + a `cleanup_on_exit::<T>` system in `OnExit` — again, not this story's concern (no entities spawned here).

### `src/state.rs` Skeleton

The dev agent writes this near-verbatim. Rustfmt's canonical formatting is allowed to adjust whitespace/trailing commas. Feel free to split the test module to a separate line if rustfmt wants.

```rust
//! Top-level application states.
//! Registered via `App::init_state::<GameState>()` in `main.rs`.

use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Arena,
    Caravan,
    PostRun,
    PhotoMode,
    Paused,
}

pub fn log_loading_entered() {
    info!("entered Loading");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_loading() {
        assert_eq!(GameState::default(), GameState::Loading);
    }
}
```

### `src/main.rs` Skeleton (post-edit)

```rust
//! asteroids3D — app entry point.
//! Registers DefaultPlugins and GameState.

use bevy::prelude::*;

mod state;

use state::{log_loading_entered, GameState};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Loading), log_loading_entered)
        .run()
}
```

Order in the builder chain is conventional: `add_plugins(DefaultPlugins)` first (because `init_state` depends on Bevy's schedule configuration from MinimalPlugins/DefaultPlugins), then `init_state`, then `add_systems`. Same order appears in all Bevy 0.18 example code for stateful apps.

### Why a single unit test, not an integration test

`App::new().add_plugins(DefaultPlugins)` boots wgpu/winit on construction — this breaks headless CI runners. Bevy's own testing guidance for stateful logic is to test pure functions (`GameState::default()`) and delegate runtime-integration assertions to manual `cargo run` + log grep. An integration test that constructs an `App` with MinimalPlugins + our state + asserts `State<GameState>::get() == Loading` *would* work on CI but it's disproportionate effort for a 1-enum story and doesn't exercise anything `default()` doesn't. Revisit at the first gameplay story (Epic 3) where state-transition logic exists and deserves coverage.

[Source: architecture.md:144-146 "Bevy-integration tests remain post-M3"; Story 1.5 testing-requirements same reasoning]

### Scope boundaries — what belongs to later stories

| Concern | Story that owns it |
|---|---|
| `SplashConfig` resource | **Story 1.7** — splash-duration config, default 2.0s. |
| `"asteroids3D"` text node spawned on `OnEnter(Loading)` | **Story 1.7** — `bevy_ui` text + `LoadingStateEntity` marker + centered flexbox. |
| `NextState<GameState>(GameState::MainMenu)` transition | **Story 1.7** — timer-driven `Loading → MainMenu`. |
| Entity cleanup via `cleanup_on_exit::<LoadingStateEntity>` | **Story 1.7** — `OnExit(Loading)` despawn. |
| Any MainMenu UI content | **Epic 3+** — `epic-1*.md:170` explicitly says "MainMenu UI is a later epic's responsibility". |
| Sub-states (`MainMenu::Title` / `MainMenu::Settings`) | **Epic 3+ or later** — nested state hierarchy mentioned in architecture.md:210; not M0. |
| `tracing_subscriber::fmt().init()` + panic hook + user-log-dir file | **Story 1.8** — keeps relying on Bevy's built-in `LogPlugin` until then. |
| `Cargo.toml` additions | **None in this story.** No new deps. |

### Architecture Compliance

- **`src/state.rs` lives at the single-binary top level** — matches `architecture.md:549` ("`state.rs`: GameState enum + phase SystemSets (AppPhase)"). This story creates the file; `AppPhase` SystemSets land with the first cross-plugin ordering need (post-M1). [Source: architecture.md:549]
- **States variant set matches architecture.md:210 + the `Loading` addition** — architecture.md:210 lists `MainMenu, Arena, Caravan, PostRun, PhotoMode, Paused` but omits `Loading`. Epic 1 spec (epic-1-*.md:134) explicitly adds `Loading` as the asset-load + splash-screen holding state. The spec-refinement is intentional: architecture enumerates the "gameplay-relevant" states; Epic 1 extends with the startup/loading state per architecture.md:432-434 ("Startup Sequencing: `OnEnter(GameState::Loading)`: kick off asset loads, show splash"). No architecture erratum needed — the Epic correctly extends the state set for M0. [Source: architecture.md:210 + architecture.md:432-434 + epic-1-*.md:134]
- **`init_state` (not `insert_state`)** — `init_state` uses `GameState::default()` which respects the `#[default]` attribute. `insert_state(GameState::Loading)` would work identically here but is for cases where the initial state is not the Default variant. `init_state` is idiomatic when Default + `#[default]` agree with the desired initial state. [Source: Bevy 0.18 state docs convention]
- **No `NextState` mutation** — AC #3 explicitly forbids automatic transitions. The `NextState<GameState>` resource is registered by `init_state` but no system writes to it in this story. [Source: AC #3 + architecture.md:418]
- **`fn main() -> AppExit`** — resolves deferred-work.md:50 (Story 1.5 code-review Blind Hunter BH14 + Edge Case Hunter EC1). Bevy 0.18's `App::run() -> AppExit` crash-signal is now propagated. [Source: deferred-work.md:50]

### Library/Framework Requirements

No new crates. No `Cargo.toml` edits.

| Crate | Version | Status after Story 1.6 |
|---|---|---|
| `bevy` | `0.18` | Unchanged; this story imports `States, State, NextState, OnEnter, AppExit` from `bevy::prelude`. |
| All other pinned deps | unchanged | Still unused in code (Epic 2+). |

### File Structure Requirements

| Path | Add/Modify | Purpose |
|---|---|---|
| `src/state.rs` | **Add** | `GameState` enum (7 variants) + `log_loading_entered` debug system + 1 unit test. ~25 lines. |
| `src/main.rs` | **Modify** | Add `mod state;` + `use state::...;` + `init_state::<GameState>()` + `add_systems(OnEnter(GameState::Loading), log_loading_entered)` + change `fn main()` → `fn main() -> AppExit` + return `run()` value. Net ~6 line additions. |
| `Cargo.toml` | **Do NOT touch** | No new deps. |
| `Cargo.lock` | **Do NOT touch** | No dep graph change. |
| `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` | **Do NOT touch** | Out of scope. |
| `_bmad-output/implementation-artifacts/deferred-work.md` | **Modify** | Append `✅ RESOLVED 2026-04-24 by Story 1.6` note to the `App::run() -> AppExit` entry (deferred-work.md:50). Match the 1.5 resolution-note style (line 33). Preserve the historical body below the note. |

**Single-binary layout note:** the project is currently a single-binary crate with no `lib.rs`. `src/state.rs` is a direct child module declared via `mod state;` in `main.rs`. When future stories need shared types across plugins (core/faction/damage per architecture.md:550-554), a `src/lib.rs` may be introduced — but that's a refactor, not a Story 1.6 concern.

### Testing Requirements

- **Unit test:** 1 test (`default_state_is_loading`) in `src/state.rs`. Pure function, zero I/O, CI-safe.
- **Manual test:** `cargo run` on macOS + log grep for `entered Loading`.
- **Integration test:** deferred (see "Why a single unit test" above). No `App`-construction tests in this story.
- **Windows/Linux runtime verification:** same pattern as Story 1.5 — Till runs `cargo run` on each physical OS and confirms `entered Loading` in the log. Can be Till-ran in parallel with the commit + CI flow or deferred to a future machine-access window; CI itself only verifies compile parity.

### Latest Technical Information

**Bevy 0.18 States API — no breaking changes relevant to this story.** The API surface (`States` trait, `#[derive(States)]`, `#[default]` attribute, `init_state::<T>()`, `State<T>`, `NextState<T>`, `OnEnter`/`OnExit` schedules) has been stable since Bevy 0.13. Bevy 0.18's `AppExit` enum (`AppExit::Success` / `AppExit::Error(NonZeroU8)`) is also stable across recent releases. No version-specific gotchas.

**`#[derive(States)]` vs manual `impl`:** the derive macro is the only recommended path since Bevy 0.12. Manual `impl States for GameState` is not documented and not maintained.

**`AppExit` propagation:** `fn main() -> AppExit` matches Bevy's own examples (`bevy/examples/app/empty.rs` since 0.13). No custom process-exit-code mapping needed — `AppExit` implements `Termination`.

### Previous Story Intelligence

**From Story 1.5 (just closed, commit `8096ff0`):**

- **Two-commit pattern:** source artifacts first (`feat:`), BMad bookkeeping second (`bmad:`). Pushed together or in close succession. Story 1.6 follows this.
- **Exit-0 is not proof:** MEMORY `feedback_full_build_output.md` explicitly rejects "cargo check exit 0 + tail looks clean" as verification. Task 5 uses `tee` + explicit `grep` for every command — paste line counts into DAR.
- **Module doc style:** Story 1.5 review-patch BH8 removed "Story 1.5 scope." from `src/main.rs`'s module doc. Story 1.6's doc says what the file IS, not which story introduced it. Keep `//!` under 2 lines.
- **Adapter log on macOS:** Expected line is `backend: Metal, "Apple M5 Pro"` (or similar, depending on hardware). Story 1.5 captured it; Story 1.6 should re-capture as drift-check.
- **CI cold-cache cost:** Story 1.5's CI run (24842252974) took 62m37s total because the Cargo.lock change invalidated `Swatinem/rust-cache` keys. Story 1.6 does NOT touch Cargo.lock, so cache should be warm → expect ~15-25m total CI runtime. If CI takes > 40m, something's off.
- **`tracing` is Bevy-wired via `LogPlugin`:** Story 1.5 Dev Notes documented this. `info!("entered Loading")` works with zero `tracing_subscriber` setup because `DefaultPlugins` includes `LogPlugin`. Do NOT add a subscriber init — Story 1.8 owns that.
- **Scope guardrails enforced via `git status --short` + targeted `grep`:** Story 1.5's Task 7 pattern. Story 1.6 Task 6 mirrors it.

**From Story 1.5's Deferred Follow-ups (inherited):**

- `.claude/scheduled_tasks.lock` gitignore addition (deferred-work.md:51) — **still deferred, not Story 1.6's job.** Story 1.6 guardrails forbid touching `.gitignore`.
- The architecture.md:256 prescribes-broken-`cfg(debug_assertions)` pattern (deferred-work.md:48) — **still deferred** to M2 debug-panels story.
- Story 1.2 `bevy_egui` addendum + planning-artifact doc drift (deferred-work.md:47-49) — **still deferred** to dedicated doc-sync chore story.

### Git Intelligence

Recent relevant commits (last 5):
```
8096ff0 bmad: story 1.5 review complete — 2 patches applied, 5 defers logged
05003d4 chore: apply code-review patches (Story 1.5)
03eb7a4 feat: minimal Bevy app + remove bevy_egui from Cargo.toml (Story 1.5)
011f99d bmad: story 1.4 review complete — 3 patches applied, 10 defers logged
3f3d5f2 ci: add timeout + DEBIAN_FRONTEND + --locked (Story 1.4 review patches)
```

**Patterns reinforced:**
- Single-line commit subjects, sub-70-char. Story-ID in parentheses at end when semantically useful.
- Prefix vocabulary: `feat:` (new code), `chore:` (refactor/cleanup), `docs:` (markdown-only), `fix:` (bug fix), `ci:` (CI config), `bmad:` (BMad bookkeeping). Story 1.6 uses `feat:`.
- No `Co-Authored-By` trailer — Till's convention.
- BMad bookkeeping always in a separate commit from source changes (clean diffs + easy revert).

### Project Structure Notes

Alignment with architecture.md file layout:
- `src/main.rs` — ✅ `App::new()` assembly, plugin + state registration. [architecture.md:548]
- `src/state.rs` — ✅ `GameState` enum. Architecture.md:549 also says "phase SystemSets (AppPhase)", but `AppPhase` is not defined here (post-M1 concern when cross-plugin ordering needs arise). Story 1.6 installs the first of two responsibilities attributed to this file.
- No other file changes → no drift.

No conflicts with the rest of the unified project structure (architecture.md:546-630). Future `src/core/`, `src/tuning/`, `src/flight/`, etc. are Epic 3+ and untouched here.

### References

- [Source: `_bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md:124-143`] — Story 1.6 spec with full AC list.
- [Source: `_bmad-output/planning-artifacts/architecture.md:210`] — Top-level States enumeration (Loading to be added per Epic 1 extension).
- [Source: `_bmad-output/planning-artifacts/architecture.md:294`] — M0 deliverables include States skeleton.
- [Source: `_bmad-output/planning-artifacts/architecture.md:417-420`] — State-transition patterns (load-bearing for future stories but only relevant as "what NOT to do in 1.6").
- [Source: `_bmad-output/planning-artifacts/architecture.md:432-434`] — `OnEnter(GameState::Loading)` startup sequencing prescription.
- [Source: `_bmad-output/planning-artifacts/architecture.md:548-549`] — `src/main.rs` + `src/state.rs` placement + purpose.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md:50`] — `App::run() -> AppExit` deferred-to-Story-1.6 assignment.
- [Source: `_bmad-output/implementation-artifacts/1-5-minimal-bevy-app-opens-a-window-on-all-three-platforms.md` — entire Dev Notes section] — prior-story conventions for Cargo, CI, verification, commit style.
- [Source: `MEMORY.md → feedback_full_build_output.md`] — verification discipline: exit-0 + tail is NOT proof; grep explicitly.
- [Source: `MEMORY.md → feedback_staged_rollout.md`] — no speculative scaffolding (applies to NOT pre-declaring `SplashConfig` / `AppPhase` / sub-states here).

## Dev Agent Record

### Agent Model Used

claude-opus-4-7[1m] (Claude Code, 1M context)

### Debug Log References

All local verification logs captured at `/tmp/story-1-6-*.log` on macOS 26.4.1 / Apple M5 Pro / arm64.

| Command | Grep pattern | Hit count | Notes |
|---|---|---|---|
| `cargo check` | `warning:\|error:` | **0** | Clean after `#[allow(dead_code)]` added to enum |
| `cargo build` | `warning:\|error:` | **0** | `Finished dev profile ... in 2.45s` |
| `cargo test` | `warning:\|error:\|FAILED` | **0** | `1 passed; 0 failed; 0 ignored` — `state::tests::default_state_is_loading ... ok` |
| `cargo clippy --all-targets -- -D warnings` | `warning:\|error:` | **0** | Clean |
| `cargo fmt --all -- --check` | (exit code) | **0** | (rustfmt auto-reordered `use state::{GameState, log_loading_entered};` once — applied and re-verified) |
| `cargo run` → `grep 'entered Loading'` | `entered Loading` | **1** | Exactly one info-log line: `INFO asteroids3D::state: entered Loading` |
| `cargo run` → `grep 'backend:'` | `AdapterInfo\|backend:` | **1** | `AdapterInfo { name: "Apple M5 Pro", ..., backend: Metal }` — parity with Story 1.5 |
| `git status --short` post-impl | (scope check) | **exact** | Only `src/main.rs` (M) + `src/state.rs` (??) prior to commit. `Cargo.toml`, `Cargo.lock`, CI, `.gitignore` untouched. |
| `grep -nE 'NextState\|MainMenu' src/` | (guardrail) | **1** | Only `src/state.rs:11: MainMenu,` (enum variant). No `NextState<GameState>` mutation. |
| `grep -n 'SplashConfig\|LoadingStateEntity' src/` | (guardrail) | **0** | Markers belong to Story 1.7 — confirmed not pre-declared. |

**CI run `24880100462`** (push `19ed03c` to `origin/master`):
- 4/4 jobs ✅: `build (ubuntu-latest)`, `build (macos-latest)`, `build (windows-latest)`, `msrv-check (rust 1.89, ubuntu-latest)`
- Slowest job: `build (windows-latest)` in 5m 0s. Total wall time ≈ 5m. Story Dev Notes predicted ~15–25m with warm cache — actual was faster (no Cargo.lock touch kept `Swatinem/rust-cache` fully hot).
- `gh run view 24880100462 --log | grep -E 'warning:|error:' | wc -l` → **0** across 2059 lines.
- CI annotations: 2× Node.js 20 deprecation warnings on `actions/checkout@v4` (third-party action pinning defer `deferred-work.md:20` — not Story 1.6 scope).

**Runtime observations (non-blocking):**
- `WARN bevy_winit::state: Skipped event Destroyed for unknown winit Window Id WindowId(...)` — emitted on window close. Known Bevy 0.18 runtime trace pattern, not a Story 1.6 code issue. Captured in `/tmp/story-1-6-run.log` for audit trail.

### Completion Notes List

- **AC #1 — `src/state.rs` with `GameState` enum:** 7 variants in prescribed order (`Loading, MainMenu, Arena, Caravan, PostRun, PhotoMode, Paused`), `#[default]` on `Loading`, derive list exactly `(States, Default, Debug, Clone, Eq, PartialEq, Hash)`. Module doc ≤2 lines, no story reference (honors 1.5 review-patch BH8).
- **AC #2 — `App::init_state::<GameState>()` boots default `Loading`:** Wired in `src/main.rs` between `add_plugins(DefaultPlugins)` and `add_systems(...)`. Default variant is `Loading` (implicit via `#[default]`). First-frame state verified via Bevy's `OnEnter(Loading)` → `log_loading_entered` → `INFO` line in `/tmp/story-1-6-run.log`.
- **AC #3 — Debug system logs "entered Loading" once, no further transitions:** `grep 'entered Loading'` = 1 hit. `grep -nE 'NextState|MainMenu' src/` returns only the enum variant declaration — no `NextState<GameState>` writer exists in code, confirming no automatic transition (Story 1.7's concern).
- **AC #4 — `fn main() -> AppExit` propagates `App::run()`'s return value:** Signature changed, trailing `;` removed on `.run()`. Resolves `deferred-work.md:50` (Story 1.5 BH14/EC1). Resolution note appended to `deferred-work.md` (2026-04-24).
- **Unavoidable spec deviation (non-breaking):** Added `#[allow(dead_code)]` to `GameState` enum — required because the `-D warnings` clippy gate would otherwise flag the 6 non-default variants as never-constructed. Story skeleton did not foresee this (7 variants defined at once, only 1 used by `init_state::<T>()::default()`). Comment explains: "non-default variants become live as state transitions land in later stories". Guardrail-compliant: no `NextState`/`MainMenu` references outside the enum declaration.
- **Rustfmt drift (non-breaking):** `cargo fmt --all` auto-reordered the `use state::{...};` import to `{GameState, log_loading_entered}` (uppercase-first). Story skeleton had the reverse order. Applied rustfmt's canonical order; Dev Notes allows rustfmt whitespace/comma adjustments.
- **First unit test in project:** `state::tests::default_state_is_loading` — pure function, no `App` construction, CI-safe on headless. `msrv-check` job's `--all-targets` gap (`deferred-work.md:21`) remains deferred per story scope.
- **Windows/Linux runtime log verification:** Deferred to Till's physical hardware per Story 1.5's model (local dev is macOS-only). CI compile parity on 3 OSes is the gating evidence.
- **Bundled resolution:** `deferred-work.md:50` (`App::run() -> AppExit`) marked ✅ RESOLVED 2026-04-24 by Story 1.6 with audit-trail note.

### File List

**Added:**
- `src/state.rs` — 7-variant `GameState` enum + `log_loading_entered` system + unit test. 32 lines.

**Modified:**
- `src/main.rs` — `mod state;` + use-import + `init_state::<GameState>()` + `add_systems(OnEnter(Loading), log_loading_entered)` + `fn main() -> AppExit`. Net +12 lines (8 → 16).
- `_bmad-output/implementation-artifacts/deferred-work.md` — ✅ RESOLVED 2026-04-24 note appended to the `App::run() -> AppExit` entry (line 50). Historical body preserved.
- `_bmad-output/implementation-artifacts/1-6-gamestate-enum-with-bevy-states-skeleton.md` — this file: Tasks checked, DAR/Completion Notes/File List/Change Log populated, Status → review.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status flip `ready-for-dev → in-progress → review` + `last_updated` bump.

**Untouched (guardrail):** `Cargo.toml`, `Cargo.lock`, `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`.

## Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-24 | claude-opus-4-7 (create-story) | Story 1.6 drafted. Scope: `src/state.rs` with 7-variant `GameState` enum + 1 unit test + debug `log_loading_entered` system; `src/main.rs` wiring (`mod state;` + `init_state::<GameState>()` + `add_systems(OnEnter(Loading), log_loading_entered)`) + bundled fix `fn main() -> AppExit` (inherited from Story 1.5's `deferred-work.md:50`). No `Cargo.toml` / `Cargo.lock` changes. Zero new deps. First project unit test lands here. Windows/Linux runtime verification parallels Story 1.5's model (Till's physical hardware). Status: ready-for-dev. |
| 2026-04-24 | claude-opus-4-7 (dev-story) | Story 1.6 implemented. Source commit `19ed03c` — `src/state.rs` added (32 lines), `src/main.rs` modified (+12 lines). Bundled `deferred-work.md:50` (`App::run() -> AppExit`) resolved. One unavoidable spec deviation: `#[allow(dead_code)]` added to enum (6 non-default variants would otherwise fail `-D warnings` clippy). Rustfmt auto-reordered import list once. All 4 CI jobs green (run `24880100462`, 0 warning/error across 2059 log lines). First project unit test passes. Status: review. |
| 2026-04-24 | claude-opus-4-7 (code-review light) | Story 1.6 reviewed. Scope: 62 diff lines over 2 files — light single-reviewer pass. Zero blocking, zero HIGH findings. 1× MED patched (`#[allow(dead_code)]` → `#[expect(dead_code, reason = "...")]` for self-cleaning lint; stable since Rust 1.81, MSRV 1.89 OK). 1× LOW logged to `deferred-work.md` (`bevy_winit` WindowClose WARN, known Bevy 0.18 race). Status: done. |

## Senior Developer Review (AI) — 2026-04-24

**Reviewer:** claude-opus-4-7 (light single-reviewer pass, no parallel adversarial agents)
**Review Date:** 2026-04-24
**Diff:** `git diff 8096ff0..19ed03c` — 2 files, +43/-4 lines
**Outcome:** ✅ Approve (with 1 patch applied + 1 LOW defer)

### Review Rationale for Light Mode

Full 3-agent adversarial review (Blind Hunter / Edge Case Hunter / Acceptance Auditor) was deemed disproportionate for a 62-line diff where ~90% of the code follows the Story 1.6 skeleton verbatim. Light review focused on the three deviations from the verbatim spec: (1) `#[allow(dead_code)]` added to enum, (2) runtime `WARN` observed on window close, (3) CI Node.js 20 deprecation annotation.

### Findings

| ID | Severity | Area | Resolution |
|---|---|---|---|
| MED-1 | MED | `src/state.rs:7` — `#[allow(dead_code)]` attribute | **✅ Applied** — swapped to `#[expect(dead_code, reason = "non-default variants become live as state transitions land in later stories")]`. Self-cleaning: fires a rustc "lint expectation not fulfilled" warning once all variants become live, forcing removal. Stable since Rust 1.81, MSRV 1.89 OK. |
| LOW-1 | LOW | `bevy_winit` runtime WARN on window close | **✅ Deferred** — logged to `deferred-work.md` as "Deferred from: code review of 1-6" section. Known Bevy 0.18 winit-event race (`WindowCloseRequested` vs `Destroyed`); not Story-1.6-introduced, not reproducible in CI. Re-evaluate at M4 Bevy version bump. |
| LOW-2 | LOW | CI Node.js 20 deprecation on `actions/checkout@v4` | **No-op** — already tracked in `deferred-work.md:20` (Story 1.4 review finding "Third-party action pinning"). |

### Acceptance Criteria Verification

| AC | Evidence | Status |
|---|---|---|
| #1 — `GameState` enum + derives | `state.rs:6-16`, derive list exactly `(States, Default, Debug, Clone, Eq, PartialEq, Hash)`, `#[default]` on `Loading`, 7 variants in prescribed order | ✅ |
| #2 — `init_state::<GameState>()` → first-frame `Loading` | `main.rs:13` + `/tmp/story-1-6-run.log` → `INFO asteroids3D::state: entered Loading` | ✅ |
| #3 — Debug system logs, no further transitions | `grep 'entered Loading'` = 1 hit; `grep -nE 'NextState\|MainMenu' src/` → only enum variant declaration | ✅ |
| #4 — `fn main() -> AppExit` propagates | `main.rs:10-16`, no trailing `;` on `.run()`, `deferred-work.md:50` marked RESOLVED | ✅ |

### Scope + Guardrail Audit

- `Cargo.toml`, `Cargo.lock`, `.gitignore`, `.github/workflows/ci.yml`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` — all untouched ✅
- `grep SplashConfig|LoadingStateEntity src/` → 0 hits (Story 1.7 markers not pre-declared) ✅
- Module docs ≤2 lines, no story-id references (honors 1.5 review-patch BH8) ✅
- Two-commit pattern (source + bookkeeping) preserved ✅

### Test Quality

- Single unit test `default_state_is_loading` — precise Default contract, no `App` construction, CI-safe on headless. Integration tests consciously deferred per Dev Notes §"Why a single unit test" and architecture.md:144-146. Acceptable scope.

### Action Items

- [x] Apply MED-1 patch: `#[allow]` → `#[expect]` with reason
- [x] Log LOW-1 defer to `deferred-work.md`
- [x] Re-verify `cargo check/clippy/test/fmt` post-patch — all clean
