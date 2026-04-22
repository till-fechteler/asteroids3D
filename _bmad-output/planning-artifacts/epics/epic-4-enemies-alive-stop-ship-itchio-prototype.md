# Epic 4: Enemies Alive & Stop-Ship (Itch.io Prototype)

The Itch.io-shippable small game. Full combat loop: 3 weapons, 1 enemy with AI, permadeath on Hull-zero, post-run summary, immediate restart, title screen, settings (volume + sensitivity), persistent settings, unsigned macOS binary (signing deferred to E7), release workflow for per-OS ZIPs. The M3 stop-and-ship waypoint. M-alignment: M3 🏁. FRs covered: FR10, FR14, FR16, FR36, FR37, FR38, FR39, FR44, FR45, FR46, FR47, FR50 (FR48 → E7).

## Story 4.1: Enemy Entity Foundation + SemanticAccent::Enemy

As a player,
I want a visible enemy ship present in the Arena when I spawn,
So that I have a new target type to recognize before AI and combat dynamics come online in Story 4.2.

**Acceptance Criteria:**

**Given** `src/combat/enemy.rs` is authored
**When** `Enemy` and `EnemyShip` components are defined and integrated with `CombatPlugin`
**Then** `Enemy` is an empty marker component and `EnemyShip` optionally holds archetype data (currently a single variant)

**Given** `OnEnter(GameState::Arena)` runs after Story 3.3's zone spawn
**When** an enemy-spawn system runs
**Then** exactly one `EnemyShip` entity is spawned at a hand-picked `Transform` within ~80 m of the PlayerShip
**And** the enemy uses a placeholder mesh (distinct from PlayerShip) rendered with `ToonMaterial` + `SemanticAccent::Enemy` + `OutlineBundle`
**And** the enemy is an Avian `RigidBody::Dynamic` with a `Collider` sized to its mesh
**And** the enemy carries the `ArenaEntity` marker

**Given** the enemy is in the Arena
**When** the player approaches within line-of-sight
**Then** the enemy's red accent tint is visibly distinct from salvage (yellow) asteroids and the PlayerShip
**And** the enemy is stationary (AI is Story 4.2)

**Given** the Arena state exits
**When** `OnExit(GameState::Arena)` runs
**Then** the enemy is despawned via `cleanup_on_exit::<ArenaEntity>`

## Story 4.2: Enemy AI State Machine — Detect / Pursue / Attack

As a player,
I want the enemy to detect, pursue, and fire on me when in range,
So that the Arena has a live adversary per FR14 and combat has stakes.

**Acceptance Criteria:**

**Given** a unified `Health { current: u32, max: u32 }` component is introduced in `src/combat/health.rs`
**When** Story 3.10's `AsteroidHp` is refactored
**Then** asteroids now carry `Health { current: 1, max: 1 }` and all prior `AsteroidHp` references use `Health`
**And** the enemy is spawned with `Health { current: 2, max: 2 }` (2-shot enemy for first playable)

**Given** `src/combat/enemy_ai.rs` defines `EnemyAiState`
**When** the enum is declared
**Then** it includes variants `Idle`, `Detect`, `Pursue`, `Attack`
**And** each `EnemyShip` carries `EnemyAiState` (default `Idle`)

**Given** `TuningConfig` is extended with `enemy_detection_range: f32 = 100.0`, `enemy_engagement_range: f32 = 50.0`, `enemy_speed: f32 = 20.0`, `enemy_fire_rate_hz: f32 = 1.0`, `enemy_ai_hysteresis_pct: f32 = 0.1`
**When** an AI-transition system runs in `FixedUpdate` inside `CombatSystems`
**Then** the enemy transitions by distance-to-PlayerShip:
- `distance > detection_range` → `Idle`
- `detection_range ≥ distance > engagement_range` → `Detect` (rotates to face player)
- `engagement_range ≥ distance > engagement_range * 0.5` → `Pursue` (moves toward player)
- `distance ≤ engagement_range * 0.5` → `Attack` (stops, fires)

**And** each transition threshold applies `enemy_ai_hysteresis_pct` as a dead-band to prevent state flicker at boundaries

**Given** the enemy is in `Pursue`
**When** the movement system runs
**Then** `ExternalForce` is applied toward the PlayerShip position
**And** the enemy orients to face the PlayerShip
**And** speed is clamped to `enemy_speed`

**Given** the enemy is in `Attack`
**When** `enemy_fire_rate_hz` cooldown elapses
**Then** a projectile is spawned toward the PlayerShip using the same `Projectile` component as Story 3.9
**And** the projectile carries an `EnemyProjectile` marker so collision filtering distinguishes player vs enemy projectiles

