# howlcore

`howlcore` is a Rust + Bevy learning prototype for studying the chimera gameplay structure used by Honkai: Star Rail event concepts.

This project is for private code reading and architecture practice. It does not include any original game art, audio, animation, UI, story, or asset files. Names such as chimera names, trait names, and mode names are used only as learning identifiers so the code can be compared with the source gameplay concepts.

## Run

```bash
cargo run
```

The app currently opens the Work Assignment debug UI. Press `Space` to advance one work round, or `Esc` to quit.

## Test

```bash
cargo test
```

## Architecture

The project is split into two UI-independent gameplay cores plus a Bevy debug UI:

- `core::work`: single-team Work Assignment mode from The Awooo Firm style prototype.
- `core::battle`: two-team Chimera Battle mode from the Chrysos Awoo Championship style prototype.
- `content`: hard-coded learning content expressed as pure data.
- `ui`: Bevy adapter. It currently renders `core::work`; `core::battle` is implemented and tested at the core layer.

The two modes intentionally use separate models. Work mode has `efficiency`, `stamina`, tasks, and cookie scoring. Battle mode has `attack`, `hp`, two teams, front-line targeting, and knockout/winner resolution.

## Work Assignment Flow

1. The app creates a test team of five chimeras.
2. The stage creates three work tasks.
3. Each `Space` press advances one round.
4. Chimeras act from the rightmost slot to the leftmost slot: `slot 4 -> slot 3 -> slot 2 -> slot 1 -> slot 0`.
5. A chimera checks stamina, consumes the current front task's stamina cost, resolves `OnWork` traits, then contributes base progress by efficiency.
6. Completed tasks grant Awoo Cookies.
7. The work ends when all tasks are complete or the max round count is reached.

## Chimera Battle Flow

1. A battle creates two teams: Challenger and Defender.
2. Each team has an ordered chimera lineup with `attack`, `hp`, and `slot`.
3. The living chimera with the lowest slot is that team's front chimera.
4. Each turn, both front chimeras attack each other in the same exchange.
5. Damage is applied to both sides, knocked-down chimeras leave the front line, and the next living chimera takes over on the next turn.
6. The battle ends when one side has no living chimeras, both sides are defeated, or the max turn count is reached.

Battle mode now has its own data-driven ability layer. The first supported trigger/effect slice covers:

- `BeforeDamageTaken`: reduce incoming damage, used by Tough Cookie.
- `OnAllyAheadDamaged`: react when the ally one slot ahead takes damage, used by Healer.
- `AfterAttack`: perform follow-up damage, used by Workaholic.
- `AfterDamageTaken`: react after taking damage, used by Absentee Freak and Ruthless Demon.
- `BattleStart`: fire once before the first turn, used by Little Villain.
- `OnSummon`: react when a chimera joins the lineup from the summon queue.
- `OnKnockdown`: react when a chimera is knocked down, used by Kind Praiser.
- `Chance`: wraps nested effects behind a deterministic percent roll.
- Target selectors for self, attack target, damage target, summoned chimera, knocked-down chimera, front enemy, living enemies/allies, adjacent allies, and simple ranked enemy targets.

Damage is resolved through a small pipeline: attack request, incoming-damage modifiers, HP application, damage-taken reactions, ally reactions, and knockdown checks.

Battle mode also supports two foundational field-control effects:

- `SwapWithTarget`: swaps the source chimera's slot with a selected ally.
- `QueueSummon`: adds a chimera to the team's summon queue and deploys queued summons to the back of the lineup.

Battle state owns a deterministic RNG seed so probability effects can be replayed in tests and future battle logs. Chimeras also have baseline `level`, `rarity`, and `tags` fields for later shop, upgrade, equipment, and trainer-specific mechanics.

Battle test content is loaded from RON files:

- `assets/battle/abilities.ron`
- `assets/battle/test_battle.ron`

The loader keeps config DTOs in `content::battle_config` and converts them into pure `core::battle` types. It validates duplicate ability ids and unknown ability references before building a `BattleDefinition`.

## Chimera Draft Flow

Battle mode has a small draft/shop layer for building a lineup before combat:

1. A `DraftState` owns gold, the current team, and visible shop offers.
2. Buying a new chimera costs 3 gold and adds it to the back of the lineup.
3. Buying a duplicate merges into the existing chimera instead of adding a second copy.
4. Each duplicate grants +1 ATK, +1 max HP, +1 current HP, and +1 experience.
5. Level 2 costs 2 experience; Level 3 costs 3 experience.

## Directory Structure

```text
src/
  main.rs
  lib.rs
  app_state.rs
  core/
    mod.rs
    work/
      data.rs
      event.rs
      formula.rs
      log.rs
      model.rs
      resolver.rs
    battle/
      event.rs
      model.rs
      resolver.rs
  content/
    battle_config.rs
    battles.rs
    chimeras.rs
    stages.rs
  ui/
    mod.rs
  tests/
    battle_tests.rs
    combat_tests.rs
```

## Expansion Ideas

- Add battle abilities for knockout, richer summon rules, and level-scaled effects.
- Add shop refresh, equipment, trainer pools, and saved defense lineups.
- Add a Bevy debug UI switch between Work Assignment and Chimera Battle.
- Move hard-coded content to RON or TOML files.
- Add richer Bevy UI panels, animation, and replay controls.
