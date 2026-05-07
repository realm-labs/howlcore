//! Pure gameplay resolution helpers.

use crate::core::work::{
    data::{Effect, TargetSelector, TraitDef, TraitId, Trigger},
    event::{CombatEvent, RoundOutcome},
    formula::{base_work_progress, clamp_stamina},
    log::round_line,
    model::{ChimeraId, CombatState, TaskId, TimedEfficiencyBonus, TimedTrait},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectTarget {
    Chimera(ChimeraId),
    Task(TaskId),
    Global,
}

pub fn step_round(state: &mut CombatState) -> RoundOutcome {
    let mut outcome = RoundOutcome::default();

    if state.is_finished {
        return outcome;
    }

    state.round += 1;
    outcome.push_log(round_line(state.round, "Round started."));
    outcome.push_event(CombatEvent::RoundStarted { round: state.round });

    for chimera in action_order(state) {
        resolve_trigger_for_chimera(state, chimera, Trigger::RoundStart, &mut outcome);
    }

    let names = action_order(state)
        .iter()
        .map(|chimera| chimera_name(state, *chimera))
        .collect::<Vec<_>>();
    outcome.push_log(round_line(
        state.round,
        format!("Work queue: {}.", names.join(" -> ")),
    ));

    let max_work_actions = state.chimeras.len().saturating_mul(32).max(1);
    let mut work_actions = 0;
    for _ in 0..max_work_actions {
        let Some(chimera) = next_work_chimera(state) else {
            break;
        };

        work_actions += 1;
        outcome.push_event(CombatEvent::WorkActionRequested { chimera });
        perform_work_action(state, chimera, &mut outcome);

        if all_tasks_completed(state) {
            break;
        }
    }
    if work_actions == max_work_actions && next_work_chimera(state).is_some() {
        outcome.push_log(round_line(
            state.round,
            "Work queue stopped after reaching the safety action limit.",
        ));
    }

    for chimera in action_order(state) {
        resolve_trigger_for_chimera(state, chimera, Trigger::RoundEnd, &mut outcome);
    }

    expire_timed_effects(state, &mut outcome);

    outcome.push_log(round_line(
        state.round,
        format!("Round ended. Awoo Cookies: {}.", state.cookie_score),
    ));

    check_work_end(state, &mut outcome);

    outcome
}

pub fn chimera_name(state: &CombatState, id: ChimeraId) -> String {
    state
        .chimera(id)
        .map(|chimera| chimera.name.clone())
        .unwrap_or_else(|| format!("Chimera {}", id.0))
}

pub fn task_name(state: &CombatState, id: TaskId) -> String {
    state
        .task(id)
        .map(|task| task.name.clone())
        .unwrap_or_else(|| format!("Task {}", id.0))
}

pub fn total_task_count(state: &CombatState) -> usize {
    state.tasks.len()
}

pub fn all_tasks_completed(state: &CombatState) -> bool {
    state.tasks.iter().all(|task| task.progress.completed)
}

pub fn front_task_id(state: &CombatState) -> Option<TaskId> {
    state
        .tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| !task.progress.completed)
        .min_by_key(|(_, task)| task.order)
        .map(|(index, _)| TaskId(index))
}

pub fn action_order(state: &CombatState) -> Vec<ChimeraId> {
    let mut chimeras = state
        .chimeras
        .iter()
        .enumerate()
        .filter(|(_, chimera)| chimera.is_active)
        .map(|(index, chimera)| (ChimeraId(index), chimera.slot))
        .collect::<Vec<_>>();

    chimeras.sort_by(|(_, left_slot), (_, right_slot)| right_slot.cmp(left_slot));
    chimeras.into_iter().map(|(id, _)| id).collect()
}

pub fn next_work_chimera(state: &CombatState) -> Option<ChimeraId> {
    front_task_id(state)?;
    action_order(state).into_iter().next()
}

