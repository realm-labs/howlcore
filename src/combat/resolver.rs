//! Pure gameplay resolution helpers.
//!
//! The Bevy systems call these functions, and tests can exercise them directly.

use bevy::prelude::*;

use crate::combat::{
    component::{
        ActiveEffects, Chimera, Name, Stats, TaskOrder, TaskProgress, TeamId, TeamSlot,
        TimedEfficiencyBonus, TimedTrait, TraitList, WorkTask,
    },
    data::{Effect, TargetSelector, TraitDef, TraitId, Trigger},
    event::{CookieGained, EffectApplied, TaskCompleted},
    formula::{base_work_progress, clamp_stamina},
    log::round_line,
    resource::{TraitDatabase, WorkLogs, WorkStateData},
    target::EffectTarget,
};

pub fn entity_name(world: &World, entity: Entity) -> String {
    world
        .get::<Name>(entity)
        .map(|name| name.0.clone())
        .unwrap_or_else(|| format!("{entity:?}"))
}

pub fn total_task_count(world: &mut World) -> usize {
    let mut query = world.query_filtered::<Entity, With<WorkTask>>();
    query.iter(world).count()
}

pub fn all_tasks_completed(world: &mut World) -> bool {
    let mut query = world.query_filtered::<&TaskProgress, With<WorkTask>>();
    query.iter(world).all(|progress| progress.completed)
}

pub fn front_task_entity(world: &mut World) -> Option<Entity> {
    let mut query = world.query_filtered::<(Entity, &TaskOrder, &TaskProgress), With<WorkTask>>();
    let mut tasks = query
        .iter(world)
        .filter(|(_, _, progress)| !progress.completed)
        .map(|(entity, order, _)| (entity, order.0))
        .collect::<Vec<_>>();

    tasks.sort_by_key(|(_, order)| *order);
    tasks.first().map(|(entity, _)| *entity)
}

pub fn action_order(world: &mut World) -> Vec<Entity> {
    let mut query = world.query_filtered::<(Entity, &TeamSlot), With<Chimera>>();
    let mut chimeras = query
        .iter(world)
        .map(|(entity, slot)| (entity, slot.0))
        .collect::<Vec<_>>();

    chimeras.sort_by(|(_, left_slot), (_, right_slot)| right_slot.cmp(left_slot));
    chimeras.into_iter().map(|(entity, _)| entity).collect()
}

pub fn select_targets(
    world: &mut World,
    selector: TargetSelector,
    source: Entity,
) -> Vec<EffectTarget> {
    match selector {
        TargetSelector::SelfChimera => vec![EffectTarget::Chimera(source)],
        TargetSelector::FrontTask => front_task_entity(world)
            .map(EffectTarget::Task)
            .into_iter()
            .collect(),
        TargetSelector::AllTasks => all_task_targets(world),
        TargetSelector::PreviousAlly => ally_by_slot_offset(world, source, -1),
        TargetSelector::NextAlly => ally_by_slot_offset(world, source, 1),
        TargetSelector::AdjacentAllies => {
            let mut targets = ally_by_slot_offset(world, source, -1);
            targets.extend(ally_by_slot_offset(world, source, 1));
            targets
        }
        TargetSelector::AllAllies => all_ally_targets(world, source),
        TargetSelector::LowestStaminaAlly => stamina_ranked_ally(world, source, true),
        TargetSelector::HighestEfficiencyAlly => efficiency_ranked_ally(world, source),
    }
}

pub fn advance_task_progress(world: &mut World, task: Entity, amount: i32, reason: &str) {
    let round = world.resource::<WorkStateData>().round;
    let task_name = entity_name(world, task);
    let mut completed_reward = None;
    let mut progress_line = None;

    if let Some(mut progress) = world.entity_mut(task).get_mut::<TaskProgress>() {
        if progress.completed {
            return;
        }

        progress.current += amount.max(0);
        progress_line = Some(round_line(
            round,
            format!(
                "{task_name} progress +{}. Progress: {}/{}. ({reason})",
                amount.max(0),
                progress.current,
                progress.required
            ),
        ));

        if progress.current >= progress.required {
            progress.completed = true;
            completed_reward = Some(progress.cookie_reward);
        }
    }

    if let Some(line) = progress_line {
        world.resource_mut::<WorkLogs>().0.push(line);
    }

    if let Some(cookie_reward) = completed_reward {
        {
            let mut state = world.resource_mut::<WorkStateData>();
            state.completed_tasks += 1;
            state.cookie_score += cookie_reward;
        }

        if let Some(mut events) = world.get_resource_mut::<Events<CookieGained>>() {
            events.send(CookieGained {
                amount: cookie_reward,
            });
        }

        if let Some(mut events) = world.get_resource_mut::<Events<TaskCompleted>>() {
            events.send(TaskCompleted {
                task,
                cookie_reward,
            });
        }

        world.resource_mut::<WorkLogs>().0.push(round_line(
            round,
            format!("{task_name} completed. Gained {cookie_reward} Awoo Cookies."),
        ));

        resolve_trigger_for_all_chimeras(world, Trigger::TaskCompleted);
    }
}

