//! Pure resolution helpers for two-team chimera battles.

use crate::core::battle::{
    data::{BattleAbilityDef, BattleEffect, BattleTargetSelector, BattleTrigger},
    event::{BattleEvent, BattleOutcome},
    model::{BattleChimeraId, BattleState, TeamSide},
};

#[derive(Debug, Clone, Copy)]
struct BattleContext {
    attack_target: Option<BattleChimeraId>,
    damage_target: Option<BattleChimeraId>,
    summoned: Option<BattleChimeraId>,
    knocked_down: Option<BattleChimeraId>,
}

impl BattleContext {
    fn empty() -> Self {
        Self {
            attack_target: None,
            damage_target: None,
            summoned: None,
            knocked_down: None,
        }
    }

    fn attack(target: BattleChimeraId) -> Self {
        Self {
            attack_target: Some(target),
            damage_target: None,
            summoned: None,
            knocked_down: None,
        }
    }

    fn damage(target: BattleChimeraId) -> Self {
        Self {
            attack_target: None,
            damage_target: Some(target),
            summoned: None,
            knocked_down: None,
        }
    }

    fn summoned(summoned: BattleChimeraId) -> Self {
        Self {
            attack_target: None,
            damage_target: None,
            summoned: Some(summoned),
            knocked_down: None,
        }
    }

