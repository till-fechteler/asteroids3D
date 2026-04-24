# Story 1.3: Toolchain, Lint, and Format Configuration

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want reproducible toolchain + lint + format configs committed,
So that local dev and CI share the same rules and formatting drift is impossible.

## Acceptance Criteria

1. **Toolchain pinned via `rust-toolchain.toml`.** A `rust-toolchain.toml` at the project root pins the stable Rust channel (see **Dev Notes → Toolchain Pinning Decision** for the exact channel string). `rustup show` inside the project reports the pinned channel. The file also lists the `rustfmt` and `clippy` components so CI (Story 1.4) picks them up automatically. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:63-66; architecture.md:140]
2. **Format config exists and passes.** A `rustfmt.toml` at the project root encodes project style (see **Dev Notes → rustfmt.toml Skeleton**). `cargo fmt --check` exits 0 on every committed `.rs` file in `src/`. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:68-70; architecture.md:156]
3. **Lint config exists and passes strictly.** A `clippy.toml` at the project root holds project thresholds (see **Dev Notes → clippy.toml Skeleton**). `cargo clippy --all-targets -- -D warnings` exits 0 on the current codebase. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:68-71; architecture.md:156]
4. **`.gitignore` covers Rust + Bevy conventions.** The existing two-line `.gitignore` is extended so that `target/`, Bevy's processed-asset cache (`imported_assets/`), IDE folders (`.vscode/`, `.idea/`), and OS artifacts (`.DS_Store`, `Thumbs.db`) are all ignored. `git status` after the extension is clean except for the intentional new/modified config files. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:72-76]
5. **MSRV declared in `Cargo.toml`.** `[package].rust-version = "1.89"` is added to `Cargo.toml` (Bevy 0.18 / `bevy_egui` 0.39 MSRV). No other `Cargo.toml` edit. [Source: _bmad-output/implementation-artifacts/deferred-work.md:7; 1-2-…md Review Findings → Defer MSRV]

## Tasks / Subtasks