pub fn apply_effect(
    world: &mut World,
    source: Entity,
    target: EffectTarget,
    effect: &Effect,
    trait_name: &str,
) {
    if let Some(mut events) = world.get_resource_mut::<Events<EffectApplied>>() {
        events.send(EffectApplied {
            description: format!("{trait_name} applied {effect:?}"),
        });
    }

    match (target, effect) {
        (EffectTarget::Task(task), Effect::AdvanceTask { amount }) => {
            advance_task_progress(world, task, *amount, trait_name);
        }
        (EffectTarget::Task(task), Effect::AdvanceTaskByEfficiency { bonus }) => {
            let efficiency = world
                .get::<Stats>(source)
                .map(|stats| stats.efficiency)
                .unwrap_or_default();
            advance_task_progress(world, task, efficiency + bonus, trait_name);
        }
        (EffectTarget::Global, Effect::GainCookie { amount }) => gain_cookie(world, *amount),
        (EffectTarget::Chimera(chimera), Effect::GainCookie { amount }) => {
            let _ = chimera;
            gain_cookie(world, *amount);
        }
        (EffectTarget::Chimera(chimera), Effect::AddEfficiency { amount, duration }) => {
            add_efficiency(world, chimera, *amount, *duration);
        }
        (EffectTarget::Chimera(chimera), Effect::AddStamina { amount }) => {
            change_stamina(world, chimera, *amount, "gained");
        }
        (EffectTarget::Chimera(chimera), Effect::ConsumeStamina { amount }) => {
            change_stamina(world, chimera, -*amount, "consumed");
        }
        (EffectTarget::Chimera(chimera), Effect::RestoreStamina { amount }) => {
            change_stamina(world, chimera, *amount, "restored");
        }
        (EffectTarget::Chimera(chimera), Effect::AddTemporaryTrait { trait_id, duration }) => {
            add_temporary_trait(world, chimera, trait_id.clone(), *duration);
        }
        _ => {}
    }
}

pub fn perform_work_action(world: &mut World, chimera: Entity) {
    let round = world.resource::<WorkStateData>().round;
    let chimera_name = entity_name(world, chimera);
    let Some(task) = front_task_entity(world) else {
        world.resource_mut::<WorkLogs>().0.push(round_line(
            round,
            format!("{chimera_name} found no unfinished task."),
        ));
        return;
    };

    let task_name = entity_name(world, task);
    let Some(cost) = world
        .get::<TaskProgress>(task)
        .map(|progress| progress.stamina_cost)
    else {
        return;
    };
    let Some(stamina) = world.get::<Stats>(chimera).map(|stats| stats.stamina) else {
        return;
    };

    if stamina < cost {
        world.resource_mut::<WorkLogs>().0.push(round_line(
            round,
            format!(
                "{chimera_name} skipped work. Stamina {stamina} is lower than required cost {cost}."
            ),
        ));
        return;
    }

    change_stamina(world, chimera, -cost, "consumed");
    resolve_trigger_for_chimera(world, chimera, Trigger::OnWork);

    if let Some(current_task) = front_task_entity(world) {
        let efficiency = world
            .get::<Stats>(chimera)
            .map(|stats| base_work_progress(stats.efficiency))
            .unwrap_or_default();
        advance_task_progress(
            world,
            current_task,
            efficiency,
            &format!("{chimera_name} work"),
        );
    }

    world.resource_mut::<WorkLogs>().0.push(round_line(
        round,
        format!("{chimera_name} finished action on {task_name}."),
    ));

    resolve_trigger_for_chimera(world, chimera, Trigger::AfterWork);
}

