# Engine Notes — Arena Demo Implementation

## ECS / Storage
- `partition!` macro and `Scope` now use the same `TypeId` key so the generated `Addition` can be added/removed from `Tables`.
- Added `Class::clear`, `Class::columns`, `Class::columns_mut`, `Tables::add`/`remove`, and `Domain::clear` to support scene teardown and re-registration.
- Added `Tables::get_mut` for mutable access to typed additions.

## Query Helpers
- Added `gather_ref`/`gather_mut` to look up a `PipId` in a specific component column.
- Added `query_mut_mut` for two mutable component iterators over the same class.

## Scene / Game Architecture
- `Scene` is now a trait with `name`, `player`, `register_tables`, `unregister_tables`, `populate`, `setup`, `teardown`, and `is_complete`.
- `Game` owns `SceneAccess` and handles scene switching, table teardown/population, and camera follow.
- `DomainView` now carries `asset_registry` and split (`with_solver_mut`/`with_script_mut`) for cleaner script/solver access.

## Scripts / Solvers
- `Solvers` are now ordered and removable.
- `PilotScript` uses a local `Vec` to queue projectiles instead of a `PipCommand` buffer.
- `SpawnerSolver`, `ProjectileSolver`, and `PickupSolver` use local `Vec`s for spawns, destroys, and heals, then flush after iteration.

## Assets
- Materials and textures live in `AssetRegistry` (Arc) and are held as `Brush`/`SpriteEntry` for the scene lifetime.
- Camera follows the player pip by `gather_ref`-ing the `CoreAddition` `xforms` column with the `player` id.

## Demo Scenes
- `SplashScene`: 8s wander-only intro with neutral props.
- `ArenaScene`: 20s or player-death, enemies chase the player, health pickups spawn.
- `SwarmScene`: 30s or player-death, dense enemy waves without pickups.

## Testing
- Unit tests for `gather` round-trip, `Domain::clear`, and the `Vec` spawn/destroy flush pattern.