pub fn select_targets(
    state: &CombatState,
    selector: TargetSelector,
    source: ChimeraId,
) -> Vec<EffectTarget> {
    match selector {
        TargetSelector::SelfChimera => vec![EffectTarget::Chimera(source)],
        TargetSelector::FrontTask => front_task_id(state)
            .map(EffectTarget::Task)
            .into_iter()
            .collect(),
        TargetSelector::AllTasks => all_task_targets(state),
        TargetSelector::PreviousAlly => ally_by_slot_offset(state, source, -1),
        TargetSelector::NextAlly => ally_by_slot_offset(state, source, 1),
        TargetSelector::AdjacentAllies => {
            let mut targets = ally_by_slot_offset(state, source, -1);
            targets.extend(ally_by_slot_offset(state, source, 1));
            targets
        }
        TargetSelector::AllAllies => all_ally_targets(state, source),
        TargetSelector::LowestStaminaAlly => stamina_ranked_ally(state, source, true),
        TargetSelector::HighestEfficiencyAlly => efficiency_ranked_ally(state, source),
    }
}

pub fn advance_task_progress(
    state: &mut CombatState,
    task: TaskId,
    amount: i32,
    reason: &str,
    outcome: &mut RoundOutcome,
) {
    let round = state.round;
    let task_name = task_name(state, task);
    let amount = amount.max(0);
    let mut completed_reward = None;
    let mut progress_line = None;

    if let Some(task) = state.task_mut(task) {
        if task.progress.completed {
            return;
        }

        task.progress.current += amount;
        progress_line = Some(round_line(
            round,
            format!(
                "{task_name} progress +{}. Progress: {}/{}. ({reason})",
                amount, task.progress.current, task.progress.required
            ),
        ));

        if task.progress.current >= task.progress.required {
            task.progress.completed = true;
            completed_reward = Some(task.progress.cookie_reward);
        }
    }

    if let Some(line) = progress_line {
        outcome.push_log(line);
    }

    if let Some(cookie_reward) = completed_reward {
        state.completed_tasks += 1;
        state.cookie_score += cookie_reward;
        outcome.push_event(CombatEvent::CookieGained {
            amount: cookie_reward,
        });
        outcome.push_event(CombatEvent::TaskCompleted {
            task,
            cookie_reward,
        });
        outcome.push_log(round_line(
            round,
            format!("{task_name} completed. Gained {cookie_reward} Awoo Cookies."),
        ));

        resolve_trigger_for_all_chimeras(state, Trigger::TaskCompleted, outcome);
    }
}

pub fn apply_effect(
    state: &mut CombatState,
    source: ChimeraId,
    target: EffectTarget,
    effect: &Effect,
    trait_name: &str,
    outcome: &mut RoundOutcome,
) {
    outcome.push_event(CombatEvent::EffectApplied {
        description: format!("{trait_name} applied {effect:?}"),
    });

    match (target, effect) {
        (EffectTarget::Task(task), Effect::AdvanceTask { amount }) => {
            advance_task_progress(state, task, *amount, trait_name, outcome);
        }
        (EffectTarget::Task(task), Effect::AdvanceTaskByEfficiency { bonus }) => {
            let efficiency = state
                .chimera(source)
                .map(|chimera| chimera.stats.efficiency)
                .unwrap_or_default();
            advance_task_progress(state, task, efficiency + bonus, trait_name, outcome);
        }
        (EffectTarget::Global, Effect::GainCookie { amount }) => {
            gain_cookie(state, *amount, outcome);
        }
        (EffectTarget::Chimera(_), Effect::GainCookie { amount }) => {
            gain_cookie(state, *amount, outcome);
        }
        (EffectTarget::Chimera(chimera), Effect::AddEfficiency { amount, duration }) => {
            add_efficiency(state, chimera, *amount, *duration, outcome);
        }
        (EffectTarget::Chimera(chimera), Effect::AddStamina { amount }) => {
            change_stamina(state, chimera, *amount, "gained", outcome);
        }
        (EffectTarget::Chimera(chimera), Effect::ConsumeStamina { amount }) => {
            change_stamina(state, chimera, -*amount, "consumed", outcome);
        }
        (EffectTarget::Chimera(chimera), Effect::RestoreStamina { amount }) => {
            change_stamina(state, chimera, *amount, "restored", outcome);
        }
        (EffectTarget::Chimera(chimera), Effect::AddTemporaryTrait { trait_id, duration }) => {
            add_temporary_trait(state, chimera, *trait_id, *duration, outcome);
        }
        _ => {}
    }
}