**Given** a player projectile hits the enemy
**When** the damage system runs
**Then** the enemy's `Health.current` decreases by projectile damage
**And** at `Health.current == 0` an `EnemyDestroyed { enemy: Entity }` event is emitted, the enemy is despawned, and no respawn occurs (single-enemy encounter per Arena entry in Epic 4)

## Story 4.3: Hull Component + Permadeath → PostRun State

As a player,
I want my ship to take damage from enemy projectiles and the run to end on hull-zero,
So that permadeath per FR16 is real and combat has stakes.

**Acceptance Criteria:**

**Given** the unified `Health` component from Story 4.2
**When** PlayerShip is extended
**Then** it carries `Health { current: 3, max: 3 }` sourced from `TuningConfig.player_hull_max: u32 = 3`
**And** no regen applies in Epic 4 (regen is Epic 5)

**Given** enemy projectiles carrying `EnemyProjectile` from Story 4.2
**When** an enemy projectile collides with the PlayerShip
**Then** a `ProjectileHitPlayer { projectile: Entity, player: Entity, damage: u32 }` event is emitted
**And** the projectile is despawned
**And** `Health.current = max(0, current - damage)` on the PlayerShip

**Given** `HudPlaceholder::Hull` from Story 3.11
**When** PlayerShip `Health.current` changes
**Then** the HUD Hull field updates live to show the current value (first-playable real HUD wiring for Hull only; Shields stays placeholder until Epic 5)

**Given** PlayerShip Health reaches 0
**When** the permadeath system runs
**Then** a `HullDepleted { player: Entity, cause: DeathCause }` event is emitted
**And** `DeathCause` is an enum with variants `EnemyFire`, `AsteroidCollision`, `Unknown`
**And** a `RunResult { cause: DeathCause, salvage_banked: u32, run_duration_seconds: f32 }` resource is inserted (consumed by Story 4.9)
**And** `NextState<GameState>` is set to `PostRun`

**Given** the Arena → PostRun transition
**When** `OnExit(GameState::Arena)` runs
**Then** all `ArenaEntity`-marked entities are despawned per existing cleanup pattern

## Story 4.4: Weapon Archetype System + Shotgun / Railgun Archetypes

As a player,
I want three distinct weapon archetypes equipped in slots I can cycle,
So that I have tactical weapon choices per FR10.

**Acceptance Criteria:**

**Given** `src/combat/weapons.rs` is authored
**When** `WeaponArchetype` enum is defined
**Then** it includes `Pulse`, `Shotgun`, `Railgun`
**And** stats are loaded from `TuningConfig`:
- `Pulse`: damage=1, fire_rate=4.0 Hz, speed=120, projectiles=1, spread=0°
- `Shotgun`: damage=1, fire_rate=1.5 Hz, speed=80, projectiles=5, spread=15°
- `Railgun`: damage=5, fire_rate=0.5 Hz, speed=300, projectiles=1, spread=0°

**Given** PlayerShip is extended with a `WeaponLoadout { slots: [Option<WeaponArchetype>; 3], active_slot: usize }` component
**When** the PlayerShip spawns in Arena
**Then** the default loadout is `[Some(Pulse), Some(Shotgun), Some(Railgun)]` with `active_slot = 0`
**And** `CombatAction` is extended with `CycleWeapon` (default binding: Q, or 1/2/3 direct-select)

**Given** the player presses `CycleWeapon`
**When** the input system runs
**Then** `active_slot` advances cyclically (0 → 1 → 2 → 0)
**And** an `info!` log records the new active archetype

**Given** the player holds `FirePrimary` with an active archetype
**When** the fire system runs
**Then** projectiles spawn per archetype stats:
- Shotgun spawns 5 projectiles simultaneously in a cone within ±spread° around ship-forward
- Railgun spawns 1 projectile with high speed and high damage but a long cooldown
- Pulse behaviour equals the Story 3.9 default

**Given** each archetype
**When** projectiles spawn
**Then** shotgun / pulse / railgun projectiles are visually distinguishable (e.g., different mesh scales) — placeholder-grade, final visuals are Epic 10 polish

## Story 4.5: SemanticAccent Wiring — Asteroids=Salvage, PlayerShip+Projectiles=PlayerOwned

As a player,
I want the semantic accent palette visibly distinguishing salvage, enemies, and my own ship,
So that I can identify entity categories at a glance per FR50 and NFR-A1 redundant encoding.

**Acceptance Criteria:**

**Given** Story 3.3's asteroid spawner uses `SemanticAccent::Neutral`
**When** updated
**Then** asteroids use `SemanticAccent::Salvage` (yellow)

**Given** Story 3.5's PlayerShip spawner
**When** updated
**Then** PlayerShip uses `SemanticAccent::PlayerOwned` (cyan)

**Given** Story 3.9's player-projectile spawner
**When** updated
**Then** player projectiles use `SemanticAccent::PlayerOwned`
**And** Story 4.2's enemy projectiles use `SemanticAccent::Enemy`

