# Epic 9: Post-Run Photo Mode

From post-run / death screen, player enters Photo Mode with free-cam orbital/dolly + click-to-focus DoF + time-frozen scene. Exports PNG screenshots in 16:9 landscape, 9:16 portrait, or 1:1 square. Marketing-ready aesthetic artifact pipeline. M-alignment: M8. FRs covered: FR40, FR41, FR42.

## Story 9.1: FreeOrbitCamera Component + F3 Dev Toggle

As a developer,
I want a `FreeOrbitCamera` component with orbit/dolly/pan math + an F3 debug toggle gated to `cfg(debug_assertions)`,
So that Photo Mode and dev-time gameplay debugging share one camera implementation without bleeding into release.

**Acceptance Criteria:**

**Given** `src/camera/free_orbit.rs` is authored
**When** `FreeOrbitCamera` component is defined
**Then** fields: `anchor_point: Vec3`, `distance: f32`, `yaw: f32`, `pitch: f32`, `pan_offset: Vec3`
**And** a system computes Transform from these fields each frame (spherical coords around anchor + pan offset)

**Given** `cfg(debug_assertions)` (dev build)
**When** `DebugCameraPlugin` is registered
**Then** F3 toggles between CockpitCamera (gameplay) and a dev FreeOrbitCamera
**And** on enable, FreeOrbitCamera spawns with `anchor_point = PlayerShip position`, `distance = 20.0`, `yaw = 0.0`, `pitch = -0.3`
**And** on disable, the dev camera despawns and CockpitCamera resumes

**Given** release build (`cfg(not(debug_assertions))`)
**When** compiled
**Then** F3 does nothing — `DebugCameraPlugin` is not registered
**And** no 3rd-person camera is accessible in release (FR8 cockpit-only enforcement intact)

**Given** FreeOrbitCamera control mappings (shared between dev F3 and PhotoMode Story 9.3)
**When** the control system runs
**Then**:
- Left mouse drag → yaw/pitch around anchor
- Mouse wheel → distance (dolly)
- WASD → pan_offset translation in camera-local axes
- Space / LCtrl → pan_offset up/down

**Given** both gates (F3 dev + PhotoMode)
**When** either activates
**Then** they reuse the same component + control system (one impl, two gates)

## Story 9.2: PhotoMode State + Entry/Exit from PostRun + Time-Freeze + Overlay

As a player,
I want a PhotoMode state I enter from PostRun, with time frozen, a "PHOTO MODE" badge, and a controls hint,
So that FR40's post-run-only access is real and the workflow is discoverable.

**Acceptance Criteria:**

**Given** `GameState` enum from Story 1.6
**When** extended
**Then** `PhotoMode` variant is fully realized (was a placeholder in Story 1.6)

**Given** Epic 4 Story 4.9's PostRun layout
**When** Story 9.2 extends it
**Then** a "Photo Mode" button is added alongside Retry / Main Menu / Shop
**And** clicking it sets `NextState<GameState>` = `PhotoMode`

**Given** FR40 constraint (post-run only, not during gameplay)
**When** the constraint is audited
**Then** NO keybinding or UI affordance routes to PhotoMode from Arena or Caravan
**And** a `#[cfg(debug_assertions)]` assertion can verify this invariant on state-entry

