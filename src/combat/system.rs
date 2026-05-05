//! Bevy systems that run the work-battle loop.

use std::{
    io::Read,
    sync::{Arc, atomic::AtomicU32},
    time::Duration,
};

use bevy::prelude::*;
use crossterm::event::{self, Event, KeyCode};

use crate::combat::{
    event::{RoundStarted, WorkActionRequested, WorkEnded},
    log::round_line,
    resolver::{
        action_order, all_tasks_completed, expire_timed_effects, perform_work_action,
        resolve_trigger_for_chimera, total_task_count,
    },
    resource::{ActionQueue, RoundFlow, TerminalInput, WorkLogs, WorkStateData},
};

use crate::combat::data::Trigger;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CombatSet {
    Input,
    StartRound,
    BuildActionQueue,
    ExecuteAction,
    ResolveEffect,
    EndRound,
    CheckEnd,
    Log,
}

/// Starts a background stdin reader so piped spaces can also advance rounds.
pub fn setup_terminal_input_system(mut commands: Commands) {
    let counter = Arc::new(AtomicU32::new(0));
    let input = TerminalInput::new(counter.clone());

    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buffer = [0_u8; 1];
        loop {
            match stdin.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if buffer[0] == b' ' || buffer[0] == b'\n' {
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            }
        }
    });

    commands.insert_resource(input);
}

/// Reads terminal Space input and requests exactly one full work round.
pub fn read_space_input_system(
    input: Option<Res<TerminalInput>>,
    mut flow: ResMut<RoundFlow>,
    state: Res<WorkStateData>,
) {
    if state.is_finished {
        return;
    }

    let mut pressed = input
        .as_ref()
        .map(|input| input.take_space_presses())
        .unwrap_or_default();

    while event::poll(Duration::ZERO).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            if key.code == KeyCode::Char(' ') {
                pressed += 1;
            }
        }
    }

    if pressed > 0 {
        flow.queued_rounds += pressed;
    }

    if !flow.round_requested && flow.queued_rounds > 0 {
        flow.queued_rounds -= 1;
        flow.round_requested = true;
    }
}

/// Starts a round, logs it, and resolves RoundStart traits such as Pressure Boost.
pub fn start_round_system(world: &mut World) {
    let should_start = {
        let flow = world.resource::<RoundFlow>();
        let state = world.resource::<WorkStateData>();
        flow.round_requested && !state.is_finished
    };

    if !should_start {
        world.resource_mut::<RoundFlow>().round_started_this_update = false;
        return;
    }

    {
        let mut flow = world.resource_mut::<RoundFlow>();
        flow.round_requested = false;
        flow.round_started_this_update = true;
    }

    let round = {
        let mut state = world.resource_mut::<WorkStateData>();
        state.round += 1;
        state.round
    };

    world
        .resource_mut::<WorkLogs>()
        .0
        .push(round_line(round, "Round started."));
    world
        .resource_mut::<Events<RoundStarted>>()
        .send(RoundStarted { round });

    for chimera in action_order(world) {
        resolve_trigger_for_chimera(world, chimera, Trigger::RoundStart);
    }
}

/// Builds the action queue from rightmost slot to leftmost slot.
pub fn build_action_queue_system(world: &mut World) {
    if !world.resource::<RoundFlow>().round_started_this_update {
        return;
    }

    let order = action_order(world);
    let names = order
        .iter()
        .map(|entity| crate::combat::resolver::entity_name(world, *entity))
        .collect::<Vec<_>>();
    let round = world.resource::<WorkStateData>().round;

    world.resource_mut::<ActionQueue>().0 = order;
    world.resource_mut::<WorkLogs>().0.push(round_line(
        round,
        format!("Action order: {}.", names.join(" -> ")),
    ));
}

/// Executes every queued chimera action for the current round.
pub fn execute_action_queue_system(world: &mut World) {
    if !world.resource::<RoundFlow>().round_started_this_update {
        return;
    }

    let queue = std::mem::take(&mut world.resource_mut::<ActionQueue>().0);

    for chimera in queue {
        world
            .resource_mut::<Events<WorkActionRequested>>()
            .send(WorkActionRequested { chimera });
        perform_work_action(world, chimera);

        if all_tasks_completed(world) {
            break;
        }
    }
}

/// Resolves end-of-round trait hooks and expires temporary effects.
pub fn end_round_system(world: &mut World) {
    if !world.resource::<RoundFlow>().round_started_this_update {
        return;
    }

    for chimera in action_order(world) {
        resolve_trigger_for_chimera(world, chimera, Trigger::RoundEnd);
    }

    expire_timed_effects(world);

    let round = world.resource::<WorkStateData>().round;
    let cookie_score = world.resource::<WorkStateData>().cookie_score;
    world.resource_mut::<WorkLogs>().0.push(round_line(
        round,
        format!("Round ended. Awoo Cookies: {cookie_score}."),
    ));

    world.resource_mut::<RoundFlow>().round_started_this_update = false;
}

/// Checks victory or defeat after each completed round.
pub fn check_work_end_system(world: &mut World) {
    if world.resource::<WorkStateData>().is_finished {
        return;
    }

    let all_completed = all_tasks_completed(world);
    let reached_max_round = {
        let state = world.resource::<WorkStateData>();
        state.round >= state.max_round
    };

    if !all_completed && !reached_max_round {
        return;
    }

    let task_count = total_task_count(world);
    let (victory, cookie_score, completed_tasks) = {
        let mut state = world.resource_mut::<WorkStateData>();
        state.is_finished = true;
        let victory = all_completed || state.cookie_score >= state.target_cookie_score;
        (victory, state.cookie_score, state.completed_tasks)
    };

    world
        .resource_mut::<Events<WorkEnded>>()
        .send(WorkEnded { victory });

    world
        .resource_mut::<WorkLogs>()
        .0
        .push("Work Finished!".to_string());
    world.resource_mut::<WorkLogs>().0.push(format!(
        "Result: {}",
        if victory { "Victory" } else { "Defeat" }
    ));
    world
        .resource_mut::<WorkLogs>()
        .0
        .push(format!("Final Awoo Cookies: {cookie_score}"));
    world
        .resource_mut::<WorkLogs>()
        .0
        .push(format!("Completed Tasks: {completed_tasks}/{task_count}"));
}

/// Prints and clears all accumulated work logs.
pub fn flush_work_logs_system(mut logs: ResMut<WorkLogs>) {
    for line in logs.0.drain(..) {
        println!("{line}");
    }
}