pub fn resolve_trigger_for_chimera(world: &mut World, chimera: Entity, trigger: Trigger) {
    let trait_defs = trait_defs_for_chimera(world, chimera, trigger);
    let round = world.resource::<WorkStateData>().round;
    let chimera_name = entity_name(world, chimera);

    for trait_def in trait_defs {
        let targets = select_targets(world, trait_def.selector, chimera);
        let target_names = targets
            .iter()
            .map(|target| match target {
                EffectTarget::Chimera(entity) | EffectTarget::Task(entity) => {
                    entity_name(world, *entity)
                }
                EffectTarget::Global => "Awoo Cookie Score".to_string(),
            })
            .collect::<Vec<_>>();

        let target_text = if target_names.is_empty() {
            "no valid target".to_string()
        } else {
            target_names.join(", ")
        };

        world.resource_mut::<WorkLogs>().0.push(round_line(
            round,
            format!("{chimera_name} used {} on {target_text}.", trait_def.name),
        ));

        for target in targets {
            for effect in &trait_def.effects {
                apply_effect(world, chimera, target, effect, trait_def.name);
            }
        }
    }
}

pub fn resolve_trigger_for_all_chimeras(world: &mut World, trigger: Trigger) {
    for chimera in action_order(world) {
        resolve_trigger_for_chimera(world, chimera, trigger);
    }
}

pub fn expire_timed_effects(world: &mut World) {
    let round = world.resource::<WorkStateData>().round;
    let mut query =
        world.query_filtered::<(Entity, &mut Stats, &mut ActiveEffects), With<Chimera>>();
    let mut expired_efficiency = Vec::new();

    for (entity, mut stats, mut active) in query.iter_mut(world) {
        for bonus in &mut active.efficiency_bonuses {
            if bonus.remaining_rounds > 0 {
                bonus.remaining_rounds -= 1;
            }
        }

        let mut removed_amount = 0;
        active.efficiency_bonuses.retain(|bonus| {
            if bonus.remaining_rounds == 0 {
                removed_amount += bonus.amount;
                false
            } else {
                true
            }
        });

        if removed_amount != 0 {
            stats.efficiency -= removed_amount;
            expired_efficiency.push((entity, removed_amount));
        }

        for timed_trait in &mut active.temporary_traits {
            if timed_trait.remaining_rounds > 0 {
                timed_trait.remaining_rounds -= 1;
            }
        }

        active
            .temporary_traits
            .retain(|timed_trait| timed_trait.remaining_rounds > 0);
    }

    for (entity, amount) in expired_efficiency {
        let name = entity_name(world, entity);
        world.resource_mut::<WorkLogs>().0.push(round_line(
            round,
            format!("{name}'s temporary +{amount} efficiency expired."),
        ));
    }
}

fn trait_defs_for_chimera(world: &World, chimera: Entity, trigger: Trigger) -> Vec<TraitDef> {
    let mut ids = Vec::new();

    if let Some(trait_list) = world.get::<TraitList>(chimera) {
        ids.extend(trait_list.traits.iter().cloned());
    }

    if let Some(active) = world.get::<ActiveEffects>(chimera) {
        ids.extend(
            active
                .temporary_traits
                .iter()
                .filter(|timed_trait| timed_trait.remaining_rounds > 0)
                .map(|timed_trait| timed_trait.trait_id.clone()),
        );
    }

    let trait_database = world.resource::<TraitDatabase>();
    ids.into_iter()
        .filter_map(|id| trait_database.traits.get(&id))
        .filter(|trait_def| trait_def.trigger == trigger)
        .cloned()
        .collect()
}

fn all_task_targets(world: &mut World) -> Vec<EffectTarget> {
    let mut query = world.query_filtered::<(Entity, &TaskOrder), With<WorkTask>>();
    let mut tasks = query
        .iter(world)
        .map(|(entity, order)| (entity, order.0))
        .collect::<Vec<_>>();
    tasks.sort_by_key(|(_, order)| *order);
    tasks
        .into_iter()
        .map(|(entity, _)| EffectTarget::Task(entity))
        .collect()
}

fn all_ally_targets(world: &mut World, source: Entity) -> Vec<EffectTarget> {
    let Some(source_team) = world.get::<TeamId>(source).copied() else {
        return Vec::new();
    };

    let mut query = world.query_filtered::<(Entity, &TeamId, &TeamSlot), With<Chimera>>();
    let mut allies = query
        .iter(world)
        .filter(|(entity, team, _)| *entity != source && **team == source_team)
        .map(|(entity, _, slot)| (entity, slot.0))
        .collect::<Vec<_>>();
    allies.sort_by_key(|(_, slot)| *slot);
    allies
        .into_iter()
        .map(|(entity, _)| EffectTarget::Chimera(entity))
        .collect()
}

fn ally_by_slot_offset(world: &mut World, source: Entity, offset: i32) -> Vec<EffectTarget> {
    let Some(source_team) = world.get::<TeamId>(source).copied() else {
        return Vec::new();
    };
    let Some(source_slot) = world.get::<TeamSlot>(source).copied() else {
        return Vec::new();
    };

    let wanted_slot = source_slot.0 as i32 + offset;
    if wanted_slot < 0 {
        return Vec::new();
    }

    let mut query = world.query_filtered::<(Entity, &TeamId, &TeamSlot), With<Chimera>>();
    query
        .iter(world)
        .find(|(_, team, slot)| **team == source_team && slot.0 == wanted_slot as u32)
        .map(|(entity, _, _)| vec![EffectTarget::Chimera(entity)])
        .unwrap_or_default()
}