pub fn perform_work_action(
    state: &mut CombatState,
    chimera: ChimeraId,
    outcome: &mut RoundOutcome,
) {
    let round = state.round;
    let source_name = chimera_name(state, chimera);
    if !state
        .chimera(chimera)
        .map(|chimera| chimera.is_active)
        .unwrap_or(false)
    {
        return;
    }

    let Some(task) = front_task_id(state) else {
        outcome.push_log(round_line(
            round,
            format!("{source_name} found no unfinished task."),
        ));
        return;
    };

    let task_name = task_name(state, task);
    let Some(cost) = state.task(task).map(|task| task.progress.stamina_cost) else {
        return;
    };
    let Some(stamina) = state.chimera(chimera).map(|chimera| chimera.stats.stamina) else {
        return;
    };

    if stamina < cost {
        outcome.push_log(round_line(
            round,
            format!(
                "{source_name} left the field. Stamina {stamina} is lower than required cost {cost}."
            ),
        ));
        deactivate_chimera(state, chimera);
        return;
    }

    change_stamina(state, chimera, -cost, "consumed", outcome);
    resolve_trigger_for_chimera(state, chimera, Trigger::OnWork, outcome);

    if let Some(current_task) = front_task_id(state) {
        let efficiency = state
            .chimera(chimera)
            .map(|chimera| base_work_progress(chimera.stats.efficiency))
            .unwrap_or_default();
        advance_task_progress(
            state,
            current_task,
            efficiency,
            &format!("{source_name} work"),
            outcome,
        );
    }

    outcome.push_log(round_line(
        round,
        format!("{source_name} finished action on {task_name}."),
    ));

    resolve_trigger_for_chimera(state, chimera, Trigger::AfterWork, outcome);
    retire_if_unable_to_continue(state, chimera, outcome);
}

fn retire_if_unable_to_continue(
    state: &mut CombatState,
    chimera: ChimeraId,
    outcome: &mut RoundOutcome,
) {
    let Some(task) = front_task_id(state) else {
        return;
    };
    let Some(cost) = state.task(task).map(|task| task.progress.stamina_cost) else {
        return;
    };
    let Some(stamina) = state.chimera(chimera).map(|chimera| chimera.stats.stamina) else {
        return;
    };

    if stamina >= cost {
        return;
    }

    let name = chimera_name(state, chimera);
    deactivate_chimera(state, chimera);
    outcome.push_log(round_line(
        state.round,
        format!("{name} left the field after running out of usable stamina."),
    ));
}

fn deactivate_chimera(state: &mut CombatState, chimera: ChimeraId) {
    if let Some(chimera) = state.chimera_mut(chimera) {
        chimera.is_active = false;
    }
}

pub fn resolve_trigger_for_chimera(
    state: &mut CombatState,
    chimera: ChimeraId,
    trigger: Trigger,
    outcome: &mut RoundOutcome,
) {
    let trait_defs = trait_defs_for_chimera(state, chimera, trigger);
    let round = state.round;
    let source_name = chimera_name(state, chimera);

    for trait_def in trait_defs {
        let targets = select_targets(state, trait_def.selector, chimera);
        let target_names = targets
            .iter()
            .map(|target| match target {
                EffectTarget::Chimera(id) => chimera_name(state, *id),
                EffectTarget::Task(id) => task_name(state, *id),
                EffectTarget::Global => "Awoo Cookie Score".to_string(),
            })
            .collect::<Vec<_>>();

        let target_text = if target_names.is_empty() {
            "no valid target".to_string()
        } else {
            target_names.join(", ")
        };

        outcome.push_log(round_line(
            round,
            format!("{source_name} used {} on {target_text}.", trait_def.name),
        ));

        for target in targets {
            for effect in &trait_def.effects {
                apply_effect(state, chimera, target, effect, trait_def.name, outcome);
            }
        }
    }
}

pub fn resolve_trigger_for_all_chimeras(
    state: &mut CombatState,
    trigger: Trigger,
    outcome: &mut RoundOutcome,
) {
    for chimera in action_order(state) {
        resolve_trigger_for_chimera(state, chimera, trigger, outcome);
    }
}