- [x] **Task 1 — Author `rust-toolchain.toml` (AC: #1)**
  - [x] Created `rust-toolchain.toml` at project root (8 lines) using the skeleton in **Dev Notes → Toolchain Pinning Decision**.
  - [x] Pinned `channel = "1.94.1"` — same toolchain version Story 1.2 empirically verified.
  - [x] Listed `components = ["rustfmt", "clippy"]`.
  - [x] Added `profile = "minimal"`.
  - [x] `rustup show` confirmed active toolchain: `1.94.1-aarch64-apple-darwin`, `active because: overridden by '/Users/tillfechteler/Projekte/rust/asteroids3D/rust-toolchain.toml'`. Log: `/tmp/story-1-3-rustup-show.log`.

- [x] **Task 2 — Declare MSRV in `Cargo.toml` (AC: #5)**
  - [x] Added `rust-version = "1.89"` immediately after `edition = "2024"` in `[package]`. Diff = +1 line, no other edits.
  - [x] `cfg(debug_assertions)` block untouched; `build-override` not added (deferred, see Task 7).
  - [x] `cargo check` exit 0, `Cargo.lock` diff = 0 lines (no churn). The single warning in the log is the known `cfg(debug_assertions)` finding, unchanged since Story 1.2.

- [x] **Task 3 — Author `rustfmt.toml` (AC: #2)**
  - [x] Created `rustfmt.toml` (7 lines) per the skeleton. Only `edition`, `max_width`, `newline_style`, `use_field_init_shorthand`, `use_try_shorthand` pinned.
  - [x] `cargo fmt` once: exit 0, `src/main.rs` unchanged (cargo default is already fmt-clean).
  - [x] `cargo fmt --check` exit 0, empty log (rustfmt only prints on disagreement). Log: `/tmp/story-1-3-fmt.log`.

- [x] **Task 4 — Author `clippy.toml` + make clippy pass strictly (AC: #3)**
  - [x] Created `clippy.toml` (7 lines): `cognitive-complexity-threshold = 30`, `too-many-arguments-threshold = 8`, `type-complexity-threshold = 500`. No deny/allow lists — those arrive with gameplay modules in Epic 2+.
  - [x] `cargo clippy --all-targets -- -D warnings` exit 0. Zero clippy lint findings on cargo-default `src/main.rs`. No `src/main.rs` patches needed.
  - [x] Log: `/tmp/story-1-3-clippy.log` (3 lines). The one `warning:` hit in the log is the Cargo-manifest `cfg(debug_assertions)` warning — a *manifest* warning, not a rustc/clippy warning, so `-D warnings` (a rustc flag) does not deny it.

- [x] **Task 5 — Extend `.gitignore` for Rust + Bevy conventions (AC: #4)**
  - [x] Preserved the existing two lines (`/target`, `.claude/settings.local.json`) exactly.
  - [x] Appended the Rust + Bevy + IDE + OS block verbatim from Dev Notes. New line count: 14 (was 2).
  - [x] `git status` reports only intended changes: `M .gitignore`, `M Cargo.toml`, `M _bmad-output/implementation-artifacts/sprint-status.yaml`, `?? _bmad-output/implementation-artifacts/1-3-...md`, `?? clippy.toml`, `?? rust-toolchain.toml`, `?? rustfmt.toml`. No stray `.DS_Store` / `imported_assets/` / IDE files found in the tree (`find` scan returned nothing).

- [x] **Task 6 — Verification grep + full-output evidence (all ACs)**
  - [x] Consolidated pass of `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo check` — all three exit 0.
  - [x] Per-log grep counts on `warning:|error:`:
    - `/tmp/story-1-3-fmt.log` — 0 lines, 0 hits.
    - `/tmp/story-1-3-clippy.log` — 3 lines, 1 hit (the expected manifest warning).
    - `/tmp/story-1-3-check.log` — 2 lines, 1 hit (same expected manifest warning).
  - [x] Zero `error:` hits across all three logs; zero *new* warnings introduced by this story (the single recurring warning is the deferred-to-1.5 `cfg(debug_assertions)` finding).

- [x] **Task 7 — Scope guardrails (what this story does NOT do)**
  - [x] `src/main.rs` unchanged — still cargo-default `fn main() { println!("Hello, world!"); }`.
  - [x] `cfg(debug_assertions)` block in `Cargo.toml` left as-is — Story 1.5's scope, per `deferred-work.md:14-22`.
  - [x] `[profile.dev.build-override] opt-level = 0` NOT added — re-deferred to M4 upgrade window. `deferred-work.md:8` annotated with the re-defer rationale (2026-04-23 stamp).
  - [x] No `.github/` directory created — CI is Story 1.4's scope (`ls .github/` reports "No such file or directory").
  - [x] No `#![deny(...)]` attributes added to `src/main.rs`. None needed — clippy passed strict on cargo-default code.
  - [x] `cargo update` not run; `Cargo.lock` diff is 0 lines. M0 pins intact.

- [x] **Task 8 — Commit plan (no auto-commit)**
  - [x] No `git commit` issued — matching 1.1 / 1.2 precedent. All changes staged for Till's manual commit.
  - [x] Suggested commit (source artifacts only, single-line subject, no `Co-Authored-By` trailer — matches `4ca3869` / `abe7742` / `113eebe` / `0cbe8a3` / `23ab9ec` / `48cedcd`):
    ```bash
    git add rust-toolchain.toml rustfmt.toml clippy.toml .gitignore Cargo.toml
    git commit -m "chore: toolchain, lint, and format configuration (Story 1.3)"
    ```
  - [x] Suggested follow-up commit for BMad bookkeeping (after source commit lands):
    ```bash
    git add _bmad-output/implementation-artifacts/1-3-toolchain-lint-and-format-configuration.md \
            _bmad-output/implementation-artifacts/sprint-status.yaml \
            _bmad-output/implementation-artifacts/deferred-work.md
    git commit -m "bmad: story 1.3 complete — toolchain, lint, format configuration"
    ```

### Review Findings

_Added 2026-04-23 by `bmad-code-review` (3-layer adversarial review: Blind Hunter + Edge Case Hunter + Acceptance Auditor). Raw findings: 22. Triage outcome: 1 Decision-Needed, 0 Patch, 3 Defer, 18 Dismissed. Acceptance Auditor verdict: **Approve** (5/5 ACs PASS, 0 PARTIAL, 0 FAIL). Blind Hunter's one HIGH-severity claim — "rustfmt uses unstable options without `unstable_features`" — was empirically **refuted** by Edge Case Hunter: `use_field_init_shorthand` and `use_try_shorthand` are stable on rustfmt 1.8.0 (shipped with rustc 1.94.1). No blockers._

- [x] [Review][Decision → Patched] **`rust-analyzer` added to `rust-toolchain.toml` components** — decision 2026-04-23: Till chose to include it (onboarding convenience > minor CI bandwidth cost). `rust-toolchain.toml` line 6 now reads `components = ["rustfmt", "clippy", "rust-analyzer"]`. `rustup show` confirmed rust-analyzer downloaded on first invocation under the pinned 1.94.1 channel (log: `/tmp/story-1-3-rustup-show-post-review.log`). Regression-check: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo check` all still exit 0 with no new warnings.
- [x] [Review][Defer] **MSRV `1.89` is declared but no CI job ever exercises it** `[Cargo.toml:5]` — deferred, pre-existing. Blind Hunter + Edge Case Hunter context: `rust-version = "1.89"` is a promise about the project's minimum supported Rust, but `rust-toolchain.toml` pins everyone (dev + CI) to `1.94.1`. Nothing verifies the claim. Naturally Story 1.4's scope — add a CI job running `cargo +1.89 check` to validate.
- [x] [Review][Defer] **`project: asteroids3D` typo persists in `sprint-status.yaml`** `[sprint-status.yaml:3,39]` — deferred, pre-existing. Blind Hunter caught the typo (real repo name is `asteroids3D`, fixed in Cargo.toml by commit `113eebe`, but BMad bookkeeping files still carry the original misspelling from the planning artifacts). Not introduced by 1.3; same typo exists in `architecture.md`, `prd.md`, `epic-1-*.md`. Systematic fix deserves its own chore story (touches many planning files).
- [x] [Review][Defer] **`rust-toolchain.toml` lacks a `targets` field** `[rust-toolchain.toml:4-7]` — deferred, pre-existing. Edge Case Hunter: fine as long as CI uses one native runner per OS (matches architecture decision). If a future story ever cross-compiles (e.g. Windows-from-Linux), `rust-toolchain.toml` will need explicit `targets = [...]`. Surface the requirement in Story 1.4's DoD, not here.

### Review Findings — Dismissed (recorded for future-reviewer context)

_These were raised but rejected during triage. Kept as breadcrumbs so a future reviewer does not re-litigate them._

- **[blind] rustfmt.toml uses unstable options** — **REFUTED**. Edge Case Hunter empirically verified `use_field_init_shorthand`, `use_try_shorthand`, and `edition = "2024"` are all stable on rustfmt 1.8.0 (shipped with 1.94.1). Blind Hunter had no rustfmt-version context.
- **[blind] Pinned `1.94.1` is future-dated / unverified** — REFUTED: the toolchain is already installed locally and Story 1.2 used it. `rustc --version` confirms.
- **[blind] `profile = "minimal"` may omit clippy/rustfmt** — REFUTED: `components = ["rustfmt", "clippy"]` explicitly adds them; rustup's component-on-minimal-profile path is a well-used pattern.
- **[blind] clippy thresholds are loosened pre-emptively** — spec-authorized in Dev Notes → clippy.toml Skeleton. Judgment call, not drift.
- **[blind] No `[lints.clippy]` or `#![warn(clippy::all)]` scaffold** — spec explicitly routes lint deny/allow lists to Epic 2+ when gameplay modules arrive. Out of scope.
- **[blind] `edition = "2024"` duplicated between rustfmt.toml and Cargo.toml** — spec skeleton lists it intentionally; minor redundancy, addressable at M4 upgrade window alongside other editions-related work.
- **[blind] `.gitignore` omits `*.rs.bk` / `flamegraph.svg` / `.env`** — modern rustfmt does not produce `.rs.bk`; profiling artifacts arrive in M2; no secrets in M0. Spec-bounded.
- **[blind] `/imported_assets` is anchored-fragile for future workspaces** — architecture explicitly keeps single-crate workspace through M0–M3.
- **[blind] Re-deferral lacks measurable trigger** — M4 is a defined PRD milestone with documented upgrade-window budget, not a vague "later".
- **[blind] Sprint status `review` without any tests** — spec explicitly declares "No `#[test]`s are added. All verification is CLI-command-based."
- **[blind] No `[profile.dev.build-override]` added** — re-defer decision is documented in `deferred-work.md` with rationale.
- **[blind] Inconsistent alignment/comment styles across the three new config files** — micro-style; no mandate in the spec.
- **[blind] No CI hook referenced** — Story 1.4 scope, explicitly documented in Task 7 guardrails.
- **[edge] `*.iml` ignores future IntelliJ contributors' committed module files** — Edge Case Hunter itself recommended "Leave as-is".
- **[edge] clippy threshold direction ambiguity** — Edge Case Hunter verified all three thresholds are LOOSENED vs defaults (consistent direction). Low-value comment addition.
- **[auditor] "Stable channel" wording vs explicit `1.94.1` pin** — Auditor self-dismissed as "spec-authorized narrowing, not drift".
- **[auditor] `sprint-status.yaml` + this story file touch themselves** — Auditor self-dismissed as standard BMad bookkeeping, reconciled in the File List section.
- **[auditor] `/tmp/story-1-3-fmt.log` shows 1 line via Read tool vs "0 lines" in Debug Log** — trailing-newline accounting nit; functionally equivalent (no fmt findings either way).

## Dev Notes

### Why this story exists

Story 1.1 proved the manifest resolves; Story 1.2 proved the plugins compile. **Story 1.3 freezes the developer-ergonomics surface** — the toolchain, the format, and the lint rules — so every subsequent story (including Story 1.4's CI matrix) has a single source of truth to point at. Everything 1.3 produces is configuration; no gameplay code changes. [Source: architecture.md:140-142,154-158,537-545; prd.md:111]

### Toolchain Pinning Decision

**Pin to `1.94.1` (not `stable`) for CI reproducibility.** The architecture calls for "Pin via `rust-toolchain.toml` to control CI reproducibility" (architecture.md:140) — `stable` floats, defeating the purpose. Story 1.2 already verified that `rustc 1.94.1 (e408947bf 2026-03-25)` compiles the current `Cargo.toml` cleanly, so pinning here locks in a known-good state. Upgrades happen at the M4/M6/M9 upgrade windows, same cadence as plugin-version bumps.

Skeleton:

```toml
# rust-toolchain.toml
# Pinned for CI reproducibility. Upgrade windows: M4, M6, M9. [Source: architecture.md:140; prd.md:114-115]

[toolchain]
channel    = "1.94.1"
components = ["rustfmt", "clippy"]
profile    = "minimal"
```

Note: `targets` is intentionally omitted. The three-platform matrix (Story 1.4) runs one job per OS, so each runner already has its own native target by default. Cross-compilation is not an M0 concern.

### rustfmt.toml Skeleton

Keep minimal — rustfmt defaults are sane and churn-minimizing. Only pin what matches Bevy-community and Till's intermediate-developer preference. [Source: architecture.md:156; prd.md:111]

```toml
# rustfmt.toml
# Project style. Defaults are acceptable; only opinionated knobs below.

edition        = "2024"
max_width      = 100
newline_style  = "Unix"
use_field_init_shorthand = true
use_try_shorthand        = true
```

**Why `max_width = 100` (not the rustfmt default 100 — identical, but made explicit):** Bevy community convention. Reading wide query tuples on narrower editors becomes painful past 100.

**Why no `imports_granularity` / `group_imports`:** those are nightly-only rustfmt options. Stable rustfmt will `warning: unstable`. Do not add them until the toolchain is on nightly, which it will not be in M0.

### clippy.toml Skeleton

Project-wide thresholds only — deny/allow lists belong in `src/main.rs` `#![...]` attributes as gameplay modules arrive (Epic 2+). [Source: architecture.md:156]

```toml
# clippy.toml
# Thresholds only. Lint deny/allow lists live in source via #![warn(...)] attributes
# once gameplay modules land (Epic 2+).

cognitive-complexity-threshold = 30
too-many-arguments-threshold   = 8
type-complexity-threshold      = 500
```

**Why these numbers:** generous enough that no current code triggers them; tight enough that the gameplay modules arriving in Epic 2 will notice if a system balloons. Tunable at M4 review if a legitimate gameplay system starts grazing a threshold.

**What is NOT in clippy.toml:** lint deny lists. Clippy's `-D warnings` flag already denies every warning at command-invocation time; deny-listing specific lints in `clippy.toml` is redundant. If specific lints need to be allowed (e.g., `clippy::type_complexity` for Bevy Query tuples), allow them at the `#![allow(clippy::type_complexity)]` module level in the offending file when it exists — not pre-emptively now.

### .gitignore Additions

Current `.gitignore` (2 lines — preserve exactly):

```
/target
.claude/settings.local.json
```

Append the following block. One blank line separates the new block from the existing two lines.

```
# Bevy processed-asset cache (generated at runtime when asset-processing is enabled)
/imported_assets

# IDE
.vscode/
.idea/
*.iml

# OS artifacts
.DS_Store
Thumbs.db
```

**Why `imported_assets/` not `assets-cache/`:** Bevy 0.12+ renamed the processed-asset cache to `imported_assets/` and writes it next to `assets/` at runtime when `asset-processing` is enabled (this project does not currently enable it, but per the starter decision the convention is pre-applied so the ignore is correct the day someone flips it on). [Source: architecture.md:536-545]

**Why not ignore `.env` / secrets:** no secrets files exist or are planned in M0. If Steam/Apple-notarization secrets arrive in M6, extend `.gitignore` there.

### Previous Story Intelligence (from 1-1 and 1-2)

**From Story 1.2's Review Findings (already resolved into this story's scope):**
- `[Review][Defer] [package].rust-version MSRV not set` → resolved by AC #5 / Task 2 here. [Source: 1-2-…md:58]
- `[Review][Defer] [profile.dev.build-override] opt-level = 0 not added` → this story **re-defers** to a milestone-gate upgrade window (M4 earliest). Record in `deferred-work.md` alongside the 1.1 entry. [Source: 1-2-…md:59; deferred-work.md:8]

**From Story 1.2's Commit Convention note:**
- "The project's four existing commits use plain single-line messages without `Co-Authored-By` trailers." Now 6 commits, same convention. Match exactly. [Source: 1-2-…md:166]

**From Story 1.1's Known Issues:**
- The `cfg(debug_assertions)` semantic bug in `Cargo.toml` is **not** fixed here; Story 1.5 owns it because that is the first story that registers the egui plugin. [Source: deferred-work.md:12-22; 1-2-…md:154-156]

### Git Intelligence

Recent commits (newest first, 6 total on `master`):

| SHA | Subject | Relevance to 1.3 |
|---|---|---|
| `48cedcd` | `bmad: story 1.2 complete — plugin compatibility gate passed` | Sprint bookkeeping commit — precedent for BMad-artifact commits being separate from source-artifact commits. |
| `23ab9ec` | `docs: add plugin compatibility verification gate (Story 1.2)` | Source-artifact commit. `docs:` type prefix is the established pattern for documentation commits. `chore:` is the expected type prefix for 1.3. |
| `0cbe8a3` | `docs: log review correction for cfg(debug_assertions) finding` | `deferred-work.md` is a living document; this story appends one more entry (the re-deferred `build-override` polish). |
| `113eebe` | `fix: correct package name typo asteroids3D -> asteroids3D` | **Critical context:** the package name is `asteroids3D` (corrected), **not** `asteroids3D` as written throughout architecture.md / PRD / epic files. Do **not** re-rename. |
| `abe7742` | `planning: import BMad artifacts` | Planning bulk import — irrelevant to 1.3. |
| `4ca3869` | `chore: bootstrap Cargo project (Story 1.1)` | Original manifest + default `src/main.rs`. This story's baseline file set. |

**Commit-type convention observed:** `chore:`, `docs:`, `fix:`, `planning:`, `bmad:`. `chore:` is the right prefix for Story 1.3's config-file commit.

### Architecture Compliance

- **Config-file location:** all four new files live flat at the project root, matching `Project Structure & Boundaries → Configuration Files: Flat at project root (Rust convention)`. [Source: architecture.md:536-545,755]
- **CI reproducibility:** the pinned toolchain is the mechanism the architecture calls out at line 140; Story 1.4 will simply add `rust-toolchain.toml` to the workflow checkout and inherit the pin.
- **Quality-lint discipline:** architecture line 156 says `#![deny(clippy::needless_pass_by_value)]` and similar quality lints on. Those live on the `src/` side; this story only sets up the invocation (`cargo clippy -- -D warnings`) that makes such attributes effective. The attributes themselves will land with the first gameplay module in Epic 2.
- **No greenfield `lib.rs` split:** single-crate workspace per architecture.md:130-131. Do not introduce `Cargo.toml` workspace tables.

### Library/Framework Requirements

No new crate dependencies. This story is pure configuration.

### File Structure Requirements

Files added/modified by this story, all paths relative to project root:

| Path | Add/Modify | Purpose |
|---|---|---|
| `rust-toolchain.toml` | Add (new) | Pin stable channel 1.94.1 + components. |
| `rustfmt.toml` | Add (new) | Project fmt style. |
| `clippy.toml` | Add (new) | Project lint thresholds. |
| `.gitignore` | Modify | Append Rust + Bevy + IDE + OS block. Preserve existing 2 lines. |
| `Cargo.toml` | Modify | Add `rust-version = "1.89"` to `[package]` only. No other edits. |

Files explicitly **not** touched by this story: `src/main.rs`, `Cargo.lock`, `docs/plugin-compatibility.md`, any `.github/` path (owned by 1.4), any `src/<module>/` file (none yet exist; first gameplay module is Epic 2).

### Testing Requirements

- No `#[test]`s are added. All verification is CLI-command-based, captured in **Debug Log References**.
- Required command set (all must exit 0):
  ```bash
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo check
  ```
- **Full-output evidence, not just exit-0.** Capture each command's full stdout+stderr to a `/tmp/story-1-3-<cmd>.log` and grep with `grep -E 'warning:|error:'` to confirm zero *unexpected* hits. The known `cfg(debug_assertions)` warning from Story 1.2 still shows on `cargo check` — expect exactly one hit, same message as 1.2's log. Zero new warnings. [Source: MEMORY.md → feedback_full_build_output.md]
- Bevy-integration tests remain deferred post-M3. [Source: architecture.md:144-146]

### Latest Technical Information

- **`rust-toolchain.toml` schema (rustup 1.27+, current):** the `[toolchain]` table accepts `channel`, `components`, `targets`, `profile`. `profile = "minimal"` omits `rust-docs` — right for CI and the dev box (Till can always `rustup doc` out-of-band). Source: rustup book, verified April 2026.
- **`rustfmt.toml` stability:** `edition`, `max_width`, `newline_style`, `use_field_init_shorthand`, `use_try_shorthand` are all stable options on `rustfmt 1.7.x` (ships with rustc 1.94). No nightly-only options in this story.
- **`clippy.toml` thresholds:** `cognitive-complexity-threshold` + `too-many-arguments-threshold` + `type-complexity-threshold` are all stable clippy configuration keys; verified against clippy 0.1.94 (ships with rustc 1.94).

### What this story explicitly does NOT fix

Enumerated here so the review step does not flag them as "missed":

1. The `cfg(debug_assertions)` semantic bug in `Cargo.toml` — **Story 1.5** owns it (feature-flag fix folded with first egui usage). [Source: deferred-work.md:14-22]
2. `[profile.dev.build-override] opt-level = 0` — re-deferred to M4 upgrade window. [Source: deferred-work.md:8]
3. Cross-platform verification — **Story 1.4** (CI matrix) verifies on Windows + Linux + macOS runners. This story's verification is macOS-local only.
4. `rustc` / `cargo` version bumps — pinned at 1.94.1 until an M4/M6/M9 upgrade window.

### Project Structure Notes

- Package name is **`asteroids3D`** (corrected, commit `113eebe`), not the `asteroids3D` typo that persists throughout the planning docs. Do not re-rename.
- `edition = "2024"` is already set in `Cargo.toml` — confirmed compatible with rustc 1.94.1. No change needed.
- `.gitignore` already contains `.claude/settings.local.json` — this is the Claude-harness settings file; preserve it untouched.
- No `rust-toolchain` (extension-less variant) file exists. Only add `rust-toolchain.toml`.

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md#Story-1.3 (lines 54-76)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Architectural-Decisions-Provided-by-Starter-Choice (lines 129-163)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Build-Tooling (lines 139-142)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Development-Experience (lines 154-158)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Complete-Project-Directory-Structure (lines 534-545)]
- [Source: _bmad-output/planning-artifacts/architecture.md#File-Organization-Patterns (line 755)]
- [Source: _bmad-output/planning-artifacts/prd.md#Starter-Template-M0-Gate (lines 105-112)]
- [Source: _bmad-output/planning-artifacts/prd.md#Version-Pinning-Governance (lines 114-115)]
- [Source: _bmad-output/implementation-artifacts/1-1-bootstrap-cargo-project-with-hand-authored-cargo-toml.md (File List, Review Findings)]
- [Source: _bmad-output/implementation-artifacts/1-2-plugin-compatibility-verification-gate.md#Commit-Convention (line 166)]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md (MSRV, build-override, cfg(debug_assertions) entries — lines 7-22)]

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Claude Code) — 1M-context configuration.

### Debug Log References

All commands issued from project root `/Users/tillfechteler/Projekte/rust/asteroids3D/`, toolchain `1.94.1-aarch64-apple-darwin`.

| Command | Exit | Log file | Log lines | `grep -c -E 'warning:|error:'` |
|---|---|---|---|---|
| `rustup show` | 0 | `/tmp/story-1-3-rustup-show.log` | 14 | n/a (informational) |
| `cargo check` (after MSRV) | 0 | `/tmp/story-1-3-cargo-check-msrv.log` | 2 | 1 |
| `cargo fmt --check` | 0 | `/tmp/story-1-3-fmt.log` | 0 | 0 |
| `cargo clippy --all-targets -- -D warnings` | 0 | `/tmp/story-1-3-clippy.log` | 3 | 1 |
| `cargo check` (final) | 0 | `/tmp/story-1-3-check.log` | 2 | 1 |

The single recurring warning across check/clippy logs is:
> `warning: Found 'debug_assertions' in 'target.'cfg(...)'.dependencies'. This value is not supported for selecting dependencies and will not work as expected.`

This is the `cfg(debug_assertions)` finding first documented in Story 1.1's post-commit review and carried forward in `deferred-work.md:14-22`. It is a **Cargo-manifest** warning (not a rustc/clippy warning), which is why `cargo clippy -- -D warnings` still exits 0. Fix is scheduled for Story 1.5 when the egui plugin is first registered. Zero new warnings introduced by this story.

`rustup show` extract: `active toolchain: 1.94.1-aarch64-apple-darwin — active because: overridden by '/Users/tillfechteler/Projekte/rust/asteroids3D/rust-toolchain.toml'`.

`Cargo.lock` diff after all edits: 0 lines — no dependency resolution churn.

### Completion Notes List

**Status: ✅ Story 1.3 complete.** All five ACs satisfied, all eight tasks checked, all scope guardrails honored.

**Deliverables (5 config files, paths relative to project root):**

| File | Action | Lines | Purpose |
|---|---|---|---|
| `rust-toolchain.toml` | Added (new) | 8 | Pin `channel = "1.94.1"`, `components = [rustfmt, clippy]`, `profile = "minimal"`. CI (Story 1.4) will inherit. |
| `rustfmt.toml` | Added (new) | 7 | Project fmt style — minimal, defaults-plus-opinion. |
| `clippy.toml` | Added (new) | 7 | Thresholds only: cognitive-complexity=30, too-many-arguments=8, type-complexity=500. |
| `.gitignore` | Modified | 2 → 14 | Preserved existing 2 lines, appended Bevy `imported_assets/` + IDE + OS block. |
| `Cargo.toml` | Modified | +1 | Added `rust-version = "1.89"` in `[package]`; no other edits. |

**Deliverables (documentation side-effects):**
- `_bmad-output/implementation-artifacts/deferred-work.md` — annotated existing `[profile.dev.build-override]` bullet with re-defer rationale to M4 upgrade window (2026-04-23 stamp). No new bullets.

**Verification evidence:**
- `cargo fmt --check` exit 0, empty log.
- `cargo clippy --all-targets -- -D warnings` exit 0, 3-line log, 1 warning (known Cargo-manifest warning, not a rustc/clippy lint).
- `cargo check` exit 0, 2-line log, 1 warning (same known warning).
- `Cargo.lock` unchanged (0-line diff).
- `src/main.rs` unchanged (cargo-default `println!("Hello, world!")`).
- No `.github/` directory created. No gameplay code added. No pin bumps. No `cargo update`.

**Deferrals explicitly preserved:**
1. `cfg(debug_assertions)` semantic bug — remains Story 1.5's scope (fix lands with first egui plugin registration).
2. `[profile.dev.build-override] opt-level = 0` — re-deferred to the M4 upgrade window so manifest churn stays concentrated at governance-approved windows.

**Commit pending user action.** Two suggested commits (split per 1.1/1.2 convention — source artifacts separate from BMad bookkeeping). See Task 8.

**Scope guardrails: all honored.** No edits to `src/`, no CI workflows, no new dependencies, no pin bumps.

### File List

All paths relative to project root.

**Added:**
- `rust-toolchain.toml` — toolchain pin (8 lines; post-review: `components` extended with `rust-analyzer`).
- `rustfmt.toml` — format config (7 lines).
- `clippy.toml` — lint config (7 lines).

**Modified (source artifacts — part of the `chore:` commit):**
- `Cargo.toml` — `+rust-version = "1.89"` in `[package]` (diff = +1 line).
- `.gitignore` — appended Bevy + IDE + OS block (diff = +12 lines, existing 2 lines preserved).

**Modified (BMad bookkeeping — part of the `bmad:` follow-up commit):**
- `_bmad-output/implementation-artifacts/1-3-toolchain-lint-and-format-configuration.md` — this file: tasks checked, Dev Agent Record populated, status flipped to `review`.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `development_status[1-3-…]` set to `review`; `last_updated` bumped to 2026-04-23.
- `_bmad-output/implementation-artifacts/deferred-work.md` — re-defer annotation appended to existing `[profile.dev.build-override]` bullet.

**Unchanged (verified via `git diff`):**
- `src/main.rs`, `Cargo.lock`, `docs/plugin-compatibility.md` — scope guardrails honored.

## Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-23 | claude-opus-4-7 (create-story) | Story 1.3 drafted. Scope: pin toolchain 1.94.1, author rustfmt.toml + clippy.toml, extend .gitignore per Rust+Bevy conventions, declare MSRV 1.89 in Cargo.toml. `cfg(debug_assertions)` fix deferred to 1.5; `build-override` polish re-deferred to M4. Status: ready-for-dev. |
| 2026-04-23 | claude-opus-4-7 (dev-story) | Story 1.3 implemented. 3 new config files (`rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`), 2 modifications (`Cargo.toml` +MSRV, `.gitignore` extended). `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo check` all exit 0. Build-override re-defer annotation added to `deferred-work.md`. `src/main.rs`, `Cargo.lock`, and all other files untouched. Status: in-progress → review. |
| 2026-04-23 | claude-opus-4-7 (code-review) | 3-layer adversarial review (Blind Hunter + Edge Case Hunter + Acceptance Auditor). 22 raw findings → 1 Decision-Needed, 0 Patch, 3 Defer, 18 Dismissed. Acceptance Auditor verdict: Approve (5/5 ACs PASS). Blind Hunter's one HIGH-severity claim refuted empirically by Edge Case Hunter. Post-review patch: Till chose to add `rust-analyzer` to `rust-toolchain.toml` components — applied, all validations green. 3 Defer findings appended to `deferred-work.md` (MSRV not CI-exercised, `asteroids3D` typo, missing `targets` field) — all flagged for Story 1.4's scope. Status: review → done. |