    fn knocked_down(knocked_down: BattleChimeraId) -> Self {
        Self {
            attack_target: None,
            damage_target: None,
            summoned: None,
            knocked_down: Some(knocked_down),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DamageRequest {
    target: BattleChimeraId,
    amount: i32,
}

pub fn step_turn(state: &mut BattleState) -> BattleOutcome {
    let mut outcome = BattleOutcome::default();

    if state.is_finished {
        return outcome;
    }

    if !state.has_started {
        state.has_started = true;
        outcome.push_log("Battle started.");
        for source in living_chimeras(state) {
            resolve_abilities(
                state,
                source,
                BattleTrigger::BattleStart,
                BattleContext::empty(),
                &mut outcome,
            );
        }
        check_battle_end(state, &mut outcome);

        if state.is_finished {
            return outcome;
        }
    }

    state.turn += 1;
    outcome.push_event(BattleEvent::TurnStarted { turn: state.turn });
    outcome.push_log(turn_line(state.turn, "Turn started."));

    for source in living_chimeras(state) {
        resolve_abilities(
            state,
            source,
            BattleTrigger::TurnStart,
            BattleContext::empty(),
            &mut outcome,
        );
    }

    let challenger_front = front_chimera_id(state, TeamSide::Challenger);
    let defender_front = front_chimera_id(state, TeamSide::Defender);

    match (challenger_front, defender_front) {
        (Some(challenger), Some(defender)) => {
            resolve_front_exchange(state, challenger, defender, &mut outcome);
            check_battle_end(state, &mut outcome);
        }
        _ => finish_by_alive_teams(state, &mut outcome),
    }

    outcome
}

pub fn front_chimera_id(state: &BattleState, side: TeamSide) -> Option<BattleChimeraId> {
    state
        .team(side)
        .chimeras
        .iter()
        .enumerate()
        .filter(|(_, chimera)| chimera.is_alive())
        .min_by_key(|(_, chimera)| chimera.slot)
        .map(|(index, _)| BattleChimeraId { side, index })
}

pub fn living_chimera_count(state: &BattleState, side: TeamSide) -> usize {
    state
        .team(side)
        .chimeras
        .iter()
        .filter(|chimera| chimera.is_alive())
        .count()
}

fn resolve_front_exchange(
    state: &mut BattleState,
    challenger: BattleChimeraId,
    defender: BattleChimeraId,
    outcome: &mut BattleOutcome,
) {
    let challenger_attack = attack_value(state, challenger);
    let defender_attack = attack_value(state, defender);
    let challenger_name = chimera_name(state, challenger);
    let defender_name = chimera_name(state, defender);

    outcome.push_log(turn_line(
        state.turn,
        format!("{challenger_name} and {defender_name} attack each other."),
    ));

    outcome.push_event(BattleEvent::BasicAttack {
        attacker: challenger,
        target: defender,
        damage: challenger_attack,
    });
    outcome.push_event(BattleEvent::BasicAttack {
        attacker: defender,
        target: challenger,
        damage: defender_attack,
    });

    resolve_ally_attack_abilities(state, challenger, defender, outcome);
    resolve_ally_attack_abilities(state, defender, challenger, outcome);

    apply_damage(
        state,
        DamageRequest {
            target: defender,
            amount: challenger_attack,
        },
        outcome,
    );
    apply_damage(
        state,
        DamageRequest {
            target: challenger,
            amount: defender_attack,
        },
        outcome,
    );

    emit_knockdowns(state, [challenger, defender], outcome);

    resolve_abilities(
        state,
        challenger,
        BattleTrigger::AfterAttack,
        BattleContext::attack(defender),
        outcome,
    );
    resolve_abilities(
        state,
        defender,
        BattleTrigger::AfterAttack,
        BattleContext::attack(challenger),
        outcome,
    );
}

fn apply_damage(state: &mut BattleState, mut request: DamageRequest, outcome: &mut BattleOutcome) {
    request.amount = request.amount.max(0);
    resolve_incoming_damage_modifiers(state, &mut request, outcome);

    let target_name = chimera_name(state, request.target);
    let turn = state.turn;
    let mut hp_line = None;

    if let Some(chimera) = state.chimera_mut(request.target) {
        chimera.stats.hp = (chimera.stats.hp - request.amount).max(0);
        hp_line = Some(format!(
            "{target_name} took {} damage. HP: {}/{}.",
            request.amount, chimera.stats.hp, chimera.stats.max_hp
        ));
    }

    outcome.push_event(BattleEvent::DamageDealt {
        target: request.target,
        amount: request.amount,
    });

    if let Some(line) = hp_line {
        outcome.push_log(turn_line(turn, line));
    }

    let context = BattleContext::damage(request.target);
    resolve_abilities(
        state,
        request.target,
        BattleTrigger::AfterDamageTaken,
        context,
        outcome,
    );
    resolve_ally_ahead_damaged_abilities(state, request.target, context, outcome);
}

fn resolve_incoming_damage_modifiers(
    state: &mut BattleState,
    request: &mut DamageRequest,
    outcome: &mut BattleOutcome,
) {
    if !is_alive(state, request.target) {
        return;
    }

    let abilities =
        ability_defs_for_chimera(state, request.target, BattleTrigger::BeforeDamageTaken);
    for ability in abilities {
        outcome.push_event(BattleEvent::AbilityTriggered {
            source: request.target,
            ability: ability.id,
        });
        outcome.push_log(turn_line(
            state.turn,
            format!(
                "{} used {}.",
                chimera_name(state, request.target),
                ability.name
            ),
        ));

        for effect in ability.effects {
            if let BattleEffect::ReduceIncomingDamage { amount, minimum } = effect {
                let before = request.amount;
                request.amount = (request.amount - amount).max(minimum.max(0));
                let reduced = before - request.amount;

                if reduced > 0 {
                    outcome.push_event(BattleEvent::DamageReduced {
                        target: request.target,
                        amount: reduced,
                    });
                    outcome.push_log(turn_line(
                        state.turn,
                        format!(
                            "{} reduced incoming damage by {reduced}.",
                            chimera_name(state, request.target)
                        ),
                    ));
                }
            }
        }
    }
}

fn resolve_ally_attack_abilities(
    state: &mut BattleState,
    attacker: BattleChimeraId,
    target: BattleChimeraId,
    outcome: &mut BattleOutcome,
) {
    let sources = living_chimeras_for_side(state, attacker.side)
        .into_iter()
        .filter(|source| *source != attacker)
        .collect::<Vec<_>>();
    let context = BattleContext::attack(target);

    for source in sources {
        resolve_abilities(state, source, BattleTrigger::OnAllyAttack, context, outcome);
    }
}

fn resolve_ally_ahead_damaged_abilities(
    state: &mut BattleState,
    damaged: BattleChimeraId,
    context: BattleContext,
    outcome: &mut BattleOutcome,
) {
    let sources = living_chimeras(state)
        .into_iter()
        .filter(|source| source.side == damaged.side && *source != damaged)
        .filter(|source| is_ally_behind(state, *source, damaged))
        .collect::<Vec<_>>();

    for source in sources {
        resolve_abilities(
            state,
            source,
            BattleTrigger::OnAllyAheadDamaged,
            context,
            outcome,
        );
    }
}

fn resolve_abilities(
    state: &mut BattleState,
    source: BattleChimeraId,
    trigger: BattleTrigger,
    context: BattleContext,
    outcome: &mut BattleOutcome,
) {
    if !is_alive(state, source) {
        return;
    }

    let abilities = ability_defs_for_chimera(state, source, trigger);
    for ability in abilities {
        let targets = select_targets(state, source, ability.selector, context);
        outcome.push_event(BattleEvent::AbilityTriggered {
            source,
            ability: ability.id,
        });
        outcome.push_log(turn_line(
            state.turn,
            format!("{} used {}.", chimera_name(state, source), ability.name),
        ));

        for target in targets {
            for effect in &ability.effects {
                apply_effect(state, source, target, effect, outcome);
            }
        }

        deploy_queued_summons(state, source.side, outcome);
    }
}

fn apply_effect(
    state: &mut BattleState,
    source: BattleChimeraId,
    target: BattleChimeraId,
    effect: &BattleEffect,
    outcome: &mut BattleOutcome,
) {
    match effect {
        BattleEffect::Chance { percent, effects } => {
            if roll_chance(state, *percent, outcome) {
                for effect in effects {
                    apply_effect(state, source, target, effect, outcome);
                }
            }
        }
        BattleEffect::DealDamage { amount } => apply_damage(
            state,
            DamageRequest {
                target,
                amount: *amount,
            },
            outcome,
        ),
        BattleEffect::DealAttackDamagePercent { percent, minimum } => {
            let damage = attack_value(state, source) * *percent as i32 / 100;
            apply_damage(
                state,
                DamageRequest {
                    target,
                    amount: damage.max(*minimum),
                },
                outcome,
            );
        }
        BattleEffect::Heal { amount } => heal(state, target, *amount, outcome),
        BattleEffect::AddAttack { amount } => add_attack(state, target, *amount, outcome),
        BattleEffect::ReduceIncomingDamage { .. } => {}
        BattleEffect::SwapWithTarget => swap_positions(state, source, target, outcome),
        BattleEffect::QueueSummon {
            name,
            attack,
            hp,
            abilities,
        } => queue_summon(state, source.side, name, *attack, *hp, abilities, outcome),
    }
}

fn roll_chance(state: &mut BattleState, percent: u32, outcome: &mut BattleOutcome) -> bool {
    let percent = percent.min(100);
    let roll = state.rng.roll_percent();
    let success = roll <= percent;

    outcome.push_event(BattleEvent::ChanceRolled {
        percent,
        roll,
        success,
    });
    outcome.push_log(turn_line(
        state.turn,
        format!(
            "Chance roll {roll}/100 against {percent}%: {}.",
            if success { "success" } else { "failed" }
        ),
    ));

    success
}

fn select_targets(
    state: &BattleState,
    source: BattleChimeraId,
    selector: BattleTargetSelector,
    context: BattleContext,
) -> Vec<BattleChimeraId> {
    match selector {
        BattleTargetSelector::SelfChimera => vec![source],
        BattleTargetSelector::AttackTarget => context.attack_target.into_iter().collect(),
        BattleTargetSelector::DamageTarget => context.damage_target.into_iter().collect(),
        BattleTargetSelector::SummonedChimera => context.summoned.into_iter().collect(),
        BattleTargetSelector::KnockedDownChimera => context.knocked_down.into_iter().collect(),
        BattleTargetSelector::FrontEnemy => front_chimera_id(state, source.side.opponent())
            .into_iter()
            .collect(),
        BattleTargetSelector::FirstLivingEnemy => {
            living_chimeras_for_side(state, source.side.opponent())
                .into_iter()
                .next()
                .into_iter()
                .collect()
        }
        BattleTargetSelector::AllyAhead => ally_by_slot_offset(state, source, -1),
        BattleTargetSelector::AllyBehind => ally_by_slot_offset(state, source, 1),
        BattleTargetSelector::HighestHpEnemy => {
            ranked_enemy(state, source, |chimera| chimera.stats.hp)
        }
        BattleTargetSelector::HighestAttackEnemy => {
            ranked_enemy(state, source, |chimera| chimera.stats.attack)
        }
        BattleTargetSelector::AllEnemies => living_chimeras_for_side(state, source.side.opponent()),
        BattleTargetSelector::AllAllies => living_chimeras_for_side(state, source.side),
    }
}

fn ability_defs_for_chimera(
    state: &BattleState,
    chimera: BattleChimeraId,
    trigger: BattleTrigger,
) -> Vec<BattleAbilityDef> {
    state
        .chimera(chimera)
        .into_iter()
        .flat_map(|chimera| chimera.abilities.iter().copied())
        .filter_map(|id| state.ability_database.abilities.get(&id))
        .filter(|ability| ability.trigger == trigger)
        .cloned()
        .collect()
}

fn living_chimeras(state: &BattleState) -> Vec<BattleChimeraId> {
    let mut chimeras = living_chimeras_for_side(state, TeamSide::Challenger);
    chimeras.extend(living_chimeras_for_side(state, TeamSide::Defender));
    chimeras
}

fn living_chimeras_for_side(state: &BattleState, side: TeamSide) -> Vec<BattleChimeraId> {
    let mut chimeras = state
        .team(side)
        .chimeras
        .iter()
        .enumerate()
        .filter(|(_, chimera)| chimera.is_alive())
        .map(|(index, chimera)| (BattleChimeraId { side, index }, chimera.slot))
        .collect::<Vec<_>>();
    chimeras.sort_by_key(|(_, slot)| *slot);
    chimeras.into_iter().map(|(id, _)| id).collect()
}

fn ranked_enemy(
    state: &BattleState,
    source: BattleChimeraId,
    value: impl Fn(&crate::core::battle::BattleChimera) -> i32,
) -> Vec<BattleChimeraId> {
    let mut enemies = state
        .team(source.side.opponent())
        .chimeras
        .iter()
        .enumerate()
        .filter(|(_, chimera)| chimera.is_alive())
        .map(|(index, chimera)| {
            (
                BattleChimeraId {
                    side: source.side.opponent(),
                    index,
                },
                value(chimera),
            )
        })
        .collect::<Vec<_>>();
    enemies.sort_by(|(_, left), (_, right)| right.cmp(left));
    enemies.first().map(|(id, _)| vec![*id]).unwrap_or_default()
}

fn ally_by_slot_offset(
    state: &BattleState,
    source: BattleChimeraId,
    offset: i32,
) -> Vec<BattleChimeraId> {
    let Some(source_chimera) = state.chimera(source) else {
        return Vec::new();
    };
    let wanted_slot = source_chimera.slot as i32 + offset;
    if wanted_slot < 0 {
        return Vec::new();
    }

    state
        .team(source.side)
        .chimeras
        .iter()
        .enumerate()
        .find(|(_, chimera)| chimera.is_alive() && chimera.slot == wanted_slot as u32)
        .map(|(index, _)| {
            vec![BattleChimeraId {
                side: source.side,
                index,
            }]
        })
        .unwrap_or_default()
}

fn is_ally_behind(state: &BattleState, source: BattleChimeraId, ahead: BattleChimeraId) -> bool {
    let Some(source_chimera) = state.chimera(source) else {
        return false;
    };
    let Some(ahead_chimera) = state.chimera(ahead) else {
        return false;
    };

    source_chimera.slot == ahead_chimera.slot + 1
}

fn attack_value(state: &BattleState, chimera: BattleChimeraId) -> i32 {
    state
        .chimera(chimera)
        .map(|chimera| chimera.stats.attack.max(0))
        .unwrap_or_default()
}

fn heal(
    state: &mut BattleState,
    target: BattleChimeraId,
    amount: i32,
    outcome: &mut BattleOutcome,
) {
    let amount = amount.max(0);
    let target_name = chimera_name(state, target);
    let turn = state.turn;
    let mut hp_line = None;

    if let Some(chimera) = state.chimera_mut(target) {
        let before = chimera.stats.hp;
        chimera.stats.hp = (chimera.stats.hp + amount).min(chimera.stats.max_hp);
        let restored = chimera.stats.hp - before;
        outcome.push_event(BattleEvent::HpRestored {
            target,
            amount: restored,
        });
        hp_line = Some(format!(
            "{target_name} restored {restored} HP. HP: {}/{}.",
            chimera.stats.hp, chimera.stats.max_hp
        ));
    }

    if let Some(line) = hp_line {
        outcome.push_log(turn_line(turn, line));
    }
}

fn add_attack(
    state: &mut BattleState,
    target: BattleChimeraId,
    amount: i32,
    outcome: &mut BattleOutcome,
) {
    let target_name = chimera_name(state, target);
    let turn = state.turn;
    let mut attack_line = None;

    if let Some(chimera) = state.chimera_mut(target) {
        chimera.stats.attack = (chimera.stats.attack + amount).max(0);
        outcome.push_event(BattleEvent::AttackChanged { target, amount });
        attack_line = Some(format!(
            "{target_name} attack changed by {amount}. ATK: {}.",
            chimera.stats.attack
        ));
    }

    if let Some(line) = attack_line {
        outcome.push_log(turn_line(turn, line));
    }
}

fn swap_positions(
    state: &mut BattleState,
    source: BattleChimeraId,
    target: BattleChimeraId,
    outcome: &mut BattleOutcome,
) {
    if source.side != target.side || source.index == target.index {
        return;
    }

    let source_name = chimera_name(state, source);
    let target_name = chimera_name(state, target);
    let team = state.team_mut(source.side);

    if source.index >= team.chimeras.len() || target.index >= team.chimeras.len() {
        return;
    }

    let source_slot = team.chimeras[source.index].slot;
    let target_slot = team.chimeras[target.index].slot;
    team.chimeras[source.index].slot = target_slot;
    team.chimeras[target.index].slot = source_slot;

    outcome.push_event(BattleEvent::PositionSwapped {
        first: source,
        second: target,
    });
    outcome.push_log(turn_line(
        state.turn,
        format!("{source_name} swapped positions with {target_name}."),
    ));
}

fn queue_summon(
    state: &mut BattleState,
    side: TeamSide,
    name: &'static str,
    attack: i32,
    hp: i32,
    abilities: &[crate::core::battle::BattleAbilityId],
    outcome: &mut BattleOutcome,
) {
    let chimera = crate::core::battle::BattleChimera {
        name: name.to_string(),
        slot: 0,
        level: 1,
        experience: 0,
        rarity: crate::core::battle::BattleRarity::White,
        tags: Vec::new(),
        stats: crate::core::battle::BattleStats {
            attack,
            max_hp: hp,
            hp,
        },
        abilities: abilities.to_vec(),
        equipment: Vec::new(),
    };
    state.team_mut(side).summon_queue.push(chimera);
    outcome.push_event(BattleEvent::ChimeraQueued {
        side,
        name: name.to_string(),
    });
    outcome.push_log(turn_line(
        state.turn,
        format!("{name} was added to {:?}'s summon queue.", side),
    ));
}

fn deploy_queued_summons(state: &mut BattleState, side: TeamSide, outcome: &mut BattleOutcome) {
    loop {
        if state.team(side).summon_queue.is_empty() {
            break;
        };

        let mut chimera = state.team_mut(side).summon_queue.remove(0);
        let slot = next_back_slot(state, side);
        chimera.slot = slot;
        let name = chimera.name.clone();
        let index = state.team(side).chimeras.len();
        state.team_mut(side).chimeras.push(chimera);
        let id = BattleChimeraId { side, index };

        outcome.push_event(BattleEvent::ChimeraSummoned { chimera: id });
        outcome.push_log(turn_line(
            state.turn,
            format!("{name} joined {:?}'s lineup at slot {slot}.", side),
        ));

        resolve_on_summon_abilities(state, id, outcome);
    }
}

fn resolve_on_summon_abilities(
    state: &mut BattleState,
    summoned: BattleChimeraId,
    outcome: &mut BattleOutcome,
) {
    let sources = living_chimeras_for_side(state, summoned.side);
    let context = BattleContext::summoned(summoned);

    for source in sources {
        resolve_abilities(state, source, BattleTrigger::OnSummon, context, outcome);
    }
}

fn next_back_slot(state: &BattleState, side: TeamSide) -> u32 {
    state
        .team(side)
        .chimeras
        .iter()
        .map(|chimera| chimera.slot)
        .max()
        .map(|slot| slot + 1)
        .unwrap_or_default()
}

fn emit_knockdowns(
    state: &mut BattleState,
    chimeras: [BattleChimeraId; 2],
    outcome: &mut BattleOutcome,
) {
    for chimera in chimeras {
        if state
            .chimera(chimera)
            .is_some_and(|chimera| !chimera.is_alive())
        {
            outcome.push_event(BattleEvent::ChimeraKnockedDown { chimera });
            outcome.push_log(turn_line(
                state.turn,
                format!("{} was knocked down.", chimera_name(state, chimera)),
            ));
            resolve_on_knockdown_abilities(state, chimera, outcome);
        }
    }
}

fn resolve_on_knockdown_abilities(
    state: &mut BattleState,
    knocked_down: BattleChimeraId,
    outcome: &mut BattleOutcome,
) {
    let sources = living_chimeras(state)
        .into_iter()
        .filter(|source| *source != knocked_down)
        .collect::<Vec<_>>();
    let context = BattleContext::knocked_down(knocked_down);

    for source in sources {
        resolve_abilities(state, source, BattleTrigger::OnKnockdown, context, outcome);
    }
}

fn check_battle_end(state: &mut BattleState, outcome: &mut BattleOutcome) {
    if state.is_finished {
        return;
    }

    let challenger_alive = living_chimera_count(state, TeamSide::Challenger);
    let defender_alive = living_chimera_count(state, TeamSide::Defender);

    if challenger_alive == 0 || defender_alive == 0 {
        finish_by_alive_teams(state, outcome);
        return;
    }

    if state.turn >= state.max_turn {
        let winner = winner_by_remaining_hp(state);
        finish(state, winner, outcome);
    }
}

fn finish_by_alive_teams(state: &mut BattleState, outcome: &mut BattleOutcome) {
    let challenger_alive = living_chimera_count(state, TeamSide::Challenger);
    let defender_alive = living_chimera_count(state, TeamSide::Defender);
    let winner = match (challenger_alive > 0, defender_alive > 0) {
        (true, false) => Some(TeamSide::Challenger),
        (false, true) => Some(TeamSide::Defender),
        _ => None,
    };
    finish(state, winner, outcome);
}

fn finish(state: &mut BattleState, winner: Option<TeamSide>, outcome: &mut BattleOutcome) {
    state.is_finished = true;
    state.winner = winner;
    outcome.push_event(BattleEvent::BattleEnded { winner });
    outcome.push_log(match winner {
        Some(TeamSide::Challenger) => "Battle ended. Winner: Challenger.".to_string(),
        Some(TeamSide::Defender) => "Battle ended. Winner: Defender.".to_string(),
        None => "Battle ended. Result: Draw.".to_string(),
    });
}

fn winner_by_remaining_hp(state: &BattleState) -> Option<TeamSide> {
    let challenger_hp = total_remaining_hp(state, TeamSide::Challenger);
    let defender_hp = total_remaining_hp(state, TeamSide::Defender);

    match challenger_hp.cmp(&defender_hp) {
        std::cmp::Ordering::Greater => Some(TeamSide::Challenger),
        std::cmp::Ordering::Less => Some(TeamSide::Defender),
        std::cmp::Ordering::Equal => None,
    }
}

fn total_remaining_hp(state: &BattleState, side: TeamSide) -> i32 {
    state
        .team(side)
        .chimeras
        .iter()
        .map(|chimera| chimera.stats.hp.max(0))
        .sum()
}

fn is_alive(state: &BattleState, id: BattleChimeraId) -> bool {
    state.chimera(id).is_some_and(|chimera| chimera.is_alive())
}

fn chimera_name(state: &BattleState, id: BattleChimeraId) -> String {
    state
        .chimera(id)
        .map(|chimera| chimera.name.clone())
        .unwrap_or_else(|| format!("{:?} Chimera {}", id.side, id.index))
}

fn turn_line(turn: u32, message: impl AsRef<str>) -> String {
    format!("[Turn {turn}] {}", message.as_ref())
}
