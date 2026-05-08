# howlcore

`howlcore` is a Rust + Bevy learning prototype for studying the chimera gameplay structure used by Honkai: Star Rail event concepts.

This project is for private code reading and architecture practice. It does not include any original game art, audio, animation, UI, story, or asset files. Names such as chimera names, trait names, and mode names are used only as learning identifiers so the code can be compared with the source gameplay concepts.

## Run

```bash
cargo run
```

The app opens a Bevy debug UI. Press `Tab` to switch between Work Assignment and Chimera Battle, `Space` to advance the active mode, number keys `1-3` to buy draft shop offers, `R` to refresh the shop, `N` to reset the active mode, or `Esc` to quit.

## Test

```bash
cargo test
```

## Architecture

The project is split into two UI-independent gameplay cores plus a Bevy debug UI:

- `core::work`: single-team Work Assignment mode from The Awooo Firm style prototype, including a small multi-week ranking run and overtime loop.
- `core::battle`: two-team Chimera Battle mode from the Chrysos Awoo Championship style prototype.
- `content`: hard-coded learning content expressed as pure data.
- `ui`: Bevy adapter that can render and advance either `core::work` or `core::battle`.

The two modes intentionally use separate models. Work mode has `efficiency`, `stamina`, tasks, and cookie scoring. Battle mode has `attack`, `hp`, two teams, front-line targeting, and knockout/winner resolution.

## Work Assignment Flow

1. The app creates a test team of five chimeras.
2. `WorkRunState` starts the first review period from the stage's run config.
3. Each `Space` press advances one work round inside the current assignment.
4. The work queue repeatedly selects the current rightmost active chimera.
5. A chimera checks stamina, consumes the current front task's stamina cost, resolves `OnWork` traits, then contributes base progress by efficiency.
6. If a chimera no longer has enough stamina for its next task, it leaves the field and stops working.
7. Completed tasks grant Awoo Cookies.
8. A review period ends when all tasks are complete, the max round count is reached, or no active chimera can work.
9. If the weekly cookie target is met, the run promotes to the configured ranking target and advances to the next review period.
10. Reaching Rank 1 unlocks Overtime Mode.
11. Overtime generates repeating task cycles with growing progress, stamina, and cookie tuning.
12. Overtime carries the current stamina state between cleared cycles and ends when the team cannot fully clear a cycle.

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

## Battle Leader Flow

Battle mode supports one optional run leader loaded from battle RON content:

1. A `BattleLeader` owns a name, optional preferred shop tags, a shop-bias cadence, and a list of `BattleLeaderEffect` values.
2. Leader effects are applied when `BattleRunState` is created, before the first draft shop refresh.
3. The current supported leader effects can add starting gold, run health, gold inside win rewards, challenger team stats, and shop offer stats.
4. During shop refresh, every configured bias slot prefers an item whose tags match the leader's preferred shop tags.
5. Leader effects and shop bias are deterministic run-level modifiers, so they can later cover richer trainer, equipment, shop, or tag-specific rules without coupling them to the turn resolver.

## Chimera Draft Flow

Battle mode has a small draft/shop layer and run loop for building a lineup before combat:

1. A `DraftState` owns gold, the current team, bench, equipment inventory, and visible shop items.
2. Shop items can be chimeras or equipment. Buying a new chimera costs 3 gold; buying equipment costs 2 gold.
3. Buying a new chimera adds it to the back of the lineup, or sends it to the bench if the active lineup is full.
4. Buying a duplicate chimera merges into the existing active or benched chimera instead of adding a second copy.
5. Each duplicate grants +1 ATK, +1 max HP, +1 current HP, and +1 experience.
6. Level 2 costs 2 experience; Level 3 costs 3 experience.
7. Purchased equipment enters inventory and can be equipped onto an active chimera for direct ATK/HP bonuses.
8. Each chimera currently has one equipment slot; equipment can be unequipped to return it to inventory and roll back its stats.
9. Draft supports swapping active positions, moving active chimeras to the bench, and deploying benched chimeras while respecting the active lineup limit.
10. `BattleRunState` moves from Draft to the next configured opponent round, resolves the battle, applies that round's win rewards, then returns to Draft for the next opponent.
11. A run has explicit opponent rounds, health, win/loss counters, and completes when all opponents are defeated or health reaches zero.
12. The shop refreshes each Draft round from the configured deterministic item pool.
13. The debug UI supports number-key purchases from the shop pool, `R` shop refreshes, `Q/W/E` adjacent lineup swaps, `B` bench, `V` deploy, and `Z/X` equip/unequip.
14. Run tuning lives in battle RON content: starting gold, health, default loss damage, shop size, active lineup limit, explicit opponent rounds, structured win rewards, and shop items.
15. Win rewards are structured effects; the current supported rewards add gold, heal run health, or add a specific item to the shop.

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
      run.rs
    battle/
      event.rs
      model.rs
      resolver.rs
      draft.rs
      run.rs
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
- Add equipment, trainer pools, tag-specific leader effects, and saved defense lineups.
- Add richer Work mode ranking boards, weekly modifiers, Alpha Chimera selection, and team adjustment between overtime cycles.
- Move remaining hard-coded prototype tuning to RON or TOML files.
- Add richer Bevy UI panels, animation, and replay controls.
