# Epic 3: Arena Flight & First Combat (First Playable)

Player flies a cockpit ship in the Arena, fires a prefab weapon, destroys asteroids, sees HUD with ship state. No enemies yet. Diegetic learning — no tutorial text (FR28 upheld as design constraint across all stories; not its own story). M-alignment: M2. FRs covered: FR1, FR2, FR3, FR5, FR8, FR9, FR12, FR24, FR27, FR43.

## Story 3.1: Title Screen Stub — MainMenu → Arena Transition

As a player launching the game,
I want a minimal title screen that lets me start a run with a single key press,
So that I reach gameplay from the first Epic-3 commit without dev hacks like default-to-Arena.

**Acceptance Criteria:**

**Given** the app is in `GameState::MainMenu` after Epic 1 Story 1.7 transition
**When** `OnEnter(GameState::MainMenu)` runs
**Then** a `bevy_ui` text Node is spawned with title "asteroids3D" plus subtitle "Press Enter to start"
**And** all spawned entities carry a `MainMenuEntity` marker component

**Given** the title screen is visible
**When** the player presses Enter / Return
**Then** `NextState<GameState>` is set to `Arena`

**Given** the state transitions `MainMenu → Arena`
**When** `OnExit(GameState::MainMenu)` runs
**Then** all `MainMenuEntity`-marked entities are despawned
**And** no orphaned title or subtitle text remains

Note: Stub. Full FR36 title screen (start / settings / credits / quit) is Epic 4.

## Story 3.2: Avian Physics Foundation + Arena State Skeleton

As a developer,
I want Avian XPBD registered in `FixedUpdate` at 60 Hz with gravity disabled, plus a `GameState::Arena` skeleton with OnEnter/OnExit hooks,
So that subsequent flight and combat stories attach to a deterministic physics world and a clean state-lifecycle.

**Acceptance Criteria:**

**Given** `avian3d` is added via `App::add_plugins(PhysicsPlugins::default())`
**When** the app runs
**Then** the physics schedule ticks at 60 Hz inside `FixedUpdate` per the architecture decision
**And** `Gravity(Vec3::ZERO)` is inserted as a Resource (zero-g space environment)

**Given** `src/arena/mod.rs` is authored
**When** `ArenaPlugin: Plugin` is added via `App::add_plugins`
**Then** the plugin declares an `ArenaSystems` SystemSet enum
**And** an `ArenaEntity` marker component is defined
**And** a `cleanup_on_exit::<ArenaEntity>` system is registered on `OnExit(GameState::Arena)`

**Given** the Arena state is entered from Story 3.1's trigger
**When** `OnEnter(GameState::Arena)` runs
**Then** an `info!` log `"entered Arena"` is emitted
**And** the scene is otherwise empty (zone content is Story 3.3)

**Given** the player transitions `Arena → MainMenu` (triggered later in Epic 4)
**When** `OnExit(GameState::Arena)` runs
**Then** every entity carrying `ArenaEntity` is despawned before the next state enters

## Story 3.3: Hand-Designed Arena Zone with Static Asteroid Field

As a player,
I want a hand-designed Arena zone with a visible asteroid field when I start a run,
So that I enter a perceivable 3D space with navigational reference points, not a void.

**Acceptance Criteria:**

**Given** `src/arena/zone.rs` is authored with a `spawn_arena_zone` system on `OnEnter(GameState::Arena)`
**When** the system runs
**Then** 15–25 asteroid entities are spawned at hand-picked `Transform` positions covering a roughly 200×200×200 m volume
**And** each asteroid uses a placeholder icosphere `Mesh3d` with radius 3.0–12.0 m
**And** each asteroid uses `ToonMaterial` from Epic 2 with `SemanticAccent::Neutral`
**And** each asteroid is an Avian `RigidBody::Static` with a `Collider::sphere` sized to its mesh radius
**And** each asteroid carries the `ArenaEntity` marker

**Given** toon shading needs a directional light for readable posterization
**When** `OnEnter(GameState::Arena)` runs
**Then** exactly one `DirectionalLight` is spawned with the `ArenaEntity` marker
**And** no ambient light beyond Bevy defaults (dark-space aesthetic preserved)

**Given** the Arena state exits
**When** `OnExit` runs
**Then** all asteroids and the `DirectionalLight` are despawned via `cleanup_on_exit::<ArenaEntity>`

## Story 3.4: Pause on Focus Loss + Pause Menu Stub

