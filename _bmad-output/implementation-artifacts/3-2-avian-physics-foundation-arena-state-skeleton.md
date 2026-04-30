# Story 3.2: Avian Physics Foundation + Arena State Skeleton

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want Avian XPBD registered in the fixed-step schedule at 60 Hz with gravity disabled, plus a `GameState::Arena` skeleton with state-scoped lifecycle hooks,
So that subsequent flight and combat stories (3.3–3.11) attach to a deterministic physics world and a clean state-cleanup contract — without re-deriving either later.

## Acceptance Criteria

1. **Given** the project pre-3.2 has no physics dependency wired into `App::new()`
   **When** Story 3.2 lands
   **Then** `avian3d::prelude::PhysicsPlugins::default()` is registered via `App::add_plugins(PhysicsPlugins::default())` in `src/main.rs` (placed alongside the other gameplay plugins, AFTER `DefaultPlugins` registration)
   **And** Bevy's fixed-timestep is set to **60 Hz** via `app.insert_resource(Time::<Fixed>::from_hz(60.0))` (Avian 0.6's `PhysicsPlugins` runs inside Bevy's `FixedMain` schedule — specifically `FixedPostUpdate` — and inherits the `Time<Fixed>` rate; the architecture decision "Avian in FixedUpdate at 60 Hz" maps to this 60 Hz `Time<Fixed>` configuration in Avian 0.6 idiom)
   **And** `Gravity(Vec3::ZERO)` is inserted as a Resource via `app.insert_resource(Gravity(Vec3::ZERO))` (zero-g space environment per FR2 + brainstorming Phase 3 design — no constant downward acceleration; thrusters are the sole motion source)

2. **Given** `src/arena/` directory does not exist pre-3.2
   **When** Story 3.2 lands
   **Then** `src/arena/mod.rs` is authored (≤ 60 lines including module doc + tests if any)
   **And** the file declares `pub struct ArenaPlugin` implementing `bevy::prelude::Plugin`
   **And** the file declares `#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)] pub enum ArenaSystems` with at least one variant (initial: `Setup` for spawn-time work in Story 3.3+; `Cleanup` is NOT a variant — cleanup runs on `OnExit` schedule, not the `ArenaSystems` set inside `Update`/`FixedUpdate`)
   **And** the file declares `#[derive(Component)] pub struct ArenaEntity;` — pure tag, no fields, `pub` so future arena-scoped systems (3.3 zone, 3.5 PlayerShip, 3.9 projectiles, 3.11 HUD) can insert it on their spawned entities