pub fn expire_timed_effects(state: &mut CombatState, outcome: &mut RoundOutcome) {
    let round = state.round;
    let mut expired_efficiency = Vec::new();

    for (index, chimera) in state.chimeras.iter_mut().enumerate() {
        for bonus in &mut chimera.active_effects.efficiency_bonuses {
            if bonus.remaining_rounds > 0 {
                bonus.remaining_rounds -= 1;
            }
        }

        let mut removed_amount = 0;
        chimera.active_effects.efficiency_bonuses.retain(|bonus| {
            if bonus.remaining_rounds == 0 {
                removed_amount += bonus.amount;
                false
            } else {
                true
            }
        });

        if removed_amount != 0 {
            chimera.stats.efficiency -= removed_amount;
            expired_efficiency.push((ChimeraId(index), removed_amount));
        }

        for timed_trait in &mut chimera.active_effects.temporary_traits {
            if timed_trait.remaining_rounds > 0 {
                timed_trait.remaining_rounds -= 1;
            }
        }

        chimera
            .active_effects
            .temporary_traits
            .retain(|timed_trait| timed_trait.remaining_rounds > 0);
    }

    for (chimera, amount) in expired_efficiency {
        let name = chimera_name(state, chimera);
        outcome.push_log(round_line(
            round,
            format!("{name}'s temporary +{amount} efficiency expired."),
        ));
    }
}

fn check_work_end(state: &mut CombatState, outcome: &mut RoundOutcome) {
    if state.is_finished {
        return;
    }

    let all_completed = all_tasks_completed(state);
    let reached_max_round = state.round >= state.max_round;
    let no_work_candidates = next_work_chimera(state).is_none();

    if !all_completed && !reached_max_round && !no_work_candidates {
        return;
    }

    state.is_finished = true;
    let victory = all_completed || state.cookie_score >= state.target_cookie_score;

    outcome.push_event(CombatEvent::WorkEnded { victory });
    outcome.push_log("Work Finished!");
    outcome.push_log(format!(
        "Result: {}",
        if victory { "Victory" } else { "Defeat" }
    ));
    outcome.push_log(format!("Final Awoo Cookies: {}", state.cookie_score));
    outcome.push_log(format!(
        "Completed Tasks: {}/{}",
        state.completed_tasks,
        total_task_count(state)
    ));
}

fn trait_defs_for_chimera(
    state: &CombatState,
    chimera: ChimeraId,
    trigger: Trigger,
) -> Vec<TraitDef> {
    let mut ids = Vec::new();

    if let Some(chimera) = state.chimera(chimera) {
        if !chimera.is_active {
            return Vec::new();
        }

        ids.extend(chimera.traits.iter().copied());
        ids.extend(
            chimera
                .active_effects
                .temporary_traits
                .iter()
                .filter(|timed_trait| timed_trait.remaining_rounds > 0)
                .map(|timed_trait| timed_trait.trait_id),
        );
    }

    ids.into_iter()
        .filter_map(|id| state.trait_database.traits.get(&id))
        .filter(|trait_def| trait_def.trigger == trigger)
        .cloned()
        .collect()
}

fn all_task_targets(state: &CombatState) -> Vec<EffectTarget> {
    let mut tasks = state
        .tasks
        .iter()
        .enumerate()
        .map(|(index, task)| (TaskId(index), task.order))
        .collect::<Vec<_>>();
    tasks.sort_by_key(|(_, order)| *order);
    tasks
        .into_iter()
        .map(|(id, _)| EffectTarget::Task(id))
        .collect()
}

fn all_ally_targets(state: &CombatState, source: ChimeraId) -> Vec<EffectTarget> {
    let Some(source_team) = state.chimera(source).map(|chimera| chimera.team_id) else {
        return Vec::new();
    };

    let mut allies = state
        .chimeras
        .iter()
        .enumerate()
        .filter(|(index, chimera)| {
            chimera.is_active && *index != source.0 && chimera.team_id == source_team
        })
        .map(|(index, chimera)| (ChimeraId(index), chimera.slot))
        .collect::<Vec<_>>();
    allies.sort_by_key(|(_, slot)| *slot);
    allies
        .into_iter()
        .map(|(id, _)| EffectTarget::Chimera(id))
        .collect()
}

fn ally_by_slot_offset(state: &CombatState, source: ChimeraId, offset: i32) -> Vec<EffectTarget> {
    let Some(source) = state.chimera(source) else {
        return Vec::new();
    };

    let wanted_slot = source.slot as i32 + offset;
    if wanted_slot < 0 {
        return Vec::new();
    }

    state
        .chimeras
        .iter()
        .enumerate()
        .find(|(_, chimera)| {
            chimera.is_active
                && chimera.team_id == source.team_id
                && chimera.slot == wanted_slot as u32
        })
        .map(|(index, _)| vec![EffectTarget::Chimera(ChimeraId(index))])
        .unwrap_or_default()
}