As a player,
I want the game to pause when I Alt-Tab away or press Escape,
So that the simulation never advances while I'm not looking and I never take invisible damage.

**Acceptance Criteria:**

**Given** `src/pause/mod.rs` is authored with a `PausePlugin`
**When** a `WindowFocused { focused: false, .. }` event arrives while `GameState` is `Arena` (or any in-gameplay state)
**Then** `NextState<GameState>` is set to `Paused`
**And** the physics simulation is paused per the applicable Bevy-0.18 / Avian-0.6 convention (e.g., `Time::<Virtual>::pause()` or the Avian-exposed physics-time control — resolved concretely at implementation time)

**Given** the app is in `GameState::Paused` via focus-loss
**When** `WindowFocused { focused: true, .. }` arrives
**Then** `NextState<GameState>` returns to the state stored in a `PausedFrom(GameState)` resource captured on pause entry
**And** the physics simulation resumes via the inverse of the pause convention

**Given** the app is in `GameState::Arena`
**When** the player presses Escape
**Then** `NextState<GameState>` is set to `Paused`
**And** a `bevy_ui` text Node "PAUSED — Esc to resume" is spawned with a `PauseOverlayEntity` marker

**Given** the app is in `GameState::Paused` via Escape
**When** the player presses Escape again
**Then** `NextState<GameState>` returns to `PausedFrom`
**And** `PauseOverlayEntity`-marked entities are despawned on `OnExit(GameState::Paused)`

## Story 3.5: Cockpit Camera + PlayerShip Entity

As a player,
I want a first-person cockpit camera attached to a visible ship placed in the Arena,
So that I see the game world from inside the ship — the FR8 cockpit-only commitment from frame one.

**Acceptance Criteria:**