fn stamina_ranked_ally(world: &mut World, source: Entity, lowest: bool) -> Vec<EffectTarget> {
    let Some(source_team) = world.get::<TeamId>(source).copied() else {
        return Vec::new();
    };

    let mut query = world.query_filtered::<(Entity, &TeamId, &Stats), With<Chimera>>();
    let mut allies = query
        .iter(world)
        .filter(|(_, team, _)| **team == source_team)
        .map(|(entity, _, stats)| (entity, stats.stamina))
        .collect::<Vec<_>>();

    if lowest {
        allies.sort_by_key(|(_, stamina)| *stamina);
    } else {
        allies.sort_by(|(_, left), (_, right)| right.cmp(left));
    }

    allies
        .first()
        .map(|(entity, _)| vec![EffectTarget::Chimera(*entity)])
        .unwrap_or_default()
}

fn efficiency_ranked_ally(world: &mut World, source: Entity) -> Vec<EffectTarget> {
    let Some(source_team) = world.get::<TeamId>(source).copied() else {
        return Vec::new();
    };

    let mut query = world.query_filtered::<(Entity, &TeamId, &Stats), With<Chimera>>();
    let mut allies = query
        .iter(world)
        .filter(|(_, team, _)| **team == source_team)
        .map(|(entity, _, stats)| (entity, stats.efficiency))
        .collect::<Vec<_>>();

    allies.sort_by(|(_, left), (_, right)| right.cmp(left));
    allies
        .first()
        .map(|(entity, _)| vec![EffectTarget::Chimera(*entity)])
        .unwrap_or_default()
}

fn gain_cookie(world: &mut World, amount: i32) {
    let round = world.resource::<WorkStateData>().round;
    world.resource_mut::<WorkStateData>().cookie_score += amount;
    if let Some(mut events) = world.get_resource_mut::<Events<CookieGained>>() {
        events.send(CookieGained { amount });
    }
    world.resource_mut::<WorkLogs>().0.push(round_line(
        round,
        format!("Gained {amount} Awoo Cookies from trait effect."),
    ));
}

fn add_efficiency(world: &mut World, chimera: Entity, amount: i32, duration: u32) {
    let round = world.resource::<WorkStateData>().round;
    let name = entity_name(world, chimera);
    let mut line = None;

    if let Some(mut stats) = world.entity_mut(chimera).get_mut::<Stats>() {
        stats.efficiency += amount;
        line = Some(format!(
            "{name} gained +{amount} efficiency for {duration} round(s). Efficiency: {}.",
            stats.efficiency
        ));
    }

    if let Some(mut active) = world.entity_mut(chimera).get_mut::<ActiveEffects>() {
        active.efficiency_bonuses.push(TimedEfficiencyBonus {
            amount,
            remaining_rounds: duration.max(1),
        });
    }

    if let Some(line) = line {
        world
            .resource_mut::<WorkLogs>()
            .0
            .push(round_line(round, line));
    }
}

fn change_stamina(world: &mut World, chimera: Entity, amount: i32, verb: &str) {
    let round = world.resource::<WorkStateData>().round;
    let name = entity_name(world, chimera);
    let mut line = None;

    if let Some(mut stats) = world.entity_mut(chimera).get_mut::<Stats>() {
        stats.stamina = clamp_stamina(stats.stamina + amount, stats.max_stamina);
        line = Some(format!(
            "{name} {verb} {} stamina. Stamina: {}/{}.",
            amount.abs(),
            stats.stamina,
            stats.max_stamina
        ));
    }

    if let Some(line) = line {
        world
            .resource_mut::<WorkLogs>()
            .0
            .push(round_line(round, line));
    }
}

fn add_temporary_trait(world: &mut World, chimera: Entity, trait_id: TraitId, duration: u32) {
    let round = world.resource::<WorkStateData>().round;
    let name = entity_name(world, chimera);

    if let Some(mut active) = world.entity_mut(chimera).get_mut::<ActiveEffects>() {
        active.temporary_traits.push(TimedTrait {
            trait_id,
            remaining_rounds: duration.max(1),
        });
    }

    world.resource_mut::<WorkLogs>().0.push(round_line(
        round,
        format!("{name} gained a temporary trait for {duration} round(s)."),
    ));
}
