# Physics Pipeline Plan

## Stage 1: Spatial Hashing (Broad Phase)

Produce candidate collision pairs from AABB overlaps using a spatial hash grid.

- [x] 1.1 Define `AABB` type in `src/spacial/`
- [x] 1.2 Create `BroadPhaseAdd` addition with `aabb: Class<AABB>` table
- [x] 1.3 Define `SpatialHash` struct (cell map + cell size + frame stamp buffer)
- [x] 1.4 Define `CandidatePairs` struct (reused `Vec<(PipId, PipId)>`)
- [x] 1.5 Implement `BroadPhaseSolver`:
  - [x] 1.5a Update AABBs from transforms + brush scales (query Active only)
  - [x] 1.5b Rebuild spatial hash from Active AABBs
  - [x] 1.5c Generate candidate pairs with frame-stamp dedup
- [x] 1.6 Wire into test scene
- [x] 1.7 Verify: pairs generated, count scales linearly

## Stage 2: Contact Manifolds (Narrow Phase)

For each candidate pair, generate a 2-point contact manifold (2D). Persist across frames for warm starting.

- [ ] 2.1 Define `ManifoldPoint` (local_a, local_b, separation, contact_id, normal_impulse, tangent_impulse)
- [ ] 2.2 Define `ContactPair` (body_a, body_b, normal, friction, restitution, point range, point count)
- [ ] 2.3 Create `NarrowPhaseAdd` addition with `contacts: Class<ContactPair>` and `manifold_points: Class<ManifoldPoint>`
- [ ] 2.4 Implement SAT + clipping for box-vs-box (the test scene shape)
- [ ] 2.5 Implement manifold persistence: match by contact_id, copy impulses for warm start
- [ ] 2.6 Implement `NarrowPhaseSolver`: consume candidate pairs, produce/update manifolds
- [ ] 2.7 Verify: manifolds persist across frames, impulses carry over

## Stage 3: Solver Subpasses (Constraint Solve)

Split the physics solver into ordered phases following Box2D v3's stage enum.

- [ ] 3.1 Define `BodyState` (linear_velocity, angular_velocity, position_delta) separate from `BodySim` (mass, inv_mass, inertia)
- [ ] 3.2 Refactor `PhysicsSolver` into ordered subpass methods:
  - [ ] 3.2a `prepare_contacts` — compute Jacobians, effective mass from manifolds
  - [ ] 3.2b `integrate_velocities` — apply gravity, damping to Active bodies
  - [ ] 3.2c `warm_start` — seed impulses from manifold cache
  - [ ] 3.2d `solve_velocity` — sequential impulse over contact constraints
  - [ ] 3.2e `integrate_positions` — advance transforms by velocity * dt
  - [ ] 3.2f `solve_position` — split impulse position correction
  - [ ] 3.2g `store_impulses` — write accumulated impulses back to manifolds
- [ ] 3.3 Sub-step loop (configurable iteration count)
- [ ] 3.4 Verify: boxes rest stably on each other, no jitter/sinking

## Stage 4: Sleep Islands (Rest Management)

Group bodies into connected components via contacts. Sleep/wake per-island, not per-body.

- [ ] 4.1 Define `IslandId` and `Island` struct (sleeping flag, min_sleep_timer, body range)
- [ ] 4.2 Add `island_id: Class<Option<IslandId>>` per-body column
- [ ] 4.3 Implement island formation from contact graph (union-find or DFS)
- [ ] 4.4 Implement incremental island updates (edge events: contact start/stop)
- [ ] 4.5 Implement island sleep check: all members below threshold for min time
- [ ] 4.6 Implement island wake: any member disturbed → wake whole island, reset timers
- [ ] 4.7 Bulk `move_pip` for island transitions (Active ↔ Sleeping)
- [ ] 4.8 Replace per-body sleep check in MotionSolver with island-based check
- [ ] 4.9 Verify: stacks sleep as a unit, disturbance wakes the whole stack
