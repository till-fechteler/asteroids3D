# Story 1.2: Plugin Compatibility Verification Gate

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a developer,
I want explicit verification that every pinned plugin has a working Bevy-0.18-compatible release,
So that I discover fork-or-substitute decisions before writing gameplay code, not three weeks into M2.

## Acceptance Criteria

1. **All four third-party plugins compile.** Given `Cargo.toml` from Story 1.1, `cargo check` executed on the local machine completes without errors, and `bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `bevy_egui` all compile against Bevy 0.18. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:40-43]
2. **Failure resolution path documented.** If any plugin fails to compile, `docs/plugin-compatibility.md` records a resolution entry — plugin name, error summary, and a resolution path that is one of: (a) upstream patch exists → pin updated, (b) fork-and-inline per PRD Tech-Risk strategy, (c) substitute alternative plugin. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:45-48]
3. **Success artifact persisted.** When all plugins resolve, `docs/plugin-compatibility.md` lists verification date, Rust toolchain version, Bevy version, and each plugin version. The story's gate is marked passed, unblocking Stories 1.3+. [Source: epics/epic-1-foundation-plugin-compatibility-gate.md:50-52]

## Tasks / Subtasks

- [x] **Task 1 — Clean-cache compilation verification (AC: #1)**
  - [x] `cargo clean` (removed 3359 files, 566.6 MiB) then `cargo check 2>&1 | tee /tmp/story-1-2-cargo-check.log`.
  - [x] Exit status 0; tail shows `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 43.61s`. Grepped the 443-line log for `warning:|error:` — exactly one hit, the expected `cfg(debug_assertions)` warning.
  - [x] Versions captured via `cargo tree --depth 1`: bevy 0.18.1, avian3d 0.6.1, bevy_mod_outline 0.12.0, bevy_kira_audio 0.25.0, leafwing-input-manager 0.20.0, bevy_egui 0.39.1 — all match Story 1.1's Completion Notes.
  - [x] Toolchain: rustc 1.94.1 (e408947bf 2026-03-25), cargo 1.94.1 (29ea6fb6a 2026-03-24).
  - [x] `cfg(debug_assertions)` warning observed and logged as expected; no fix attempted (deferred to Story 1.5).

- [x] **Task 2 — Create `docs/plugin-compatibility.md` (AC: #3)**
  - [x] `mkdir -p docs` at project root; directory was previously absent.
  - [x] `docs/plugin-compatibility.md` authored per the schema in Dev Notes — verification date 2026-04-22, Rust 1.94.1, one row per plugin, bevy/avian in Core engine section, four plugins in Third-party plugins section.
  - [x] Status line: `✅ GATE PASSED — all four third-party plugins compile against Bevy 0.18 on macOS 26.4.1 / arm64, 2026-04-22.` (OS string from `sw_vers` + `uname -m`.)
  - [x] Known Issues / Deferred section links to `_bmad-output/implementation-artifacts/deferred-work.md` for the `cfg(debug_assertions)` finding.
  - [x] File size: 41 lines. Well under the 120-line cap.

- [x] **Task 3 — Failure-path handling (AC: #2) — contingency only**
  - [x] No-op: all four plugins compiled cleanly on first clean `cargo check`. No Resolution Log rows added. Doc status remains ✅ GATE PASSED.
  - [x] Resolution-path guidance retained as reference in the doc for future upgrade-window stories (M4 / M6 / M9).
  - [x] No doc status change required.

- [x] **Task 4 — Commit and gate-pass signal (AC: #3)**
  - [ ] **Pending user-initiated commit** — following Story 1.1's precedent ("this workflow does not auto-commit"), the docs file is left unstaged. Suggested stage + message below. [Source: 1-1-…md:203]
  - [x] Commit message prepared: `docs: add plugin compatibility verification gate (Story 1.2)`. Single-line, matches Till's subject-line convention across commits `4ca3869` / `abe7742` / `113eebe` / `0cbe8a3`.
  - [x] `Cargo.toml` and `Cargo.lock` untouched — confirmed via `git status`: only `docs/` (untracked) and `_bmad-output/implementation-artifacts/sprint-status.yaml` (BMad bookkeeping) changed. No Task 3 (a) pin update was required.

- [x] **Task 5 — Scope guardrails (what this story did NOT do)**
  - [x] `src/main.rs` unchanged — still cargo-default `println!("Hello, world!")`. Bevy App assembly remains Story 1.5.
  - [x] No `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` added; `.gitignore` untouched — Story 1.3 scope.
  - [x] No CI workflows added — Story 1.4 scope.
  - [x] `cfg(debug_assertions)` warning observed but not patched — deferred to Story 1.5 per `deferred-work.md`.
  - [x] No dependency versions bumped; M0 pins intact until the next milestone-gate upgrade window (M4).
  - [x] `cargo update` not run; `Cargo.lock` unchanged.

## Dev Notes

### Why this story exists (scope clarification)

Story 1.1 authored `Cargo.toml` and empirically confirmed all dependencies resolve; the dev agent already captured resolved versions in 1.1's Completion Notes. **Story 1.2 is the formal gate record** — the architecture and PRD explicitly call out third-party-plugin compatibility as the #1 M0 risk, and the artifact `docs/plugin-compatibility.md` becomes the source of truth for which plugins are approved, at which versions, on what toolchain. This file is referenced by future upgrade-window stories (M4/M6/M9) and by any fork-or-substitute decision. [Source: architecture.md:70,885; prd.md:401-403,442]

**Key inversion from the original epic text:** the epic describes the gate as if compilation and documentation happen together. In practice, Story 1.1 already proved compilation. This story's dev-work is therefore ~80% documentation + ~20% re-verification.

### Context inherited from Story 1.1

Story 1.1's dev agent resolved these exact versions on 2026-04-22 (do not "update" unless Task 1 shows a resolution change):

| Declared in `Cargo.toml` | Pinned | Resolved in Lockfile |
|---|---|---|
| `bevy` | `0.18` | `0.18.1` |
| `avian3d` | `0.6` | `0.6.1` |
| `bevy_mod_outline` | `0.12` | `0.12.0` |
| `bevy_kira_audio` | `0.25` | `0.25.0` |
| `leafwing-input-manager` | `0.20` | `0.20.0` |
| `bevy_egui` (cfg-dev-only, currently always-on; see Known Issues) | `0.39` | `0.39.1` |

[Source: 1-1-bootstrap-cargo-project-with-hand-authored-cargo-toml.md:190-200]

**No plugin version gap was found** in Story 1.1. Expect Task 1 to confirm this. If Task 1 disagrees (e.g. because the lockfile drifted), treat the disagreement itself as a finding and record it.

### Compatibility Doc Schema (`docs/plugin-compatibility.md`)

Structure the file exactly as follows — the structure is consumed by future upgrade-window stories:

```markdown
# Plugin Compatibility Matrix