**Given** all accent wiring applies
**When** the Arena is rendered with all entity types simultaneously visible
**Then** four accent tints (Salvage yellow / Enemy red / PlayerOwned cyan / any remaining Neutral) are simultaneously distinguishable under toon shading

## Story 4.6: PersistencePlugin + Save Schema v1

As a developer,
I want a `PersistencePlugin` that reads and atomically writes a versioned save at the OS-convention path,
So that settings, meta-currency, and future unlocks share one durable, crash-safe persistence mechanism per FR44/FR45/FR46.

**Acceptance Criteria:**

**Given** `src/persistence/mod.rs` is authored
**When** `PersistencePlugin` is added
**Then** the plugin declares `PersistenceSystems` SystemSet

**Given** `src/persistence/save_data.rs` defines the schema
**When** `SaveData` struct is declared with Serde derives
**Then** fields are: `version: u32 = 1`, `settings: Settings { master_volume: f32, sfx_volume: f32, mouse_sensitivity: f32 }`, `meta_currency: u32 = 0` (Epic 7 placeholder), `unlocked_upgrades: Vec<String> = []` (Epic 7 placeholder)

**Given** the `directories` crate resolves the per-OS data dir
**When** `save_path()` is called
**Then** it returns:
- Windows `%APPDATA%\asteriods3D\save.json`
- Linux `$XDG_DATA_HOME/asteriods3D/save.json` (or `~/.local/share/asteriods3D/save.json` fallback)
- macOS `~/Library/Application Support/asteriods3D/save.json`

**And** the directory is created if absent

**Given** `save(data: &SaveData) -> Result<(), SaveError>`
**When** called
**Then** `save.json.tmp` is written, fsync'd, then atomically renamed to `save.json` via `std::fs::rename` on Unix or the Windows-equivalent atomic-rename API
**And** `SaveError` is a `thiserror`-derived enum

**Given** `load() -> Result<SaveData, LoadError>`
**When** called and the file exists
**Then** JSON is parsed and returned
**And** schema-version mismatch returns `LoadError::VersionMismatch { found, expected }` (full migration path is Epic 5)

**Given** no save file exists on first launch
**When** `OnEnter(GameState::Loading)` runs and `load()` returns `FileNotFound`
**Then** `save(&SaveData::default())` creates a default save
**And** on subsequent launches the default is readable

**Given** load succeeds
**When** the app initializes
**Then** `SaveData` is inserted as a `Resource`
**And** settings are applied to audio + `TuningConfig.mouse_sensitivity`

**Given** a save write interrupted mid-flight (kill -9)
**When** the app relaunches
**Then** either the new save or the prior save is valid (never partial)
**And** if both are corrupt, `LoadError::Corrupt` surfaces, a warn! is logged, and a default save is written

## Story 4.7: Title Screen — Full FR36 (Start / Settings / Credits / Quit)

As a player opening the game,
I want a proper title screen with Start, Settings, Credits, and Quit,
So that I have a front-door menu per FR36, replacing the Story 3.1 stub.

**Acceptance Criteria:**

**Given** Story 3.1's stub title screen
**When** Story 4.7 is implemented
**Then** the stub is replaced with `src/ui/title.rs` under a `TitleScreenPlugin`
**And** `OnEnter(GameState::MainMenu)` spawns four `bevy_ui` buttons: `Start Run`, `Settings`, `Credits`, `Quit`
**And** each button has hover and press styling and carries a `MainMenuEntity` marker

**Given** the player selects a button via mouse click or keyboard (arrow keys + Enter)
**When** the action fires
**Then** the corresponding transition runs:
- `Start Run` → `NextState<GameState>` to `Arena`
- `Settings` → `NextState<GameState>` to `Settings`
- `Credits` → `NextState<GameState>` to `Credits`
- `Quit` → emits `AppExit::Success`

**Given** `GameState` enum from Story 1.6
**When** Story 4.7 is merged
**Then** `Settings` and `Credits` variants are added

**Given** `OnEnter(GameState::Credits)` runs
**When** the credits screen is built
**Then** a static text Node shows: title, `env!("CARGO_PKG_VERSION")`, "by Till Fechteler, 2026", key plugin credits (Bevy, Avian, bevy_mod_outline, bevy_kira_audio, leafwing-input-manager), and "Press Esc or Backspace to return"
**And** Esc or Backspace returns to `MainMenu`

## Story 4.8: Settings Menu (Master / SFX Volume + Mouse Sensitivity)

As a player,
I want a Settings menu with sliders for master volume, SFX volume, and mouse sensitivity that persist across launches,
So that I can customize audio and feel per FR37.

**Acceptance Criteria:**