**Given** `src/flight/mod.rs` is authored with a `FlightPlugin`
**When** `OnEnter(GameState::Arena)` runs (after Story 3.3's zone spawn)
**Then** exactly one `PlayerShip` entity is spawned with the `ArenaEntity` marker
**And** the `PlayerShip` has a placeholder cockpit mesh (cuboid or simple fighter silhouette) rendered with `ToonMaterial`
**And** the `PlayerShip` is an Avian `RigidBody::Dynamic` (gravity inherited zero from the world)
**And** the `PlayerShip` has a sphere or capsule `Collider` sized to the placeholder mesh

**Given** a cockpit camera is attached as a child
**When** the `PlayerShip` hierarchy is inspected
**Then** exactly one `Camera3d` entity is a child of `PlayerShip`
**And** the camera's local `Transform` places it at the pilot-seat wingtip-framing position (slightly behind and above the mesh origin, angled slightly downward)
**And** the camera carries a `CockpitCamera` marker component

**Given** the `PlayerShip` spawns in Arena
**When** the initial `Transform` is chosen relative to Story 3.3's zone layout
**Then** the ship has line of sight to at least 3 asteroids within 50 m
**And** the ship has zero initial linear and angular velocity (no drift at spawn)

**Given** no flight-input systems exist yet
**When** the game enters Arena
**Then** the player sees the asteroid field from the cockpit, stationary

## Story 3.6: Flight Input → 6-DOF Translation

As a player,
I want keyboard input to translate my ship forward, reverse, strafe left/right, and up/down in ship-local space,
So that I can navigate the Arena in 3D per FR2.

**Acceptance Criteria:**

**Given** `leafwing-input-manager` is registered in `FlightPlugin`
**When** `src/flight/input.rs` defines a `FlightAction` enum
**Then** it includes variants `ThrustForward`, `ThrustReverse`, `StrafeLeft`, `StrafeRight`, `ThrustUp`, `ThrustDown`
**And** default bindings are W/S (forward/reverse), A/D (strafe), Space/LCtrl (up/down)
**And** the `PlayerShip` from Story 3.5 is spawned with an `InputManagerBundle<FlightAction>` using the defaults

**Given** `TuningConfig` is extended with `ship_thrust_newtons: f32` (default 500.0)
**When** a thrust-application system runs in `FixedUpdate` inside `FlightSystems`
**Then** for each pressed `FlightAction` variant, an `ExternalForce` is applied in the ship-local direction (+Z forward, -Z reverse, ±X strafe, ±Y up/down) at magnitude `ship_thrust_newtons`

**Given** the player presses `ThrustForward` in Arena
**When** 2 seconds elapse
**Then** the ship's world linear velocity is approximately `(ship_thrust_newtons / mass) * 2` m/s forward within 10% integration tolerance
**And** the ship continues to drift after release (pure Newtonian — dampener is Story 3.8)

**Given** `ThrustForward` + `StrafeRight` are pressed simultaneously
**When** the composite force is inspected
**Then** the forces sum (diagonal motion in ship-local space)
**And** ship rotation is unchanged (translation does not induce rotation)

## Story 3.7: Flight Input → 3-Axis Rotation (Pitch / Yaw / Roll)

As a player,
I want mouse input to pitch and yaw my ship, plus Q/E to roll,
So that I can aim the ship freely per FR3.

**Acceptance Criteria:**

**Given** `FlightAction` is extended
**When** new variants are added
**Then** they include `Pitch` (DualAxis), `Yaw` (DualAxis), `RollLeft`, `RollRight`
**And** default bindings are Mouse Y → Pitch, Mouse X → Yaw, Q → RollLeft, E → RollRight

**Given** `TuningConfig` is extended with `mouse_sensitivity: f32` (default 1.0) and `ship_torque_nm: f32` (default 80.0)
**When** a rotation system runs in `FixedUpdate` inside `FlightSystems`
**Then** mouse delta values scaled by `mouse_sensitivity` are applied as `ExternalTorque` around ship-local pitch and yaw axes
**And** `RollLeft` / `RollRight` apply constant `±ship_torque_nm` around the ship-local roll axis

**Given** the player moves the mouse up
**When** the frame advances
**Then** the ship pitches up relative to its current roll orientation (pitch is ship-local, not world-up)

**Given** the player presses Q for 1 second
**When** the action ends
**Then** the ship has rotated approximately `ship_torque_nm / moment_of_inertia` radians around its local +Z axis
**And** angular velocity persists (no dampener yet)

**Given** the Arena state is active
**When** the mouse cursor is inspected
**Then** the cursor is confined (`CursorGrabMode::Confined`) and hidden (`visible: false`) so mouse motion maps cleanly to rotation

## Story 3.8: Inertial Dampener Toggle

As a player,
I want a toggleable inertial dampener that bleeds my linear and angular velocity toward zero when active,
So that I can modulate between Newtonian drift and arcade-tight control per FR5.

**Acceptance Criteria:**

**Given** `FlightAction` is extended
**When** a new variant is added
**Then** it is `ToggleDampener`
**And** the default binding is X (press to toggle)

**Given** a `DampenerState { active: bool }` component on the `PlayerShip` (initial `active = true`)
**When** the player presses `ToggleDampener`
**Then** `DampenerState.active` flips
**And** an `info!` log records the new state for dev feedback

**Given** `TuningConfig` is extended with `dampener_linear_strength: f32` (default 2.0) and `dampener_angular_strength: f32` (default 3.0)
**When** the dampener system runs in `FixedUpdate` inside `FlightSystems` and `DampenerState.active == true`
**Then** `ExternalForce` equal to `-linear_velocity * dampener_linear_strength * mass` is applied
**And** `ExternalTorque` equal to `-angular_velocity * dampener_angular_strength * moment_of_inertia` is applied

**Given** the dampener is active and the player releases all thrust/rotation input
**When** 3 seconds elapse
**Then** both linear and angular velocities are within 5% of zero

**Given** the dampener is inactive
**When** the player releases all input
**Then** velocities persist indefinitely (pure drift)

## Story 3.9: Weapon Firing + Projectile Ballistics

As a player,
I want to fire projectiles from my ship when I hold the primary-fire trigger,
So that I have an offensive capability in the Arena per FR9.

**Acceptance Criteria:**

**Given** a `CombatAction` enum is introduced in `src/combat/input.rs`
**When** the enum is defined
**Then** it includes `FirePrimary`
**And** the default binding is Left Mouse Button
**And** the `PlayerShip` bundle is extended to include `InputManagerBundle<CombatAction>`

**Given** `src/combat/mod.rs` is authored with a `CombatPlugin`
**When** the app starts
**Then** `CombatPlugin` declares a `CombatSystems` SystemSet
**And** `CombatPlugin` is registered in `main.rs`

**Given** `TuningConfig` is extended with `projectile_speed: f32` (120.0 m/s), `projectile_fire_rate_hz: f32` (4.0), `projectile_ttl_seconds: f32` (3.0)
**When** the player holds `FirePrimary`
**Then** a projectile-spawn system emits shots at `projectile_fire_rate_hz`, enforced via a `PrimaryWeaponCooldown { remaining: f32 }` component on the `PlayerShip`
**And** each shot spawns a `Projectile` entity with a placeholder sphere `Mesh3d` + `ToonMaterial`
**And** the `Projectile` is an Avian `RigidBody::Dynamic` with initial `LinearVelocity = ship_linear_velocity + ship_forward_vector * projectile_speed`
**And** the `Projectile` carries a `Collider`, a `Projectile { ttl: f32, damage: u32 }` component (default damage = 1), and the `ArenaEntity` marker

**Given** a `Projectile` with `ttl > 0`
**When** `FixedUpdate` runs
**Then** `ttl` decrements by the fixed timestep
**And** when `ttl <= 0` the `Projectile` entity is despawned

**Given** the player fires while stationary versus while drifting forward at 30 m/s
**When** projectile trajectories are measured
**Then** the drifting case's world velocity equals `30 + projectile_speed` forward
**And** the stationary case's world velocity equals `projectile_speed` forward

## Story 3.10: Projectile-Asteroid Collision & Damage

As a player,
I want my projectiles to destroy asteroids on contact,
So that I have a visible combat outcome per FR12, closing the core Arena interaction loop.

**Acceptance Criteria:**

**Given** Story 3.3's zone-spawner is extended
**When** each asteroid spawns
**Then** it carries an `AsteroidHp { current: u32 }` component with `current = 1` (single-hit destruction in Epic 3; multi-hit is Epic 4/5)

**Given** projectiles and asteroids share an Avian `CollisionLayers` group that allows contact
**When** a `Projectile` collides with an asteroid entity carrying `AsteroidHp`
**Then** a `ProjectileHitAsteroid { projectile: Entity, asteroid: Entity, damage: u32 }` event is emitted (past-tense PascalCase per architecture)
**And** `damage` is read from the `Projectile::damage` field

**Given** a `ProjectileHitAsteroid` event
**When** the damage-application system runs
**Then** `AsteroidHp.current = max(0, current - damage)`
**And** the projectile entity is despawned (single-hit-per-projectile — cluster-hit cases resolve to first contact only in Epic 3)

**Given** an asteroid reaches `AsteroidHp.current == 0`
**When** the destruction system runs
**Then** an `AsteroidDestroyed { asteroid: Entity }` event is emitted
**And** the asteroid entity is despawned
**And** the `cleanup_on_exit::<ArenaEntity>` pattern remains valid (mid-state despawn of individual ArenaEntity children does not break state-exit cleanup)

**Given** the player shoots an asteroid
**When** behaviour is observed
**Then** the projectile visibly disappears at impact
**And** the asteroid visibly disappears
**And** no error logs are emitted

## Story 3.11: HUD Baseline (Screen-Space Placeholders)

As a player,
I want a HUD showing placeholder shields, hull, ammo, and salvage values,
So that I learn where to look for tactical state before real values arrive in Epic 5 and 6.

**Acceptance Criteria:**

**Given** `src/ui/hud.rs` is authored with a `HudPlugin`
**When** `OnEnter(GameState::Arena)` runs
**Then** a full-window-absolute HUD root `Node` is spawned with a `HudEntity` marker

**Given** the HUD root exists
**When** child Nodes are spawned
**Then** four labeled fields are positioned in the four screen corners:
- top-left: `"SHIELDS 100"`
- top-right: `"HULL 100"`
- bottom-left: `"AMMO ∞"`
- bottom-right: `"SALVAGE 0"`

**And** each value is wired to a `HudPlaceholder { field: HudField }` component (enum variants Shields / Hull / Ammo / Salvage) for later Epic-5/6 connection

**Given** the HUD is visible
**When** the player flies, shoots, destroys asteroids
**Then** the four placeholder values remain static (real wiring is Epic 5/6)
**And** HUD elements do not obstruct the central line of sight to the asteroid field (transparent Node backgrounds, corner-anchored)

**Given** the state transitions `Arena → Paused` or `Arena → MainMenu`
**When** `OnExit(GameState::Arena)` runs
**Then** all `HudEntity`-marked entities are despawned (HUD is Arena-scoped in Epic 3; cross-state HUD persistence is Epic 4+)

<!-- Epic 3 complete — 11 stories deliver First Playable (M2). Next epic to decompose: Epic 4 (Enemies Alive & Stop-Ship / M3). -->