**Status:** ✅ GATE PASSED — all four third-party plugins compile against Bevy 0.18 on <os>, <verification-date>.

**Verification date:** <ISO date, e.g. 2026-04-23>
**Platform verified:** <e.g. macOS 14 / Apple Silicon (Till's dev machine)>
**Verification scope:** local `cargo clean && cargo check` only. Cross-platform verification is Story 1.4 (CI matrix).

## Toolchain

| Component | Version |
|---|---|
| Rust (`rustc`) | <from `rustc --version`> |
| Cargo | <from `cargo --version`> |
| Edition | 2024 |

## Core engine

| Crate | Declared pin | Resolved | Role |
|---|---|---|---|
| `bevy` | `0.18` | `0.18.1` | Engine / ECS |
| `avian3d` | `0.6` | `0.6.1` | Physics (XPBD) |

## Third-party plugins (gated)

| Crate | Declared pin | Resolved | Bevy compat | Role | Risk |
|---|---|---|---|---|---|
| `bevy_mod_outline` | `0.12` | `0.12.0` | bevy 0.18.1 | FR49 silhouette outlines | Upgrade-churn; fork-ready. [Source: prd.md:401-403] |
| `bevy_kira_audio` | `0.25` | `0.25.0` | bevy 0.18.1 | FR23 spatial audio channels | Upgrade-churn; fork-ready. |
| `leafwing-input-manager` | `0.20` | `0.20.0` | bevy 0.18.1 | FR1–FR8 input abstraction | Well-maintained. |
| `bevy_egui` | `0.39` | `0.39.1` | bevy 0.18 sub-crates | Dev-only debug panels | Dev-path only. Currently leaks into release builds — see Known Issues. |

## Resolution Log

<!-- Populated only if a plugin fails to compile. Format: plugin | error | resolution path | link -->

_(empty — no resolutions required at M0 start)_

## Known Issues / Deferred

- `[target.'cfg(debug_assertions)'.dependencies]` does not strip `bevy_egui` from release builds — see `_bmad-output/implementation-artifacts/deferred-work.md` (Review correction, 2026-04-22). Scheduled fix: Story 1.5 when the egui plugin is first registered.

## Upgrade policy

Version bumps happen only at M4, M6, M9 milestone-gate windows, with a 4–6 h budgeted per minor-version migration. No ad-hoc mid-milestone upgrades. [Source: prd.md:406; requirements-inventory.md:114-115]
```

Write exactly this structure; the table columns and section order are what future stories will parse.

### Fork-or-substitute decision tree (Task 3)

Only engaged if Task 1's `cargo check` fails. The ordering below reflects escalating cost:

1. **Is there a newer crates.io release pinned against Bevy 0.18?** → update the pin (Task 3 path (a)), re-run `cargo check`. Cheapest resolution.
2. **Is the plugin small (< 500 lines) and low-churn?** → propose fork-and-inline as a new story (Task 3 path (b)). This is the PRD's Tech-Risk mitigation strategy. Do not fork in this story.
3. **Is there a comparable alternative plugin with a 0.18 release?** → propose substitution (Task 3 path (c)). Substitution is an architecture-impacting change — route to `correct-course` before adopting.
4. **None of the above?** → stop and escalate. The gate does not pass. Subsequent stories remain blocked until the failure is resolved.

[Source: prd.md:401-403,442; architecture.md:56-70,885]

### Platform matrix context

This story verifies **only on the developer's local machine** (macOS / Apple Silicon). Cross-platform (Windows + Linux) verification is Story 1.4's responsibility via the GitHub Actions CI matrix. The doc's **Verification scope** line must make this explicit so future readers don't mistake a local-only pass for a full three-platform pass. [Source: epic-1-foundation-plugin-compatibility-gate.md:86-91,97-99; architecture.md:65-68]

### Testing Standards

- No automated tests are required for this story. The AC validation is:
  1. `cargo check` exit status is 0 on a clean cache.
  2. `docs/plugin-compatibility.md` exists and conforms to the schema above.
- Bevy-integration tests remain deferred post-M3. [Source: architecture.md:143-146]

### Why the `cfg(debug_assertions)` finding is NOT in scope here

Story 1.1's post-commit review correctly identified that `[target.'cfg(debug_assertions)'.dependencies]` does not gate `bevy_egui` out of release builds. The recommended fix (feature flag `dev-tools`) is scheduled for Story 1.5 because that is the first story that actually registers the `bevy_egui` plugin — folding the fix and the first usage together keeps the change visible and testable. This story's job is to **document the finding in the compatibility artifact** so it is not lost; the *fix* remains Story 1.5's scope. [Source: deferred-work.md:12-22]

### Project Structure Notes

**New directory.** `docs/` does not yet exist at the project root (confirmed 2026-04-22). Task 2 creates it. The config `project_knowledge: "{project-root}/docs"` in `_bmad/bmm/config.yaml` points here; other documents written by the tech-writer skill will also live here, so the directory's creation is expected infrastructure.

**No code changes.** `src/main.rs` remains the cargo-default `println!("Hello, world!")` from Story 1.1. Do not touch it.

**Cargo manifest immutability.** `Cargo.toml` and `Cargo.lock` are Story 1.1's committed artifacts. Unless Task 3 path (a) is triggered by a real compile failure, leave them untouched.

**Commit convention.** The project's four existing commits use plain single-line messages without `Co-Authored-By` trailers. Match that style for Task 4. [Source: git log: `4ca3869`, `abe7742`, `113eebe`, `0cbe8a3`]

### References

- [Source: _bmad-output/planning-artifacts/epics/epic-1-foundation-plugin-compatibility-gate.md#Story-1.2 (lines 32-52)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Technical-Constraints-Dependencies (lines 53-70)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Current-Versions-verified-April-2026 (lines 90-96)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Cargo-Configuration (lines 132-137)]
- [Source: _bmad-output/planning-artifacts/architecture.md#Important-Gaps-Gap-1 (line 885)]
- [Source: _bmad-output/planning-artifacts/architecture.md#First-Implementation-Priority-M0-start (lines 961-985)]
- [Source: _bmad-output/planning-artifacts/prd.md#Tech-Risk-Third-party-crate-risk (lines 401-403, 442)]
- [Source: _bmad-output/planning-artifacts/prd.md#Build-CI (line 406)]
- [Source: _bmad-output/planning-artifacts/epics/requirements-inventory.md#Starter-Template-M0-Gate (lines 105-112)]
- [Source: _bmad-output/planning-artifacts/epics/requirements-inventory.md#Version-Pinning-Governance (lines 114-115)]
- [Source: _bmad-output/implementation-artifacts/1-1-bootstrap-cargo-project-with-hand-authored-cargo-toml.md#Completion-Notes-List (lines 188-200)]
- [Source: _bmad-output/implementation-artifacts/deferred-work.md (Review correction, lines 12-22)]

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (Claude Code) — 1M-context configuration.

### Debug Log References

- `cargo clean`: removed 3359 files (566.6 MiB) from `target/`.
- `cargo check` log: `/tmp/story-1-2-cargo-check.log` (443 lines). Exit status 0. Duration 43.61 s (cold cache). Tail: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 43.61s`.
- `grep -E 'warning:|error:' /tmp/story-1-2-cargo-check.log` → exactly 1 hit, the expected `cfg(debug_assertions)` warning (deferred to Story 1.5). Zero `error:` matches.
- `cargo tree --depth 1`: confirmed direct dependencies resolved to the versions pinned in `Cargo.toml`. No drift vs. Story 1.1's recorded versions.
- OS string sources: `sw_vers -productName` → `macOS`; `sw_vers -productVersion` → `26.4.1`; `uname -m` → `arm64`.

### Completion Notes List

**Gate result: ✅ PASSED.** All four third-party plugins (`bevy_mod_outline`, `bevy_kira_audio`, `leafwing-input-manager`, `bevy_egui`) compile cleanly against Bevy 0.18 on the local development machine. Full artifact: `docs/plugin-compatibility.md`.

**Resolved versions (2026-04-22 — stable since Story 1.1, no drift):**

| Component | Declared pin | Resolved |
|---|---|---|
| `bevy` | `0.18` | `0.18.1` |
| `avian3d` | `0.6` | `0.6.1` |
| `bevy_mod_outline` | `0.12` | `0.12.0` |
| `bevy_kira_audio` | `0.25` | `0.25.0` |
| `leafwing-input-manager` | `0.20` | `0.20.0` |
| `bevy_egui` | `0.39` | `0.39.1` |

**Toolchain:** rustc 1.94.1 (e408947bf 2026-03-25), cargo 1.94.1 (29ea6fb6a 2026-03-24), edition 2024.

**Platform verified:** macOS 26.4.1 / arm64. Cross-platform (Windows + Linux) verification is Story 1.4's responsibility via CI matrix.

**Warnings encountered:** 1 — the expected `cfg(debug_assertions)` warning from `Cargo.toml`. Deferred to Story 1.5 per `deferred-work.md`; no fix attempted. The warning is preserved in `docs/plugin-compatibility.md#Known-Issues--Deferred` as a visible breadcrumb.

**No pin adjustments, no fork proposals, no substitutions.** Task 3 (failure-path contingency) was a no-op, as predicted by Story 1.1's resolution evidence.

**Commit pending user action.** Per Story 1.1's precedent, this workflow does not auto-commit. Suggested actions for Till:
```bash
git add docs/plugin-compatibility.md
git commit -m "docs: add plugin compatibility verification gate (Story 1.2)"
```
The BMad bookkeeping files (`_bmad-output/implementation-artifacts/1-2-…md`, `sprint-status.yaml`) are intentionally left out of that staging — mirror 1.1's pattern of keeping source-artifact commits separate from BMad-artifact commits if desired.

**Scope guardrails: all honored.** No edits to `src/`, `Cargo.toml`, `Cargo.lock`, `.gitignore`, or any CI/toolchain config. Only new file: `docs/plugin-compatibility.md`.

### File List

All paths relative to project root.

**Added:**
- `docs/plugin-compatibility.md` — plugin compatibility gate artifact; 41 lines; conforms to the schema in this story's Dev Notes.

**Added (directory):**
- `docs/` — new directory at project root; first file is `plugin-compatibility.md`. Aligns with `_bmad/bmm/config.yaml`'s `project_knowledge: "{project-root}/docs"` pointer.

**Modified (BMad bookkeeping — not part of the gate commit):**
- `_bmad-output/implementation-artifacts/1-2-plugin-compatibility-verification-gate.md` — this file: tasks checked, Dev Agent Record populated, status flipped to `review`.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `development_status[1-2-plugin-compatibility-verification-gate]` set to `review`; `last_updated` bumped to 2026-04-22.

**Unchanged (verified via `git status`):**
- `Cargo.toml`, `Cargo.lock`, `src/main.rs` — scope guardrail honored.

## Change Log

| Date | Author | Change |
|---|---|---|
| 2026-04-22 | claude-opus-4-7 (create-story) | Story 1.2 drafted. Scope reframed as documentation + re-verification gate (Story 1.1 already proved compilation). Fork-or-substitute contingency encoded in Task 3. `cfg(debug_assertions)` finding explicitly scoped out (deferred to 1.5). Status: ready-for-dev. |
| 2026-04-22 | claude-opus-4-7 (dev-story) | Gate executed. Clean `cargo check` on macOS 26.4.1 / arm64 passed in 43.61 s with only the expected `cfg(debug_assertions)` warning. `docs/plugin-compatibility.md` authored (41 lines). Task 3 contingency unused (all plugins resolved cleanly). Task 4 commit deferred to user-initiated action per 1.1 precedent. Status: ready-for-dev → review. |