**Given** OnEnter(GameState::PhotoMode)
**When** state entry runs
**Then** a FreeOrbitCamera spawns with `anchor_point = PlayerShip position, distance = 20.0, yaw = 0.0, pitch = -0.3` (death-pose per Till's decision)
**And** CockpitCamera (Story 3.5) is disabled (`Camera::active = false`)
**And** time is frozen per Story 3.4's pause mechanism (reused)
**And** all audio is muted (SfxChannel + AlertChannel silenced per Till's decision)

**Given** the PhotoMode overlay UI
**When** it renders on entry
**Then** a `bevy_ui` Node shows:
- "PHOTO MODE" badge (top-left corner, small semi-transparent text)
- Controls hint: "Drag = rotate · Wheel = zoom · WASD = pan · F = focus · E = export · Esc = back"

**And** entities carry `PhotoModeEntity` marker

**Given** the player presses Esc in PhotoMode
**When** the exit system runs
**Then** `NextState<GameState>` = `PostRun` (returns, allowing further Retry/Menu/Shop)

**Given** OnExit(GameState::PhotoMode)
**When** cleanup runs
**Then** FreeOrbitCamera is despawned
**And** CockpitCamera re-enables
**And** time-freeze lifts (inverse of Story 3.4 pause)
**And** audio channels resume
**And** `PhotoModeEntity`-marked entities are despawned

**Given** the player re-enters PhotoMode from PostRun multiple times per Till's decision
**When** each entry happens
**Then** a fresh FreeOrbitCamera spawns each time (no stale cross-entry state)
**And** multiple exports (Story 9.5) within a single entry are supported without forced exit

## Story 9.3: PhotoMode Free-Cam Orbital + Dolly Controls

As a player in PhotoMode,
I want to orbit, zoom, and pan the camera freely,
So that I can frame any angle per FR41.

**Acceptance Criteria:**

**Given** Story 9.1's FreeOrbitCamera controls exist
**When** Story 9.3 scopes them to PhotoMode
**Then** the control system is gated to `GameState == PhotoMode`
**And** input is consumed by the camera, not by any gameplay system (gameplay is time-frozen anyway)

**Given** standard orbital controls (shared mappings with dev F3)
**When** input arrives
**Then**:
- Left mouse drag → `yaw` + `pitch` (orbit around anchor)
- Mouse wheel → `distance` (clamped to `TuningConfig.photo_min_distance = 2.0`, `photo_max_distance = 200.0`)
- WASD → `pan_offset` translation in camera-local XZ
- Space / LCtrl → `pan_offset` translation in camera-local Y

**And** mouse sensitivity reuses `mouse_sensitivity` from TuningConfig for consistency with flight

**Given** pitch extremes
**When** the player drags pitch past ±89°
**Then** pitch is clamped to [-89°, +89°] (prevents gimbal lock / flip)

**Given** Tilt/Roll is Post-MVP per Till 2026-04-22
**When** input bindings are inspected
**Then** roll input is NOT bound in MVP — camera stays world-up-aligned
**And** a source comment notes "tilt/roll deferred to Post-MVP / Epic 10 Polish"

**Given** the player is in PhotoMode
**When** mouse movement occurs
**Then** cursor is NOT confined (visible + free) so the player can click UI buttons (focus, export, watermark, back)
**And** drag detection uses mouse-button-hold semantics (click-to-start-drag, release-to-end)

## Story 9.4: Depth-of-Field Post-Processing + Click-to-Focus

As a player,
I want adjustable depth-of-field with click-to-focus,
So that I can compose cinematic shots per FR41.

**Acceptance Criteria:**

**Given** a DoF post-processing node is added to the PhotoMode rendering pipeline
**When** PhotoMode is active
**Then** the node is enabled — either Bevy 0.18's built-in DoF (if available) or a custom Gaussian-blur-based node at `src/camera/photo_dof.rs`
**And** outside PhotoMode, DoF is disabled (zero rendering cost in gameplay)

**Given** DoF state
**When** `PhotoDofState { focus_distance: f32, bokeh_intensity: f32, enabled: bool }` resource is defined
**Then** OnEnter(PhotoMode) initializes it to `focus_distance = 20.0, bokeh_intensity = 0.5, enabled = true`

**Given** DoF UI controls are shown in the overlay
**When** the UI renders
**Then** two sliders:
- "Focus distance" (0.5–200.0 m) → updates `focus_distance`
- "Bokeh" (0.0–1.0) → updates `bokeh_intensity`

**And** a checkbox toggles `enabled` (completely off for sharp-focus shots)

**Given** click-to-focus per Till 2026-04-22
**When** the player presses `F` key
**Then** a raycast from camera through cursor position is performed
**And** if it hits any entity, `focus_distance` is set to the hit distance
**And** the slider UI updates reactively

**Given** DoF is actively blurring
**When** the frame renders
**Then** pixels at `focus_distance` ± small tolerance are sharp
**And** further pixels are progressively blurred proportional to `bokeh_intensity`
**And** no flicker / no crash; minor banding acceptable

**Given** PhotoMode exits
**When** DoF node deactivates
**Then** the resource remains in-memory (next PhotoMode entry starts with last-used values, soft state persistence within app session)
**And** gameplay rendering is unchanged

## Story 9.5: PNG Export — 16:9 / 9:16 / 1:1 Aspect Ratio Presets

As a player,
I want to export the current PhotoMode view as a PNG in stream-friendly aspect ratios,
So that FR42 is functional.

**Acceptance Criteria:**

**Given** `src/camera/photo_export.rs` is authored
**When** the export system is registered
**Then** three aspect-ratio presets at fixed resolutions per Till 2026-04-22:
- 16:9 landscape → 1920×1080
- 9:16 portrait → 1080×1920
- 1:1 square → 1080×1080

**(4K / variable-resolution deferred to Epic 10 Polish)**

**Given** PhotoMode overlay UI
**When** rendered
**Then** three export buttons are shown: "Export 16:9", "Export 9:16", "Export 1:1"
**And** the `E` keyboard shortcut triggers a quick 16:9 export (default) — user can click buttons for other ratios

**Given** the player triggers an export
**When** the export system runs
**Then** a render-to-texture pass renders the scene at the preset resolution (temporary camera + Bevy render-target)
**And** the PhotoMode overlay UI is excluded from capture (overlay hidden for the render frame, restored immediately after)
**And** the texture is encoded to PNG via the `image` crate (or Bevy's screenshot primitives if available in 0.18)

**Given** export file location per Till 2026-04-22
**When** the PNG is written
**Then** the file goes to:
- Unix/macOS: `~/Pictures/asteroids3D/screenshots/`
- Windows: `%USERPROFILE%\Pictures\asteroids3D\screenshots\`

**And** the filename is `asteroids3D-<YYYYMMDD>-<HHMMSS>-<ratio>.png` (e.g., `asteroids3D-20260815-143022-16x9.png`)
**And** the directory is created if absent
**And** `directories` crate (already a dependency from Story 4.6) resolves the Pictures dir per-OS

**Given** the export succeeds
**When** the UI is notified
**Then** a toast-style notification appears in the overlay: "Exported: <filename>" (visible 3 s)
**And** the player remains in PhotoMode (no state change) for further exports

**Given** the export fails (disk full, permission error)
**When** the error is caught
**Then** a `warn!` log is written
**And** a toast shows "Export failed: <reason>"
**And** PhotoMode persists without crash

**Given** multiple exports per PhotoMode session per Till 2026-04-22
**When** the player exports repeatedly
**Then** each creates a new PNG with a distinct timestamp
**And** no forced state-exit after export

## Story 9.6: Toggleable Watermark

As a player,
I want an optional "asteroids3D" watermark I can toggle on for credited screenshots,
So that exported PNGs can be identified as from this game.

**Acceptance Criteria:**

**Given** SaveData
**When** Story 9.6 extends it
**Then** `watermark_enabled: bool` (default `false`) is added
**And** SaveData.version bumps to v5 with a migration injecting `false` via Story 5.6's scaffold

**Given** PhotoMode overlay (Story 9.2)
**When** rendered
**Then** a checkbox "Watermark on export" is shown alongside DoF controls
**And** its value binds to `SaveData.watermark_enabled`
**And** toggling updates SaveData and calls `save(&save_data)` immediately (persists across sessions)

**Given** Story 9.5's export system
**When** rendering the final PNG
**Then** if `watermark_enabled == true`:
- Small "asteroids3D" text overlay in the bottom-right corner
- Text is semi-transparent (~70% opacity) with a subtle drop-shadow for readability on any background
- Neutral color (light gray / off-white), not palette-colored

**And** if `watermark_enabled == false`:
- No watermark — pure scene render

**Given** the player toggles the watermark multiple times and exports
**When** each export runs
**Then** each PNG reflects the watermark state at its export time
**And** previously-exported PNGs are not retroactively modified

**Given** watermark text rendering approach
**When** implementation is chosen
**Then** either bevy_ui into a render-to-texture step, OR overlay at image-encode step via the `image` crate — either acceptable; pragmatic choice at implementation time

<!-- Epic 9 complete — 6 stories deliver M8 Post-Run Photo Mode (PNG export pipeline). SaveData v4→v5 (watermark_enabled). Next epic to decompose: Epic 10 (Polish Pass & MVP Completion / M9). -->