**Given** `src/ui/settings.rs` is authored
**When** `OnEnter(GameState::Settings)` runs
**Then** three slider UI elements are spawned for `master_volume` (0.0–1.0), `sfx_volume` (0.0–1.0), `mouse_sensitivity` (0.1–3.0)
**And** initial values are read from the `SaveData` Resource's `settings` field
**And** a "Back" button and `SettingsEntity` marker are present
**And** slider control is mouse-only in Epic 4 (keyboard nav is Epic 10 polish)

**Given** the player drags a slider
**When** the value changes
**Then** `SaveData.settings` is updated in-memory
**And** the change applies live:
- `master_volume` → `bevy_kira_audio` master mixer
- `sfx_volume` → SFX channel mixer
- `mouse_sensitivity` → `TuningConfig.mouse_sensitivity`

**Given** the player clicks Back or presses Esc
**When** the transition triggers
**Then** `save(&save_data)` is called
**And** `NextState<GameState>` returns to `MainMenu`
**And** on next launch, values are restored

**Given** a save-write error (e.g., disk full)
**When** the Back button attempts to save
**Then** the error is `warn!`-logged, the transition still completes (no modal error — Principle 4 death-as-feedback extends to non-critical errors)

## Story 4.9: Post-Run Summary Screen (Retry / Main Menu)

As a player,
I want a post-run summary showing why my run ended, with options to retry or return to menu,
So that I receive feedback on death per FR38 and re-engage quickly per FR39 — without any "GAME OVER" defeat frame per Design Principle 4.

**Acceptance Criteria:**

**Given** `src/ui/post_run.rs` is authored with a `PostRunPlugin`
**When** `OnEnter(GameState::PostRun)` runs
**Then** a summary Node is spawned showing:
- Header: "Run ended" (neutral — no "GAME OVER", no "YOU DIED", no red overlay)
- Cause-of-death text from `RunResult.cause` (e.g., "Hull depleted by enemy fire")
- "Salvage banked: 0" (placeholder for Epic 6 economy)
- Two buttons: `Retry` and `Main Menu`

**And** all spawned entities carry a `PostRunEntity` marker

**Given** the player clicks `Retry`
**When** the action fires
**Then** `NextState<GameState>` is set to `Arena` directly (does not route through `MainMenu`)

**Given** the player clicks `Main Menu`
**When** the action fires
**Then** `NextState<GameState>` is set to `MainMenu`

**Given** Design Principle 4 compliance is reviewed
**When** PostRun is audited
**Then** no "GAME OVER" string exists anywhere, no red-tinted overlay, no defeat music crossfade, no death-specific screen shake
**And** ambient audio (once added in Epic 8) continues seamlessly across Arena → PostRun

**Given** the state transitions `PostRun → Arena` or `PostRun → MainMenu`
**When** `OnExit(GameState::PostRun)` runs
**Then** all `PostRunEntity`-marked entities are despawned

## Story 4.10: Cross-Platform Release Workflow — per-OS ZIPs

As the project author,
I want a GitHub Actions release workflow that builds per-OS release binaries and ZIPs them on tag push,
So that M3 Itch.io shipping has reproducible artifacts per FR47 with minimal manual overhead.

**Acceptance Criteria:**

**Given** `.github/workflows/release.yml` is authored
**When** a git tag matching `v*` is pushed
**Then** three parallel jobs run on `windows-latest`, `ubuntu-latest`, `macos-latest` (Apple Silicon arm64)

**Given** each job executes
**When** it runs
**Then** it:
- Runs `cargo build --release`
- Stages the binary + full `assets/` directory into `asteriods3D-<os>-<version>/`
- ZIPs to `asteriods3D-{windows-x64|linux-x64|macos-arm64}-<version>.zip`
- Uploads the ZIP as a GitHub Actions artifact

**And** the macOS build is **unsigned** per Till's decision on 2026-04-22 (signing deferred to Epic 7 / M6)
**And** Intel x86_64 macOS support is explicitly deferred to Epic 7 / M6 — M3 ships macOS arm64 only

**Given** all three jobs pass
**When** the workflow completes
**Then** a GitHub draft Release is created attached to the tag with all three ZIPs as assets

**Given** the draft Release exists
**When** Till manually reviews
**Then** the release-process runbook `docs/release-process.md` documents: (1) version bump in Cargo.toml, (2) `git tag v<version>`, (3) `git push --tags`, (4) wait for CI, (5) download artifacts, (6) smoke-test ZIP launches locally on at least one platform, (7) upload to Itch.io via web UI (or `butler push` CLI — optional), (8) publish GitHub Release
**And** the runbook flags that macOS users must right-click → Open on first launch to bypass Gatekeeper (unsigned build)

<!-- Epic 4 complete — 10 stories deliver M3 stop-and-ship (Itch.io prototype). FR48 deferred to Epic 7. Next epic to decompose: Epic 5 (Ship Subsystem State & Formal Save Schema / M4). -->
