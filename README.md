# howlcore

`howlcore` is a Rust + Bevy learning prototype for studying the core gameplay structure of the Honkai: Star Rail 3.1 event "嗷呜嗷呜事务所 / The Awooo Firm".

This project is for private code reading and architecture practice. It does not include any original game art, audio, animation, UI, story, or asset files. Names such as chimera names, trait names, and mode names are used only as learning identifiers so the code can be compared with the source gameplay concept.

## Run

```bash
cargo run
```

Press `Space` to advance one work round. In some terminals, stdin may require pressing `Space` then `Enter`; piped spaces also work for quick checks.

## Test

```bash
cargo test
```

## Current Gameplay Flow

1. The app creates a test team of five chimeras.
2. The stage creates three work tasks.
3. Each `Space` press advances one round.
4. Chimeras act from the rightmost slot to the leftmost slot: `slot 4 -> slot 3 -> slot 2 -> slot 1 -> slot 0`.
5. A chimera checks stamina, consumes the current front task's stamina cost, resolves `OnWork` traits, then contributes base progress by efficiency.
6. Completed tasks grant Awoo Cookies.
7. The work ends when all tasks are complete or the max round count is reached.

## Concepts

- **Chimera**: A worker unit with a name, team id, slot, stats, and traits.
- **Trait**: A data-driven skill-like rule made from `Trigger`, `TargetSelector`, and `Effect`.
- **WorkTask**: A task with progress, required progress, stamina cost, and cookie reward.
- **Progress**: The task workload. Chimeras advance it with efficiency.
- **Stamina**: The resource spent to work on tasks.
- **Awoo Cookie**: The score rewarded by completed tasks or trait effects.

## Directory Structure

```text
src/
  main.rs
  lib.rs
  app_state.rs
  combat/
    mod.rs
    component.rs
    resource.rs
    event.rs
    system.rs
    data.rs
    resolver.rs
    target.rs
    formula.rs
    log.rs
  content/
    mod.rs
    chimeras.rs
    stages.rs
  tests/
    mod.rs
    combat_tests.rs
```

## Expansion Ideas

- Move hard-coded content to RON or TOML files.
- Add more chimeras and traits from the original gameplay structure.
- Add Overtime Mode for high-score attempts.
- Add a team ranking system.
- Add a simple Bevy UI.
- Add a work replay system.
- Split the combat core into an independent crate for easier unit testing.