3. **Given** `ArenaPlugin::build` is implemented
   **When** the plugin is added to the app
   **Then** the plugin registers a generic `cleanup_on_exit::<ArenaEntity>` system on `OnExit(GameState::Arena)` (despawn pattern matches `cleanup_main_menu` from Story 3.1: `for e in &query { commands.entity(e).despawn(); }` — root-only marking, children cascade via Bevy 0.18 `ChildOf` linked-despawn)
   **And** `ArenaPlugin` is registered in `src/main.rs` via `App::add_plugins(ArenaPlugin)` (placed after `UiPlugin` registration, before the splash systems)
   **And** the existing `OnEnter(GameState::Arena)` registration of `log_arena_entered` from Story 3.1 STAYS in `main.rs` (the `info!("entered Arena")` log line is the satisfying signal for the AC #3 spec text "an `info!` log `\"entered Arena\"` is emitted"; ArenaPlugin does NOT add a duplicate log)

4. **Given** the Arena state is exited (transition `Arena → MainMenu` or `Arena → Paused` — both arrive in Epic 4+; not exercised in 3.2 itself)
   **When** `OnExit(GameState::Arena)` runs
   **Then** every `Entity` in `Query<Entity, With<ArenaEntity>>` is despawned (currently zero entities; Story 3.3 spawns the first batch of `ArenaEntity`-marked asteroids + DirectionalLight, and Story 3.5 adds the PlayerShip + cockpit Camera3d)
   **And** the cleanup system is generic (`fn cleanup_on_exit<T: Component>(...)`) so the same function definition serves Stories 3.4 (`PauseOverlayEntity`), 3.11 (`HudEntity`), and any future state-scoped marker — defined ONCE in `src/arena/mod.rs` (or a more general home if structurally cleaner; see Dev Notes "Generic vs. arena-specific cleanup" decision)

5. **Given** the post-Story-3.1 source baseline (test count = 14; `cargo build --release` 0 warnings; main.rs ~50 lines)
   **When** Story 3.2 verification runs
   **Then** the local sweep produces **0** lines matching `grep -cE 'warning:|error:'` for each of: `cargo check`, `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build --release`
   **And** `cargo fmt --all -- --check` exits 0
   **And** `cargo run` (no env var) opens a window, transitions through `Loading (~2 s splash) → MainMenu (visible title + subtitle) → Arena (blank window after Enter press)` AND no Avian-emitted runtime warnings appear in `/tmp/story-3-2-run.log` (specifically: no `WARN bevy_ecs::change_detection: ...PhysicsPlugins`, no `panic`, no `backtrace`, no `wgpu`-level errors beyond known macOS warnings already documented in Story 1.6 deferred-work LOW-1)
   **And** `cargo test` summary line reads exactly `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — Story 3.2 ships **zero new tests** (rationale in Dev Notes "Test count discipline"; 3.3+ asteroid-spawn integration is the natural test-target landing)
   **And** `git status --short` final set after dev work is exactly: `Cargo.toml` (NOT modified; avian3d is already pinned at `0.6` per Cargo.toml:9), `Cargo.lock` (NOT modified; no dep-tree change), `src/main.rs` (M), `src/arena/mod.rs` (??), plus bookkeeping `_bmad-output/implementation-artifacts/sprint-status.yaml` (M) and this story file (M). NO entries under `Cargo.toml`, `Cargo.lock`, `src/state.rs`, `src/visual/**`, `src/ui/**`, `src/tuning/**`, `assets/`, `docs/`, `.github/workflows/**`.

## Tasks / Subtasks

- [ ] **Task 1: Author `src/arena/mod.rs` — ArenaPlugin skeleton + ArenaEntity marker + ArenaSystems set + generic cleanup** (AC: #2, #3, #4)
  - [ ] Create `src/arena/` directory at the repo root (sibling of `src/ui/`).
  - [ ] Create `src/arena/mod.rs`. Target ≤ 60 lines including module doc.
  - [ ] Module doc 2 lines max, no story-id references (per Story 1.5 review patch BH8 + Story 1.7/3.1 dev-notes precedent).
  - [ ] Skeleton:
    ```rust
    //! ArenaPlugin — owns GameState::Arena entity lifecycle (spawn / cleanup).
    //! Story 3.3 attaches asteroid spawning; later stories add player ship, projectiles, HUD.

    use bevy::prelude::*;

    pub struct ArenaPlugin;

    #[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
    pub enum ArenaSystems {
        Setup,
    }

    #[derive(Component)]
    pub struct ArenaEntity;

    impl Plugin for ArenaPlugin {
        fn build(&self, app: &mut App) {
            app.configure_sets(
                OnEnter(crate::state::GameState::Arena),
                ArenaSystems::Setup,
            );
            app.add_systems(
                OnExit(crate::state::GameState::Arena),
                cleanup_on_exit::<ArenaEntity>,
            );
        }
    }

    pub fn cleanup_on_exit<T: Component>(
        mut commands: Commands,
        query: Query<Entity, With<T>>,
    ) {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    }
    ```
  - [ ] Rationale for `pub fn cleanup_on_exit<T: Component>`: architecture.md:420 prescribes a `cleanup_on_exit::<T>` pattern for ALL state-scoped markers. Story 3.4 needs `cleanup_on_exit::<PauseOverlayEntity>`, Story 3.11 needs `cleanup_on_exit::<HudEntity>`. Defining once here (and re-using via the generic) keeps the codebase DRY. Living it inside `src/arena/mod.rs` is acceptable for now; if a third+ consumer arrives in Epic 4+ AND `arena/` feels semantically wrong as the home, a future story can move it to `src/core/cleanup.rs` (architecture.md:550 reserves `src/core/` for shared types). Don't pre-create `src/core/` in 3.2 — YAGNI; the generic at `src/arena/mod.rs` is reachable via `crate::arena::cleanup_on_exit::<T>`.
  - [ ] Rationale for `ArenaSystems::Setup` only (no `Cleanup` variant): architecture.md:347 prescribes `<Feature>Systems` enum "for ordering". `Setup` runs in `OnEnter(Arena)` (Stories 3.3 zone-spawn, 3.5 PlayerShip-spawn — these will need ordering once both exist; 3.2 declares the set so 3.3 can `.in_set(ArenaSystems::Setup)` without a follow-up patch). `Cleanup` is NOT in the set because cleanup runs in `OnExit(Arena)` schedule, which is a separate ECS schedule — ordering it via the same set would be a category error. Future runtime-systems variants (e.g., `Update` ordering) can be added when the first runtime arena system arrives (likely 3.6+ flight input or 3.8 dampener).
  - [ ] **Pattern alignment with VisualSystems and TuningSystems:** `VisualSystems::Setup` is configured on `OnEnter(Loading)` (currently empty post-3.1 cleanup); `TuningSystems::Reload` is configured on `Update`. `ArenaSystems::Setup` configured on `OnEnter(Arena)` is the parallel idiom for arena-scoped state-entry work — semantics identical to the existing patterns the dev has already seen. [Source: src/visual/mod.rs:23-26, src/tuning/mod.rs:29]
  - [ ] **No tests** in `src/arena/mod.rs`. The plugin has no testable behavior in 3.2 (the marker is a unit struct, the generic cleanup wraps a 4-line query loop, the SystemSet is a derive-generated enum). Story 3.3 will be the natural integration-test target when actual asteroid spawning behavior exists; per architecture.md:354, integration tests are deferred post-M3 anyway.

- [ ] **Task 2: Wire physics + ArenaPlugin into `src/main.rs`** (AC: #1, #3)
  - [ ] Add `mod arena;` after `mod ui;` (rustfmt may reorder; accept its order).
  - [ ] Add `use arena::ArenaPlugin;` after `use ui::UiPlugin;` (rustfmt alphabetization).
  - [ ] Add the avian3d prelude import: `use avian3d::prelude::{Gravity, PhysicsPlugins};` — placed near the other top-level `use` statements, sorted by rustfmt. **Do NOT** import the entire prelude (`use avian3d::prelude::*`) — selective import keeps the symbol surface small and matches the project's existing import discipline (the Bevy `prelude::*` is the only wildcard import in main.rs at present).
  - [ ] Inside `fn main()`, AFTER `App::new().add_plugins(default_plugins).init_state::<GameState>()`, AND BEFORE the existing `.add_plugins(TuningPlugin)` chain entry:
    - Add `.add_plugins(PhysicsPlugins::default())`.
    - Add `.insert_resource(Time::<Fixed>::from_hz(60.0))`.
    - Add `.insert_resource(Gravity(Vec3::ZERO))`.
  - [ ] Add `.add_plugins(ArenaPlugin)` AFTER the existing `.add_plugins(UiPlugin)` line, BEFORE `.init_resource::<SplashConfig>()`.
  - [ ] **Resulting plugin-registration block (rustfmt-tolerant):**
    ```rust
    App::new()
        .add_plugins(default_plugins)
        .init_state::<GameState>()
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Gravity(Vec3::ZERO))
        .add_plugins(TuningPlugin)
        .add_plugins(VisualPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ArenaPlugin)
        .init_resource::<SplashConfig>()
        // ... existing add_systems chain unchanged below
    ```
  - [ ] **Why physics resources go in `main.rs`, not `ArenaPlugin`:** physics is a global concern (any state may need it; future Caravan state in Epic 6 will share the same physics world). `ArenaPlugin` owns Arena-state lifecycle, NOT engine-wide setup. This matches architecture.md:660-664 (Cross-Cutting Resources registered in `main.rs`). Architecturally identical to how Bevy's `DefaultPlugins` lives in `main.rs` rather than inside any feature plugin.
  - [ ] **Why register physics BEFORE TuningPlugin/VisualPlugin:** order is documented Bevy convention — engine plugins (DefaultPlugins, PhysicsPlugins) before feature plugins. Functionally not load-bearing in this story (no plugin in 3.2 reads PhysicsPlugins-emitted events at startup), but matches idiomatic Bevy app structure and avoids future-story re-ordering churn.
  - [ ] Net delta to `main.rs`: +6 lines (`mod arena;`, `use arena::ArenaPlugin;`, `use avian3d::prelude::{Gravity, PhysicsPlugins};`, `add_plugins(PhysicsPlugins::default())`, `insert_resource(Time::<Fixed>::from_hz(60.0))`, `insert_resource(Gravity(Vec3::ZERO))`, `add_plugins(ArenaPlugin)`). File grows from ~50 to ~56 lines.

- [ ] **Task 3: Local verification sweep — full build + runtime smoke** (AC: #5)
  - [ ] **`cargo check`:**
    ```bash
    cargo check 2>&1 | tee /tmp/story-3-2-check.log
    grep -cE 'warning:|error:' /tmp/story-3-2-check.log
    ```
    Expected: `0`. If non-zero, the most likely culprits are: (a) `Time::<Fixed>::from_hz` not in scope — Bevy 0.18 has it as `Time::<Fixed>::from_hz(60.0)` directly via `bevy::prelude::Time`/`bevy::time::Fixed` (verify import); (b) `Gravity` import path — the architecture.md text uses bare `Gravity`, the actual import is `avian3d::prelude::Gravity` (Avian re-exports it from `avian3d::dynamics::integrator`); (c) missing `mod arena;` line.
  - [ ] **`cargo build` (debug):**
    ```bash
    cargo build 2>&1 | tee /tmp/story-3-2-build.log
    grep -cE 'warning:|error:' /tmp/story-3-2-build.log
    ```
    Expected: `0`. **First-time avian3d compile may take several minutes** — Avian pulls in `parry3d`, `nalgebra`, `simba`, additional crates not previously in the build graph. This is a one-time cost; subsequent builds reuse the cache. **No spurious "warning" hits expected** — Avian 0.6 is a mature crate with clean clippy/rustc output as of April 2026.
  - [ ] **`cargo test`:**
    ```bash
    cargo test 2>&1 | tee /tmp/story-3-2-test.log
    grep -cE 'warning:|error:|FAILED' /tmp/story-3-2-test.log
    ```
    Expected: `0`. Summary line MUST read **exactly** `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Story 3.2 ships zero new tests; pre-3.2 baseline is 14 (post-3.1).
  - [ ] **`cargo clippy --all-targets -- -D warnings`:**
    ```bash
    cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/story-3-2-clippy.log
    grep -cE 'warning:|error:' /tmp/story-3-2-clippy.log
    ```
    Expected: `0`. **Particular vigilance** for `dead_code` on the `Setup` variant of `ArenaSystems` — Story 3.2 declares the variant but doesn't put any system in the set yet. Mitigation: the variant being part of a public enum used in `configure_sets` is sufficient; the variant itself doesn't need explicit consumers to escape `dead_code`. If clippy disagrees on your local toolchain, add `#[allow(dead_code, reason = "consumer arrives in Story 3.3 spawn_arena_zone")]` to the enum (or just to the `Setup` variant). Document in Completion Notes if applied.
  - [ ] **`cargo fmt --all -- --check`:**
    ```bash
    cargo fmt --all -- --check
    echo $?
    ```
    Expected exit: `0`. If non-zero, run `cargo fmt --all` once and re-check. Rustfmt may canonicalize the new `use` ordering in `main.rs` (`avian3d` is alphabetically before `bevy`; the existing `use bevy::prelude::*;` may move).
  - [ ] **`cargo build --release`:**
    ```bash
    cargo build --release 2>&1 | tee /tmp/story-3-2-release.log
    grep -cE 'warning:|error:' /tmp/story-3-2-release.log
    ```
    Expected: `0`. **Release LTO + codegen-units=1 (Cargo.toml:29-31) makes this slower** — first-time avian3d release-build can take 5–15 minutes. Once cached, incremental rebuilds are fast.
  - [ ] **`cargo run` runtime smoke (foreground or short-running background):**
    ```bash
    RUST_LOG=info,wgpu=warn,naga=warn,avian3d=info cargo run 2>&1 | tee /tmp/story-3-2-run.log &
    PID=$!
    sleep 3   # wait for splash (2 s) + MainMenu paint
    # send Enter via manual press in the focused window
    # ... after ~5–8 s total, close window manually or send SIGINT
    ```
  - [ ] **Log-grep evidence for runtime smoke:**
    ```bash
    grep -c 'entered Loading' /tmp/story-3-2-run.log     # expected: 1
    grep -c 'splash timer elapsed' /tmp/story-3-2-run.log  # expected: 1
    grep -c 'entered MainMenu' /tmp/story-3-2-run.log    # expected: 1
    grep -c 'MainMenu: Enter pressed, transitioning to Arena' /tmp/story-3-2-run.log  # expected: 1
    grep -c 'entered Arena' /tmp/story-3-2-run.log       # expected: 1
    grep -cE 'panic|backtrace|FATAL' /tmp/story-3-2-run.log  # expected: 0
    grep -E 'AdapterInfo|backend:' /tmp/story-3-2-run.log    # expected: backend: Metal on Apple M5 Pro
    ```
    All five lifecycle counts MUST be 1 (after a single Enter press); the panic-grep MUST be 0. **New for 3.2:** physics plugin emits no `entered Arena`-like log of its own; physics is silent at startup beyond optional Avian initialization info (gated by `avian3d=info` log filter).
  - [ ] **Visual verification (manual):** the MainMenu screen shows "asteroids3D" centered with "Press Enter to start" below it (unchanged from 3.1). After pressing Enter, the screen goes blank (Arena state has no rendering yet — that's Story 3.3). Window stays open, no visible regression vs. post-3.1 behavior. **Specifically:** no Avian-spawned diagnostic visualizers (Avian 0.6's `PhysicsDebugPlugin` is opt-in and is NOT registered in 3.2; if you see colliders or wireframes, you accidentally added the debug plugin — remove it).
  - [ ] **Tuning hot-reload non-regression check:** run `cargo run` in debug, then in another terminal edit `assets/config/tuning.ron` (e.g., bump `toon_steps` from 4 to 6), save, and observe `/tmp/story-3-2-run.log` for `TuningReloaded: toon_steps=6 ...`. Expected: 1 hit, confirming Story 3.1's `file_watcher`-only hot-reload still works after Avian dependency addition (no expected interaction; this is a sanity check that Cargo.lock changes from avian3d don't disturb the asset pipeline).

- [ ] **Task 4: Scope guardrails — verify nothing else drifted** (AC: #5)
  - [ ] `git status --short` final inspection. Expected file set:
    - `src/main.rs` (M) — Task 2.
    - `src/arena/mod.rs` (??) — Task 1.
    - `_bmad-output/implementation-artifacts/sprint-status.yaml` (M) — Task 5.
    - `_bmad-output/implementation-artifacts/3-2-...-md` (M) — Task 5 (this file's Status flip).
    - **NO** `Cargo.toml` (M), `Cargo.lock` (M), `.gitignore` (M), `src/state.rs` (M), `src/ui/**` (M), `src/visual/**` (M), `src/tuning/**` (M), `assets/**` (M), `docs/**` (M), `.github/workflows/**` (M), `rust-toolchain.toml` (M), `rustfmt.toml` (M), `clippy.toml` (M).
  - [ ] **Cargo.lock should NOT change.** `avian3d 0.6` is already pinned in `Cargo.toml:9` (since Story 1.1) and was already resolved into `Cargo.lock`. Story 3.2 adds USAGE of avian3d types but does not change which version is selected — Cargo.lock is byte-identical pre/post-3.2. **If `git status` shows `Cargo.lock` (M),** investigate: maybe a transitive dep of avian3d has a version range that re-resolved on touch; commit it in Commit 1 if so, but document the unexpected change.
  - [ ] `grep -rn 'avian3d\|PhysicsPlugins\|Gravity' src/ --include='*.rs'` → expected: **3+ hits** (the import + the two `insert_resource` calls + the `add_plugins` call in `src/main.rs`; possibly `Gravity` referenced in a doc-comment).
  - [ ] `grep -rn 'ArenaPlugin\|ArenaEntity\|ArenaSystems\|cleanup_on_exit' src/ --include='*.rs'` → expected: **5+ hits** (definitions in `src/arena/mod.rs`; `ArenaPlugin` import + add_plugins in `main.rs`).
  - [ ] **Files NOT touched (must remain byte-identical):** `Cargo.toml`, `Cargo.lock` (modulo the conditional above), `.gitignore`, `src/state.rs` (`log_arena_entered` already exists from 3.1), `src/splash.rs`, `src/logging.rs`, `src/ui/**`, `src/visual/**`, `src/tuning/**`, `assets/**`, `docs/**`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, all `_bmad-output/planning-artifacts/**`.

- [ ] **Task 5: Bookkeeping — story status flip + commit + push** (AC: all)
  - [ ] Populate this story file's **Dev Agent Record**: Agent Model Used, Debug Log References (per-command grep counts + log paths), Completion Notes (per-AC evidence + any deviations), File List (added / modified).
  - [ ] Set this story's `Status:` header → `review`.
  - [ ] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
    - Flip `3-2-avian-physics-foundation-arena-state-skeleton: ready-for-dev` → `3-2-avian-physics-foundation-arena-state-skeleton: review` (the dev-story flips through `ready-for-dev → in-progress → review`; final state at handoff is `review`; the post-code-review bookkeeping flip handles `review → done`).
    - epic-3 status stays `in-progress` — this is the second story in the epic; no transition.
    - Bump `last_updated:` (both top-comment line and YAML body key) to: `last_updated: YYYY-MM-DD (Story 3.2 ready-for-dev → review — Avian physics foundation + ArenaPlugin skeleton)`.
    - YAML parse verification: `python3 -c "import yaml; yaml.safe_load(open('_bmad-output/implementation-artifacts/sprint-status.yaml')); print('OK')"` → expected `OK`. Falls back to Ruby (`ruby -ryaml -e "YAML.load_file(...)"`) if PyYAML unavailable, per Story 2.6 precedent.
  - [ ] **Commit 1 (source — triggers CI):** stage `src/main.rs` and `src/arena/mod.rs`. **NO** `_bmad-output/**` files in this commit.
    - HEREDOC commit message subject: `feat: Avian physics foundation + ArenaPlugin skeleton (Story 3.2)`. Single-line, target ≤ 70 chars (literal length: 62 — within limit).
    - Push to `origin/master`. Triggers full 4-job `ci.yml` matrix because `src/**` falls through `paths-ignore`.
    - **Expected CI outcome:** all 4 jobs ✓. Wall time projection: **15–40 m on cold avian3d cache** (the matrix builds avian3d + parry3d + nalgebra fresh on each runner; subsequent commits hit warm caches at ~5–10 m). The msrv-check job (Rust 1.89) MUST also pass — avian3d 0.6 supports Rust 1.89+; if the msrv leg fails on a transitive dep MSRV mismatch, that's a documented gotcha in the deferred-work tracker (Story 1.4 MSRV section, lines 16, 25, 26).
    - `gh run list --workflow=ci.yml -L 1` → capture run ID. Wait for completion. `gh run view <ID> --log | grep -cE 'warning:|error:'` → expected `0` (modulo the `Free disk space` action ambient-noise filter from 3.1's precedent).
  - [ ] **Commit 2 (bookkeeping — does NOT trigger CI):** stage `_bmad-output/implementation-artifacts/sprint-status.yaml` and `_bmad-output/implementation-artifacts/3-2-avian-physics-foundation-arena-state-skeleton.md`.
    - HEREDOC commit message subject: `bmad: story 3.2 ready-for-dev → review (Avian + ArenaPlugin skeleton)`. Matches Story 3.1 / 2.6 / 2.5 bookkeeping commit shape.
    - Push to `origin/master`. **Does NOT trigger CI** — `_bmad-output/**` is in `ci.yml`'s `paths-ignore`.
  - [ ] **Why two commits, not one:** matches Stories 3.1 / 2.6 / 2.5 / 2.4 precedent — clean diff focus (Commit 1 is reviewable code; Commit 2 is YAML/docs); CI cost focus (Commit 1 triggers CI, Commit 2 doesn't); roll-back granularity (a code-review patch can amend Commit 1 without disturbing the bookkeeping).
  - [ ] **Push-fold optimization:** if the dev opts to fold both commits into a single `git push` event, that's acceptable — one CI run captures everything; document the fold in Dev Agent Record. Do NOT collapse the two commits into one.
  - [ ] Story awaits code review. **Code review recommended via `bmad-code-review` skill, ideally with a different LLM than the implementer.** The diff surface is small (~6 lines added in `main.rs`; ~30 lines new in `src/arena/mod.rs`); a 3-agent review is appropriate but may produce few findings given the narrow scope. Specific review attention areas: (a) the FixedPostUpdate-vs-FixedUpdate semantic gap — the architecture says "FixedUpdate" colloquially, the implementation uses `Time::<Fixed>::from_hz(60.0)` which is the correct Avian 0.6 idiom; (b) the generic-cleanup function placement decision (`src/arena/mod.rs` vs. `src/core/cleanup.rs`); (c) the `ArenaSystems::Setup`-with-no-consumers `dead_code` risk; (d) confirmation that no debug PhysicsDebugPlugin sneaked in.

## Dev Notes

### Why this story exists

Story 3.2 lays the **foundation** for every later Epic-3 story. Three concrete things land:

1. **Avian physics in the schedule.** Stories 3.5 (PlayerShip RigidBody::Dynamic), 3.6 (`ExternalForce` thrust), 3.7 (`ExternalTorque` rotation), 3.8 (dampener `ExternalForce`/`ExternalTorque`), 3.9 (Projectile RigidBody::Dynamic), 3.10 (collision events between Projectile + Asteroid), and Story 3.3 (Asteroid RigidBody::Static) ALL assume the physics schedule is registered, ticking at 60 Hz, in zero-g. Without 3.2, every subsequent story has to either (a) add physics plumbing as a side-effect (scope drift), or (b) be blocked. Landing this once, deliberately, in 3.2 is the architecture.md:240 commitment paying off.

2. **Arena state lifecycle skeleton.** Stories 3.3 (asteroid field + DirectionalLight), 3.5 (PlayerShip + cockpit Camera3d), 3.9 (Projectile spawning), 3.11 (HUD root) all spawn entities tagged `ArenaEntity` and rely on `OnExit(Arena)` cleanup to despawn them. 3.2 establishes the `ArenaEntity` marker type AND the `cleanup_on_exit::<ArenaEntity>` system so 3.3+ can simply "tag and walk away." Without 3.2, the cleanup pattern is re-derived ad hoc per spawning story — an architecture violation (architecture.md:420) waiting to happen.

3. **`src/arena/` as a first-class subtree.** Architecture.md:582-588 reserves `src/run/` for run/director/caravan/waypoint logic; `src/arena/` is NOT in the architecture's directory tree, but the epic-3 ACs explicitly name `src/arena/mod.rs` as the file path. **Reconciliation:** `src/arena/` is the new home for arena-state lifecycle (a strict subset of "run logic" — Arena is one run-state amongst Caravan, PostRun, etc.). Story 3.2 establishes the precedent. When Caravan-state arrives in Epic 6, it lands at `src/run/caravan.rs` per architecture (the broader run-director machinery); arena-specific code stays at `src/arena/`. Architecturally clean: `src/arena/` is a leaf for Arena lifecycle; `src/run/` is the cross-state director when it arrives.

[Source: [`epics/epic-3-arena-flight-first-combat-first-playable.md:29-55`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md) (Story 3.2 epic spec); [`architecture.md:240`](../planning-artifacts/architecture.md) (FixedUpdate physics decision); [`architecture.md:420`](../planning-artifacts/architecture.md) (`cleanup_on_exit::<T>` pattern); [`architecture.md:343-350`](../planning-artifacts/architecture.md) (Plugin-per-feature module pattern); [`3-1-...-md` lines 722-736](./3-1-title-screen-stub-mainmenu-arena-transition.md) (Story 3.2 forward-compat hand-off from 3.1)]

### Inherited context from Stories 1.1, 2.1, 3.1

| Fact | Value | Source |
|---|---|---|
| Bevy version | `0.18` (resolved `0.18.1`) | `Cargo.toml:8` |
| Avian version | `avian3d = "0.6"` — already pinned, never used until 3.2 | `Cargo.toml:9` |
| Avian build status | dep is in `Cargo.lock`; `cargo build` resolves it but does NOT compile the crate (no `use avian3d` in source pre-3.2) | empirical |
| Bevy `Time::<Fixed>` default rate | 64 Hz (Bevy 0.16+) | Bevy 0.18 docs |
| `GameState::Arena` reachability | live since Story 3.1 (Enter on title screen → `NextState::set(Arena)`) | `src/ui/main_menu.rs` |
| `log_arena_entered` registration | wired in main.rs:41 since Story 3.1 — `info!("entered Arena")` fires on `OnEnter(Arena)` | `src/main.rs:41`, `src/state.rs:30` |
| `cleanup_main_menu` pattern | root-only marker, `commands.entity(e).despawn()` loop, children cascade via Bevy 0.18 `ChildOf` linked-despawn — Story 3.1 canonicalized this | `src/ui/main_menu.rs:37-44` |
| `MainMenuEntity` precedent | unit struct, `pub`, no fields — exact pattern `ArenaEntity` should mirror | `src/ui/main_menu.rs:14-15` |
| `VisualSystems::Setup` | configured on `OnEnter(Loading)`, currently EMPTY (deferred-work.md:139 — re-target to `OnEnter(Arena)` is Story 3.3's job, NOT 3.2's) | `src/visual/mod.rs:23-26`, `deferred-work.md:139` |
| `TuningSystems::Reload` | configured on `Update`; `ArenaSystems::Setup` is the parallel `OnEnter(Arena)` analogue | `src/tuning/mod.rs:29` |
| Test count post-3.1 | **14 passing** | `_bmad-output/implementation-artifacts/3-1-...-md` Dev Agent Record |
| Test count post-3.2 (expected) | **14** (no new tests, no removed tests) | this story |
| `tracing` + panic-hook | live since 1.8 | `src/logging.rs` |
| `TuningPlugin` hot-reload | active in debug; 3.2 must not disturb it | `src/tuning/mod.rs` |
| Splash race re-deferred | `src/splash.rs` cleanup-iteration WARN remains; Story 3.2 does NOT touch splash; race is non-deterministic per 3.1 dev-log observation | `deferred-work.md:137` |
| Splash file location debt | `src/splash.rs` flat at `src/`, not `src/ui/splash.rs`; 3.2 does NOT move it | `deferred-work.md:138` |
| Commit style precedent | `feat:` for source, `bmad:` for bookkeeping; no `Co-Authored-By` trailer; HEREDOC for multi-line | `git log --oneline -n 15` |
| Two-commit pattern | source + bookkeeping (separate `git push` allowed); used by Stories 1.7/2.4/2.5/2.6/3.1 | `git log` |
| `paths-ignore` in CI | `.github/workflows/ci.yml` excludes `_bmad/**` and `_bmad-output/**` from `push`/`pull_request` triggers | `deferred-work.md:5` |
| Architecture path discrepancy | `src/arena/` is NOT in architecture.md tree; epic-3 spec mandates it; Dev Notes section above reconciles | architecture.md:582-588, epic-3 |

### Five-key constraint summary (memorize these)

1. **`Time::<Fixed>::from_hz(60.0)` is THE 60-Hz mechanism in Avian 0.6.** Avian 0.6 runs in `FixedPostUpdate` by default and inherits Bevy's `Time<Fixed>` rate. Setting `Time::<Fixed>::from_hz(60.0)` gives the architecture-mandated 60 Hz. **DO NOT** try to configure Avian's schedule explicitly via `app.edit_schedule(...)` or by replacing `PhysicsPlugins::default()` with a custom assembly — `default()` is the canonical entry point per Avian 0.6 docs.
2. **`Gravity(Vec3::ZERO)` in main.rs, NOT in ArenaPlugin.** Physics is a global concern; `ArenaPlugin` owns Arena lifecycle. Inserting `Gravity` inside `ArenaPlugin::build` would scope it to Arena (wrong for Caravan reuse in Epic 6) AND only insert it after `add_plugins(ArenaPlugin)` runs (race-prone if any system reads Gravity earlier).
3. **Generic `cleanup_on_exit<T>` is intentional.** Stories 3.4, 3.11, and any future state-marker user need the same despawn pattern. Defining once in `src/arena/mod.rs` (or moving to `src/core/cleanup.rs` later if a third+ consumer warrants it) avoids 4–6 copy-pasted variants by Epic 6. Don't make 3.2-specific copies.
4. **Don't add `log_arena_entered` again.** Story 3.1 already wired `info!("entered Arena")` via `state.rs::log_arena_entered` registered on `OnEnter(GameState::Arena)` in `main.rs`. The epic AC #3 "info! log emitted" is satisfied by 3.1's existing wiring. Adding a duplicate inside `ArenaPlugin::build` produces double log lines.
5. **Test count stays at 14.** Story 3.2 adds zero tests. The plugin skeleton has nothing meaningfully testable in isolation; Story 3.3+ asteroid spawning is the natural integration-test target. If `cargo test` reports anything other than 14, investigate.

### Architecture compliance

- **Avian XPBD in fixed-step at 60 Hz** matches `architecture.md:240` ("Avian physics runs in `FixedUpdate` at 60 Hz (fixed-step). Rendering runs at display refresh via `Update`."). The implementation uses Avian 0.6's idiomatic `PhysicsPlugins::default()` (which schedules into `FixedPostUpdate`, a sub-schedule of Bevy's `FixedMain`) plus `Time::<Fixed>::from_hz(60.0)`. The architecture's "FixedUpdate" wording predates Avian 0.6's schedule restructure; the canonical 60 Hz semantic is preserved. [Source: architecture.md:240]
- **`ArenaPlugin` per-feature plugin pattern** matches `architecture.md:343-350` ("each feature module exposes a `<Feature>Plugin` type ... a `SystemSet` enum (`<Feature>Systems`) for ordering ... declares components / resources / events locally"). [Source: architecture.md:343-350]
- **`ArenaSystems::Setup` SystemSet** matches `architecture.md:347` (`<Feature>Systems` enum). The minimal-variant approach (one variant, no consumers in 3.2 itself) matches `VisualSystems::Setup`'s precedent (one variant, currently empty). [Source: architecture.md:347, src/visual/mod.rs:13-16]
- **`cleanup_on_exit::<T>` generic pattern** matches `architecture.md:420` ("entities spawned for a state tag themselves with a marker component (e.g., `ArenaEntity`) and are despawned by a `cleanup_on_exit::<ArenaEntity>` system in `OnExit(GameState::Arena)`"). The generic implementation (`fn cleanup_on_exit<T: Component>(...)`) is a refinement that matches the architectural intent literally. [Source: architecture.md:420]
- **Marker-only-on-roots cleanup** matches the precedent canonicalized by Story 3.1's `cleanup_main_menu` — leverages Bevy 0.18 `ChildOf` linked-despawn. ArenaEntity follows the same pattern. [Source: src/ui/main_menu.rs:36-44, deferred-work.md:91 "the canonical state-exit cleanup pattern for all future UI surfaces"]
- **Cross-cutting Resources in `main.rs`** — `Gravity(Vec3::ZERO)` and `Time::<Fixed>::from_hz(60.0)` registered in main.rs match `architecture.md:660-664` (`PlayerInputAxes`, `SalvageCurrency`, `TuningConfig`, `State<GameState>`, `SaveData` are listed as Cross-Cutting Resources registered in main.rs). Gravity is similarly cross-cutting (any physics-affected entity reads it). [Source: architecture.md:660-664]
- **No `.after(specific_system_function)`** — ArenaPlugin uses `configure_sets` for ordering (when 3.3+ adds systems to `ArenaSystems::Setup`), not function-name-based `.after()`. [Source: architecture.md:415]
- **No god-plugin** — `ArenaPlugin` owns ONLY arena-state lifecycle; physics setup is in main.rs (engine-wide); UI is in `UiPlugin`; tuning is in `TuningPlugin`. Plugin boundaries match `architecture.md:643-657`. [Source: architecture.md:643-657]

### Library / framework requirements

| Crate | Version | Change in Story 3.2 |
|---|---|---|
| `bevy` | `0.18` (resolved `0.18.1`) | unchanged — uses existing `bevy::prelude::*` and `bevy::time::Time<Fixed>` (no new feature additions) |
| `avian3d` | `0.6` | first-time USAGE in source — already pinned in `Cargo.toml:9` since Story 1.1 (plugin-compatibility verification gate) |
| All other pinned deps | unchanged | unchanged |
| `Cargo.toml` | unchanged | no feature additions, no version bumps, no new deps |
| `Cargo.lock` | unchanged (expected) | no dep tree change; should be byte-identical post-3.2; if it changes, see Task 4 conditional |

**Avian 0.6 imports needed in 3.2:**
- `avian3d::prelude::PhysicsPlugins` — the plugin bundle (covers `IntegratorPlugin`, `BroadPhasePlugin`, `NarrowPhasePlugin`, `ContactReportingPlugin`, etc.).
- `avian3d::prelude::Gravity` — the `Resource` newtype around `Vec3`. Re-exported from `avian3d::dynamics::integrator::Gravity`. Default value is `Gravity(Vec3::Y * -9.81)`; we override to `Gravity(Vec3::ZERO)`.

**No imports needed yet (deferred to Stories 3.3+):**
- `RigidBody`, `Collider` — Story 3.3 (Static asteroids), Story 3.5 (Dynamic player ship), Story 3.9 (Dynamic projectiles).
- `ExternalForce`, `ExternalTorque` — Stories 3.6, 3.7, 3.8.
- `LinearVelocity`, `AngularVelocity` — Stories 3.7, 3.8, 3.10.
- `CollisionLayers`, `CollisionStarted` events — Story 3.10.

### File structure changes

| Path | Action | Purpose |
|---|---|---|
| `src/arena/mod.rs` | **Add** | `ArenaPlugin` orchestrator + `ArenaEntity` marker + `ArenaSystems` enum + generic `cleanup_on_exit<T>`; ≤ 60 lines. |
| `src/main.rs` | **Modify** | +`mod arena;`, +`use arena::ArenaPlugin;`, +`use avian3d::prelude::{Gravity, PhysicsPlugins};`, +`add_plugins(PhysicsPlugins::default())`, +`insert_resource(Time::<Fixed>::from_hz(60.0))`, +`insert_resource(Gravity(Vec3::ZERO))`, +`add_plugins(ArenaPlugin)`. Net +6 lines. |
| `Cargo.toml`, `Cargo.lock` | **Do NOT touch** | avian3d already pinned at 0.6 since Story 1.1; no feature additions. |
| `src/state.rs` | **Do NOT touch** | `log_arena_entered` already exists (Story 3.1, line 30); the `#[expect(dead_code)]` attribute still satisfies (4 unused variants remain post-3.2: `Caravan`, `PostRun`, `PhotoMode`, `Paused`). |
| `src/splash.rs` | **Do NOT touch** | Race remains deferred (deferred-work.md:137). |
| `src/ui/**` | **Do NOT touch** | Title screen stays as-is. |
| `src/visual/**` | **Do NOT touch** | The `VisualSystems::Setup` `OnEnter(Loading)`-vs-`OnEnter(Arena)` mismatch (deferred-work.md:139) is **Story 3.3's job**, NOT 3.2's. 3.2 leaves it alone. |
| `src/tuning/**`, `src/logging.rs` | **Do NOT touch** | Out of scope. |
| `assets/**` | **Do NOT touch** | No asset changes. |
| `docs/**` | **Do NOT touch** | Out of scope (docs/tech-spike/m1-decision.md is audit-trail evidence). |
| `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.gitignore` | **Do NOT touch** | Out of scope. |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | **Modify** (Task 5) | 3-2 → review, last_updated bump. |
| `_bmad-output/implementation-artifacts/3-2-...-md` (this file) | **Modify** | Tasks checked, Dev Agent Record populated, Status → review. |
| `_bmad-output/implementation-artifacts/deferred-work.md` | **Do NOT modify in dev-story** | Code-review may add entries post-3.2; that's a review-time concern, not a dev-time one. |

### `src/arena/mod.rs` skeleton (near-verbatim — rustfmt-tolerant)

```rust
//! ArenaPlugin — owns GameState::Arena entity lifecycle (spawn / cleanup).
//! Story 3.3 attaches asteroid spawning; later stories add player ship, projectiles, HUD.

use bevy::prelude::*;

pub struct ArenaPlugin;

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ArenaSystems {
    Setup,
}

#[derive(Component)]
pub struct ArenaEntity;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            OnEnter(crate::state::GameState::Arena),
            ArenaSystems::Setup,
        );
        app.add_systems(
            OnExit(crate::state::GameState::Arena),
            cleanup_on_exit::<ArenaEntity>,
        );
    }
}

pub fn cleanup_on_exit<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

### `src/main.rs` post-edit skeleton (rustfmt-tolerant — diff against current)

```rust
//! asteroids3D — app entry point.
//! Initializes tracing subscriber + panic-hook-to-file before Bevy startup.
//! Registers DefaultPlugins (minus LogPlugin), GameState, splash flow, and gameplay plugins.

use avian3d::prelude::{Gravity, PhysicsPlugins};
use bevy::prelude::*;

mod arena;
mod logging;
mod splash;
mod state;
mod tuning;
mod ui;
mod visual;

use arena::ArenaPlugin;
use logging::init_logging;
use splash::{SplashConfig, cleanup_loading_entities, spawn_splash, tick_splash_timer};
use state::{GameState, log_arena_entered, log_loading_entered, log_mainmenu_entered};
use tuning::TuningPlugin;
use ui::UiPlugin;
use visual::VisualPlugin;

fn main() -> AppExit {
    let log_path = init_logging();
    if let Some(path) = &log_path {
        info!("file logging active at {}", path.display());
    }

    let default_plugins = DefaultPlugins.build().disable::<bevy::log::LogPlugin>();

    App::new()
        .add_plugins(default_plugins)
        .init_state::<GameState>()
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Gravity(Vec3::ZERO))
        .add_plugins(TuningPlugin)
        .add_plugins(VisualPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ArenaPlugin)
        .init_resource::<SplashConfig>()
        .add_systems(
            OnEnter(GameState::Loading),
            (log_loading_entered, spawn_splash),
        )
        .add_systems(OnEnter(GameState::MainMenu), log_mainmenu_entered)
        .add_systems(OnEnter(GameState::Arena), log_arena_entered)
        .add_systems(
            Update,
            tick_splash_timer.run_if(in_state(GameState::Loading)),
        )
        .add_systems(OnExit(GameState::Loading), cleanup_loading_entities)
        .run()
}
```

### Generic vs. arena-specific cleanup — placement decision

**Decision:** ship `pub fn cleanup_on_exit<T: Component>(...)` inside `src/arena/mod.rs` for 3.2.

**Alternatives considered:**

| Placement | Pro | Con |
|---|---|---|
| `src/arena/mod.rs` (selected) | Local to the first consumer; no premature `core/` directory creation. | Semantically generic but lives in a feature module; future consumer (Story 3.4 `PauseOverlayEntity`) imports from `crate::arena::cleanup_on_exit` which reads slightly weird. |
| `src/core/cleanup.rs` (rejected for 3.2; reconsider in Epic 4+) | Architecturally pure home (architecture.md:550 reserves `src/core/` for shared types). | Premature creation of `src/core/` for one function. YAGNI. |
| Inlined per consumer | Zero indirection. | Copy-paste growth: 4–6 sites by Epic 6. Refactor pain. |

**Migration trigger:** when the **third** consumer arrives (3.4 PauseOverlayEntity is #2; 3.11 HudEntity is #3 → migrate at 3.11), or earlier if a code reviewer flags it. The migration is mechanical — `git mv` plus 2–3 `use` updates. Not 3.2's problem.

### Test count discipline

Pre-3.2 (post-3.1): **14 passing tests**. Story 3.2 modifies the count as follows:
- **Removed:** none.
- **Added:** none.
- **Net post-3.2: 14 passing tests.**

If `cargo test` reports anything other than `14 passed`:
- **<14:** another test was accidentally deleted from `src/`. Investigate `git diff --stat src/`; revert.
- **>14:** an unscoped test was added. Investigate; this story spec authorizes zero new tests.

**Why no tests in 3.2:**
1. `ArenaEntity` is a unit struct — no behavior to test.
2. `ArenaSystems` is a derive-generated enum — `Hash + Eq + Copy + Debug` are all rustc-derived; testing them tests rustc.
3. `cleanup_on_exit<T>` is a 4-line query loop wrapping `commands.entity(e).despawn()` — a Bevy-provided primitive. Testing it requires App + state-transition harness (architecture.md:354 defers integration tests post-M3).
4. `PhysicsPlugins::default()` registration + `Gravity(Vec3::ZERO)` insertion — these are configuration calls; testing them requires asserting the resource is present after `App::new()` runs, which is closer to Avian's own test surface.

The natural test landing is Story 3.3+ when actual asteroid-spawn behavior exists and end-to-end OnEnter/OnExit lifecycle has observable outcomes (entity counts, position assertions).

### Latest technical information

- **Avian 0.6 `PhysicsPlugins::default()`** — schedules physics into `FixedPostUpdate` (a sub-schedule of Bevy 0.18's `FixedMain`). The `Time<Physics>` clock automatically follows `Time<Fixed>` (per Avian 0.2+ change documented in upstream release notes). Configuring `Time::<Fixed>::from_hz(60.0)` is the canonical 60-Hz mechanism. [Source: docs.rs/avian3d/0.6 + docs.rs/avian3d/latest schedule docs]
- **Bevy 0.18 `Time::<Fixed>::from_hz(rate)`** — direct constructor. Imported via `bevy::prelude::*` (which re-exports `Time<Fixed>`) plus `bevy::time::Fixed` if needed explicitly. The default `Time<Fixed>` rate in Bevy 0.16+ is 64 Hz; we override to 60 Hz. [Source: Bevy 0.18 docs]
- **Avian 0.6 `Gravity` resource** — `pub struct Gravity(pub Vec3);` re-exported via `avian3d::prelude::Gravity` (canonical path: `avian3d::dynamics::integrator::Gravity`). Default value is `Gravity(Vec3::new(0.0, -9.81, 0.0))`. Our override `Gravity(Vec3::ZERO)` matches the project's zero-g space environment. [Source: docs.rs/avian3d/0.6]
- **Avian 0.6 schedule semantics** — physics runs `FixedFirst → FixedPreUpdate → FixedUpdate → FixedPostUpdate` per Bevy fixed-schedule order. `PhysicsPlugins` injects systems into `FixedPostUpdate` so user-authored `FixedUpdate` systems (e.g., flight thrust application in Story 3.6) run BEFORE physics integration each tick — the correct order for `ExternalForce`-driven motion. **No 3.2 user code touches FixedUpdate yet**, but this is load-bearing context for 3.6+.
- **No `PhysicsDebugPlugin` in 3.2** — Avian 0.6 ships an opt-in `PhysicsDebugPlugin` for visualizing colliders / contacts / AABBs. **DO NOT add it in 3.2**. It's a debug-time dev tool that fits Epic 5+ playtesting; the M2 milestone gate doesn't require it. If a future story adds it, gate behind `cfg(debug_assertions)` per architecture.md:613-615 (the `src/debug/` module pattern).

### Previous-story intelligence — what to learn from 1.1 / 2.1 / 3.1

**From Story 1.1 (Cargo.toml hand-authoring):**
- `avian3d = "0.6"` was added to Cargo.toml at line 9 from the start as part of the plugin-compatibility-verification gate. The crate has been resolved by Cargo since 1.1 but never compiled (no `use avian3d` in source). Story 3.2 is the first compile-and-link of avian3d's actual code. **First-build cost:** several minutes (parry3d, nalgebra, simba, et al. compile fresh). Plan the verification sweep (Task 3) accordingly.

**From Story 2.1 (VisualPlugin skeleton + first SystemSet introduction):**
- The `VisualSystems::Setup` enum was the first `<Feature>Systems` declaration in the project. `ArenaSystems::Setup` follows the SAME idiom (single-variant enum, used in `configure_sets`, no consumers at introduction time). 2.1's pattern survived 2.2/2.3/2.4/2.5 without `dead_code` warnings, so 3.2's identical pattern should also clear clippy.
- The `#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]` derive set is identical between `VisualSystems` and `TuningSystems`. Use the same set for `ArenaSystems`. (`Copy` requires the enum to be a unit-variant; `Hash + Eq` are required by `SystemSet`.)

**From Story 3.1 (UiPlugin + cleanup_main_menu canonicalization):**
- The `cleanup_main_menu` system at `src/ui/main_menu.rs:36-44` IS the canonical pattern for `ArenaEntity` cleanup. **3.2's `cleanup_on_exit::<ArenaEntity>` is the generic equivalent.** The pattern matches verbatim — `for e in &query { commands.entity(e).despawn(); }` — except T is generic.
- The two-commit pattern (source + bookkeeping) is now established for 4 consecutive stories (2.4, 2.5, 2.6, 3.1). 3.2 follows.
- Till's commit-message style (no `Co-Authored-By` trailer; `feat:` for source; `bmad:` for bookkeeping; HEREDOC for multi-line) is consistent and binding.
- The `paths-ignore` convention means a Commit 2 push of pure `_bmad-output/**` files triggers ZERO CI cost.
- **Splash race re-deferred** — `src/splash.rs:67-73`'s `Entity despawned... invalid` WARN is non-deterministic and present in some 3.1 dev-runs. **3.2 must not interpret this WARN as a 3.2 regression** if it appears in `/tmp/story-3-2-run.log`. (deferred-work.md:137)

### Forward compatibility — Story 3.3 (asteroid field + DirectionalLight) hand-off

Story 3.3 reads this story's outcome and assumes:
- `ArenaEntity` marker is constructable as `(Component) ArenaEntity` (true post-3.2).
- `ArenaSystems::Setup` is a public `SystemSet` configurable on `OnEnter(GameState::Arena)` (true post-3.2 — already configured by `ArenaPlugin`).
- `cleanup_on_exit::<ArenaEntity>` runs on `OnExit(GameState::Arena)` (true post-3.2).
- Avian's `RigidBody::Static` and `Collider::sphere(...)` types are importable via `avian3d::prelude::*` (true — first 3.2 puts the dep in the build graph).
- `Gravity(Vec3::ZERO)` is in effect — static asteroids don't need gravity, but if 3.3 spawns any `RigidBody::Dynamic` debris (it shouldn't per epic spec), zero-g applies. (true post-3.2.)

Story 3.3 will:
- Author `src/arena/zone.rs` with `pub fn spawn_arena_zone(...)` system, registered in `ArenaPlugin::build` (3.3 adds to ArenaPlugin, 3.2 just establishes the plugin shell).
- `.in_set(crate::arena::ArenaSystems::Setup)` — uses 3.2's set declaration.
- Spawn 15–25 asteroids tagged `ArenaEntity` plus `ToonMaterial` plus `RigidBody::Static` + `Collider::sphere(radius)`.
- Spawn 1 `DirectionalLight` tagged `ArenaEntity`.
- **Reconfigure `VisualSystems::Setup` from `OnEnter(Loading)` → `OnEnter(Arena)`** (deferred-work.md:139 — Story 3.3's job, NOT 3.2's).

Story 3.3's author should NOT need to modify `src/arena/mod.rs`'s structure; they add a sibling file (`src/arena/zone.rs`) and `pub mod zone;` it. The `ArenaPlugin::build` function gets one or two new lines (`add_systems` for `spawn_arena_zone`).

### Forward compatibility — Story 3.4 (pause overlay) hand-off

Story 3.4 needs `cleanup_on_exit::<PauseOverlayEntity>` on `OnExit(GameState::Paused)`. **Generic `cleanup_on_exit<T>` from 3.2 is the canonical entry point** — Story 3.4's author imports `crate::arena::cleanup_on_exit::<PauseOverlayEntity>` and registers it. No new generic helper needed.

If the placement at `src/arena/mod.rs` becomes ergonomically awkward by Story 3.11 (third consumer), migrate to `src/core/cleanup.rs` then. 3.4 is fine importing from `arena::`.

### Forward compatibility — Stories 3.5+ (PlayerShip RigidBody, flight forces)

Stories 3.5–3.10 will ALL spawn entities with `ArenaEntity` marker (PlayerShip + cockpit Camera3d, projectiles, HUD root). `cleanup_on_exit::<ArenaEntity>` (registered in 3.2) catches all of them on state exit. **No 3.2 work is needed for those stories;** 3.2's contribution is the marker + cleanup contract.

The physics-foundation half of 3.2 (PhysicsPlugins + Time<Fixed> + Gravity) is similarly load-bearing for 3.5+ — Story 3.5's PlayerShip RigidBody + Collider, 3.6's ExternalForce, 3.7's ExternalTorque all attach to the physics world this story registers.

### Project structure notes

- **Path alignment:**
  - `src/arena/mod.rs` is **NEW**. Architecture.md:582-588 lists `src/run/` (for caravan/director/waypoint), but does NOT list `src/arena/` — yet the epic-3 ACs explicitly mandate `src/arena/mod.rs`. **Reconciliation:** `src/arena/` is a legitimate leaf for arena-specific lifecycle code; `src/run/` arrives in Epic 6 for cross-state director machinery. Both can coexist (architecture remains clean: `src/arena/` owns Arena, `src/run/` owns Run-as-cross-state-orchestrator). This story establishes the precedent.
  - `src/main.rs` modifications are in-place additions; net +6 lines.
- **No path conflicts** introduced by Story 3.2.
- **Open architecture clarification (post-3.2 candidate):** the architecture.md tree (lines 582-588) could be updated in a future planning-sweep story to add `src/arena/` as a sibling of `src/run/`. NOT 3.2's job — `_bmad-output/planning-artifacts/**` is read-only from a story-execution perspective (deferred-work.md precedent: planning artifacts are touched only by dedicated planning-sweep stories).
- **Splash file location debt unresolved:** `src/splash.rs` stays flat at `src/`, not `src/ui/splash.rs`. Re-deferred from Story 3.1 (deferred-work.md:138). 3.2 does NOT touch splash.
- **`Cargo.toml`, `Cargo.lock`, `.gitignore`, `.github/workflows/**`, `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `assets/**`, `docs/**` — UNTOUCHED.**

### LLM dev-agent guardrails — most-likely-to-go-wrong patterns

These are the failure modes most likely to bite a fast-moving dev:

1. **Forgetting `Time::<Fixed>::from_hz(60.0)`.** Avian 0.6 inherits Bevy's default 64 Hz `Time<Fixed>` if not overridden. Architecture mandates 60 Hz (architecture.md:240). The default 64 Hz is functionally fine for the prototype, but the AC text expects 60 Hz; verifying via a `Time<Fixed>` resource read in a debug log line during Task 3 is overkill — just trust the `from_hz(60.0)` call is correct.
2. **Importing `PhysicsPlugins` as a trait or struct from the wrong path.** The canonical path is `avian3d::prelude::PhysicsPlugins` (re-exported from `avian3d::PhysicsPlugins`). Don't import `avian3d::dynamics::PhysicsPlugins` or similar.
3. **Inserting `Gravity` resource via `init_resource` instead of `insert_resource`.** `init_resource::<Gravity>()` would use Avian's default `Gravity(Vec3::Y * -9.81)`. We need `insert_resource(Gravity(Vec3::ZERO))` to override.
4. **Adding `PhysicsDebugPlugin`.** Avian's debug visualizer is opt-in. Don't add it. Its presence shows colliders/contacts as wireframes during gameplay — visually noisy and a M2-out-of-scope distraction.
5. **Putting physics resources in `ArenaPlugin::build`.** Physics is global. Plug-it-in-once-in-main is the architecture commitment (architecture.md:660-664). Putting it in ArenaPlugin scopes it incorrectly to Arena and creates registration-order races.
6. **Re-registering `log_arena_entered`.** Story 3.1 already wired it (`src/main.rs:41` + `src/state.rs:30`). ArenaPlugin doesn't add a duplicate log. The `info!("entered Arena")` from 3.1 is the satisfying signal for AC #3.
7. **Making `ArenaSystems` a non-`Copy` enum.** SystemSet derive requires `Hash + Eq + Send + Sync + 'static`; `Copy` is a quality-of-life requirement so `.in_set(ArenaSystems::Setup)` works without `clone()`. Match `VisualSystems` / `TuningSystems` exactly: `#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]`.
8. **Making `cleanup_on_exit<T>` non-generic by hard-coding `ArenaEntity`.** The function is generic ON PURPOSE — Stories 3.4, 3.11 reuse it. Hard-coding ArenaEntity forces 3.4 to copy-paste a `cleanup_pause_overlay` variant. Don't.
9. **Touching `src/splash.rs`.** Race deferred to whichever story next legitimately touches splash (4.7 most likely). 3.2 is not that story.
10. **Touching `src/visual/mod.rs`'s `VisualSystems::Setup` → `OnEnter(Arena)` reconfigure.** That's Story 3.3's job (deferred-work.md:139). 3.2 leaves the mismatched-but-empty configuration alone.
11. **Touching `Cargo.toml`.** No version bumps, no feature additions. avian3d 0.6 is already pinned. `Cargo.lock` should not change either; if it does, document the reason in Completion Notes.
12. **Touching `_bmad-output/planning-artifacts/**`.** Read-only from story-execution perspective.
13. **Adding tests.** None for 3.2. The test count discipline expects 14. Don't add a "trivial" test like `arena_entity_is_unit` — it tests rustc, not the architecture.
14. **Skipping the runtime smoke (Task 3 `cargo run`).** Without it, you don't catch (a) Avian-emitted runtime warnings, (b) physics-plugin registration ordering bugs, (c) the title-screen → Arena flow regressing. The smoke is fast (10–15 seconds of human time after the build).

### Why bundle physics + ArenaPlugin in one story?

This is a deliberate scope choice. Three alternatives were considered:

**Alternative A (rejected): Physics-only — defer ArenaPlugin to a separate story.**
- Pro: very narrow diff (~3 lines in main.rs), trivial review.
- Con: Story 3.3 (asteroid field) needs BOTH physics (RigidBody::Static + Collider) AND ArenaEntity tagging. Splitting forces 3.3 to either land both or wait twice. Net throughput loss.
- Con: physics + arena lifecycle are conceptually coupled — Avian setup is the precondition for Arena spawning anything physics-affected.

**Alternative B (rejected): ArenaPlugin-only — defer physics to Story 3.3 or 3.5.**
- Pro: pure ECS skeleton landing.
- Con: Story 3.3 inherits "add Avian + spawn 25 asteroids" — that's where alternative A fails too. Same throughput-loss penalty.
- Con: Architecture explicitly assigns physics-foundation to "subsequent flight and combat stories" preconditions; the epic-3 spec collocates them in 3.2 deliberately.

**Alternative C (selected): Physics + ArenaPlugin in one story.**
- Pro: matches the epic-3 spec verbatim.
- Pro: produces a coherent post-commit state — Avian is in the build graph + plugin is registered + lifecycle skeleton is in place. Story 3.3 can land asteroid-spawning incrementally without preamble.
- Pro: small diff (~6 lines main.rs + ~30 lines arena/mod.rs); easy review even with mixed concerns.
- Con: mixes physics-engine setup with arena-state lifecycle. Mitigated by tightly-scoped subtasks (Task 1 is purely arena; Task 2 splits physics + ArenaPlugin wiring cleanly).

### References

- [Source: [`_bmad-output/planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md:29-55`](../planning-artifacts/epics/epic-3-arena-flight-first-combat-first-playable.md)] — Story 3.2 epic spec.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:240`](../planning-artifacts/architecture.md)] — Avian FixedUpdate at 60 Hz decision.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:415-420`](../planning-artifacts/architecture.md)] — `cleanup_on_exit::<T>` pattern.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:343-350`](../planning-artifacts/architecture.md)] — Plugin-per-feature module pattern.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:660-664`](../planning-artifacts/architecture.md)] — Cross-cutting Resources registered in main.rs.
- [Source: [`_bmad-output/planning-artifacts/architecture.md:582-588`](../planning-artifacts/architecture.md)] — Source-tree layout (note: `src/arena/` not listed; reconciliation in Dev Notes).
- [Source: [`_bmad-output/planning-artifacts/prd.md`](../planning-artifacts/prd.md)] — FR2 6-DOF translation, FR8 cockpit-only camera (Story 3.5+ consumers), FR12 projectile damage (3.10 consumer).
- [Source: [`Cargo.toml:8-9`](../../Cargo.toml)] — bevy 0.18 + avian3d 0.6 pinned versions.
- [Source: [`src/main.rs`](../../src/main.rs)] — current plugin-registration block (post-3.1).
- [Source: [`src/state.rs:30`](../../src/state.rs)] — `log_arena_entered` from Story 3.1.
- [Source: [`src/ui/main_menu.rs:36-44`](../../src/ui/main_menu.rs)] — `cleanup_main_menu` precedent.
- [Source: [`src/visual/mod.rs:13-26`](../../src/visual/mod.rs)] — `VisualSystems::Setup` pattern.
- [Source: [`src/tuning/mod.rs:12-15,29`](../../src/tuning/mod.rs)] — `TuningSystems::Reload` pattern.
- [Source: [`_bmad-output/implementation-artifacts/3-1-title-screen-stub-mainmenu-arena-transition.md:722-736`](./3-1-title-screen-stub-mainmenu-arena-transition.md)] — Story 3.1 forward-compat hand-off describing what 3.2 will do.
- [Source: [`_bmad-output/implementation-artifacts/deferred-work.md:137-139`](./deferred-work.md)] — Story 3.1 deferrals (splash race, splash location, VisualSystems::Setup mismatch) re-affirmed for 3.2.
- [Source: [docs.rs/avian3d/0.6](https://docs.rs/avian3d/0.6.0/avian3d/)] — Avian 0.6 PhysicsPlugins, Gravity, schedule semantics.
- [Source: [`MEMORY.md` → `feedback_full_build_output.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_full_build_output.md)] — verification discipline: exit-0 + tail is NOT proof; grep explicitly.
- [Source: [`MEMORY.md` → `feedback_staged_rollout.md`](../../.claude/projects/-Users-tillfechteler-Projekte-rust-asteroids3D/memory/feedback_staged_rollout.md)] — staged-rollout preference; informs scope-bundling rationale.

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

Local verification sweep (all logs in `/tmp/`):

| Command | Log file | `grep -cE 'warning:\|error:'` | Notes |
|---|---|---|---|
| `cargo check` | `/tmp/story-3-2-check.log` | | |
| `cargo build` | `/tmp/story-3-2-build.log` | | |
| `cargo test` | `/tmp/story-3-2-test.log` | | |
| `cargo clippy --all-targets -- -D warnings` | `/tmp/story-3-2-clippy.log` | | |
| `cargo fmt --all -- --check` | exit code | | |
| `cargo build --release` | `/tmp/story-3-2-release.log` | | |
| `cargo run` (runtime smoke) | `/tmp/story-3-2-run.log` | | |

### Completion Notes List

### File List
