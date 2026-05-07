use crate::core::work::{
    ActiveEffects, Chimera, ChimeraId, CombatState, Effect, RoundOutcome, Stats, TargetSelector,
    TaskId, TaskProgress, TraitDatabase, WorkTask,
    resolver::{
        EffectTarget, action_order, advance_task_progress, expire_timed_effects, front_task_id,
        next_work_chimera, perform_work_action, select_targets,
    },
};

fn test_state() -> CombatState {
    CombatState {
        trait_database: TraitDatabase::default(),
        ..Default::default()
    }
}

fn add_task(state: &mut CombatState, order: u32, required: i32, completed: bool) -> TaskId {
    let id = TaskId(state.tasks.len());
    state.tasks.push(WorkTask {
        name: format!("Task {order}"),
        order,
        progress: TaskProgress {
            current: if completed { required } else { 0 },
            required,
            stamina_cost: 2,
            cookie_reward: 10,
            completed,
        },
    });
    id
}

fn add_chimera(state: &mut CombatState, name: &str, slot: u32, stamina: i32) -> ChimeraId {
    let id = ChimeraId(state.chimeras.len());
    state.chimeras.push(Chimera {
        name: name.to_string(),
        team_id: 1,
        slot,
        is_active: true,
        stats: Stats {
            max_stamina: 20,
            stamina,
            efficiency: 5,
            resilience: 0,
        },
        traits: Vec::new(),
        active_effects: ActiveEffects::default(),
    });
    id
}

#[test]
fn target_selector_front_task_should_find_first_unfinished_task() {
    let mut state = test_state();
    let _completed = add_task(&mut state, 0, 10, true);
    let expected = add_task(&mut state, 1, 10, false);
    let source = add_chimera(&mut state, "Little Villain", 0, 20);

    let targets = select_targets(&state, TargetSelector::FrontTask, source);

    assert_eq!(targets, vec![EffectTarget::Task(expected)]);
    assert_eq!(front_task_id(&state), Some(expected));
}

#[test]
fn action_order_should_start_from_rightmost_slot() {
    let mut state = test_state();
    let left = add_chimera(&mut state, "Little Villain", 0, 20);
    let middle = add_chimera(&mut state, "Healer", 3, 20);
    let right = add_chimera(&mut state, "Rat Race King", 4, 20);

    assert_eq!(action_order(&state), vec![right, middle, left]);
}

#[test]
fn next_work_chimera_should_keep_rightmost_active_until_it_leaves() {
    let mut state = test_state();
    add_task(&mut state, 0, 100, false);
    let _left = add_chimera(&mut state, "Left Worker", 0, 20);
    let right = add_chimera(&mut state, "Right Worker", 4, 4);

    assert_eq!(next_work_chimera(&state), Some(right));

    let outcome = state.step_round();
    let work_actions = outcome
        .events
        .iter()
        .filter_map(|event| match event {
            crate::core::work::CombatEvent::WorkActionRequested { chimera } => Some(*chimera),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(work_actions[0], right);
    assert_eq!(work_actions[1], right);
    assert!(!state.chimera(right).unwrap().is_active);
}

#[test]
fn advance_task_should_increase_task_progress() {
    let mut state = test_state();
    let task = add_task(&mut state, 0, 20, false);
    let mut outcome = RoundOutcome::default();

    advance_task_progress(&mut state, task, 7, "test", &mut outcome);

    let progress = &state.task(task).unwrap().progress;
    assert_eq!(progress.current, 7);
    assert!(!progress.completed);
}

#[test]
fn task_should_complete_when_progress_reaches_required_value() {
    let mut state = test_state();
    let task = add_task(&mut state, 0, 10, false);
    let mut outcome = RoundOutcome::default();

    advance_task_progress(&mut state, task, 10, "test", &mut outcome);

    let progress = &state.task(task).unwrap().progress;
    assert!(progress.completed);
}

#[test]
fn completed_task_should_grant_awoo_cookies() {
    let mut state = test_state();
    let task = add_task(&mut state, 0, 10, false);
    let mut outcome = RoundOutcome::default();

    advance_task_progress(&mut state, task, 10, "test", &mut outcome);

    assert_eq!(state.cookie_score, 10);
    assert_eq!(state.completed_tasks, 1);
}

#[test]
fn chimera_should_skip_action_when_stamina_is_not_enough() {
    let mut state = test_state();
    let chimera = add_chimera(&mut state, "Tired Worker", 0, 1);
    let task = add_task(&mut state, 0, 10, false);
    let mut outcome = RoundOutcome::default();

    perform_work_action(&mut state, chimera, &mut outcome);

    assert_eq!(state.task(task).unwrap().progress.current, 0);
    assert_eq!(state.chimera(chimera).unwrap().stats.stamina, 1);
    assert!(!state.chimera(chimera).unwrap().is_active);
}

#[test]
fn add_efficiency_should_expire_after_duration() {
    let mut state = test_state();
    let chimera = add_chimera(&mut state, "Pressure Monster", 0, 20);
    let effect = Effect::AddEfficiency {
        amount: 2,
        duration: 1,
    };
    let mut outcome = RoundOutcome::default();

    crate::core::work::resolver::apply_effect(
        &mut state,
        chimera,
        EffectTarget::Chimera(chimera),
        &effect,
        "Pressure Boost",
        &mut outcome,
    );

    assert_eq!(state.chimera(chimera).unwrap().stats.efficiency, 7);

    expire_timed_effects(&mut state, &mut outcome);

    assert_eq!(state.chimera(chimera).unwrap().stats.efficiency, 5);
}