fn stamina_ranked_ally(state: &CombatState, source: ChimeraId, lowest: bool) -> Vec<EffectTarget> {
    let Some(source_team) = state.chimera(source).map(|chimera| chimera.team_id) else {
        return Vec::new();
    };

    let mut allies = state
        .chimeras
        .iter()
        .enumerate()
        .filter(|(_, chimera)| chimera.is_active && chimera.team_id == source_team)
        .map(|(index, chimera)| (ChimeraId(index), chimera.stats.stamina))
        .collect::<Vec<_>>();

    if lowest {
        allies.sort_by_key(|(_, stamina)| *stamina);
    } else {
        allies.sort_by(|(_, left), (_, right)| right.cmp(left));
    }

    allies
        .first()
        .map(|(id, _)| vec![EffectTarget::Chimera(*id)])
        .unwrap_or_default()
}

fn efficiency_ranked_ally(state: &CombatState, source: ChimeraId) -> Vec<EffectTarget> {
    let Some(source_team) = state.chimera(source).map(|chimera| chimera.team_id) else {
        return Vec::new();
    };

    let mut allies = state
        .chimeras
        .iter()
        .enumerate()
        .filter(|(_, chimera)| chimera.is_active && chimera.team_id == source_team)
        .map(|(index, chimera)| (ChimeraId(index), chimera.stats.efficiency))
        .collect::<Vec<_>>();

    allies.sort_by(|(_, left), (_, right)| right.cmp(left));
    allies
        .first()
        .map(|(id, _)| vec![EffectTarget::Chimera(*id)])
        .unwrap_or_default()
}

fn gain_cookie(state: &mut CombatState, amount: i32, outcome: &mut RoundOutcome) {
    state.cookie_score += amount;
    outcome.push_event(CombatEvent::CookieGained { amount });
    outcome.push_log(round_line(
        state.round,
        format!("Gained {amount} Awoo Cookies from trait effect."),
    ));
}

fn add_efficiency(
    state: &mut CombatState,
    chimera: ChimeraId,
    amount: i32,
    duration: u32,
    outcome: &mut RoundOutcome,
) {
    let round = state.round;
    let name = chimera_name(state, chimera);
    let mut line = None;

    if let Some(chimera) = state.chimera_mut(chimera) {
        chimera.stats.efficiency += amount;
        chimera
            .active_effects
            .efficiency_bonuses
            .push(TimedEfficiencyBonus {
                amount,
                remaining_rounds: duration.max(1),
            });
        line = Some(format!(
            "{name} gained +{amount} efficiency for {duration} round(s). Efficiency: {}.",
            chimera.stats.efficiency
        ));
    }

    if let Some(line) = line {
        outcome.push_log(round_line(round, line));
    }
}

fn change_stamina(
    state: &mut CombatState,
    chimera: ChimeraId,
    amount: i32,
    verb: &str,
    outcome: &mut RoundOutcome,
) {
    let round = state.round;
    let name = chimera_name(state, chimera);
    let mut line = None;

    if let Some(chimera) = state.chimera_mut(chimera) {
        chimera.stats.stamina =
            clamp_stamina(chimera.stats.stamina + amount, chimera.stats.max_stamina);
        line = Some(format!(
            "{name} {verb} {} stamina. Stamina: {}/{}.",
            amount.abs(),
            chimera.stats.stamina,
            chimera.stats.max_stamina
        ));
    }

    if let Some(line) = line {
        outcome.push_log(round_line(round, line));
    }
}

fn add_temporary_trait(
    state: &mut CombatState,
    chimera: ChimeraId,
    trait_id: TraitId,
    duration: u32,
    outcome: &mut RoundOutcome,
) {
    let round = state.round;
    let name = chimera_name(state, chimera);

    if let Some(chimera) = state.chimera_mut(chimera) {
        chimera.active_effects.temporary_traits.push(TimedTrait {
            trait_id,
            remaining_rounds: duration.max(1),
        });
    }

    outcome.push_log(round_line(
        round,
        format!("{name} gained a temporary trait for {duration} round(s)."),
    ));
}
