use bevy::prelude::*;

use crate::combat::{
    component::{
        ActiveEffects, Chimera, Name, Stats, TaskOrder, TaskProgress, TeamId, TeamSlot, TraitList,
        WorkTask,
    },
    data::{Effect, TargetSelector},
    resolver::{
        action_order, advance_task_progress, front_task_entity, perform_work_action, select_targets,
    },
    resource::{TraitDatabase, WorkLogs, WorkStateData},
    target::EffectTarget,
};

fn test_world() -> World {
    let mut world = World::new();
    world.insert_resource(WorkStateData::default());
    world.insert_resource(WorkLogs::default());
    world.insert_resource(TraitDatabase::default());
    world
}

fn spawn_task(world: &mut World, order: u32, required: i32, completed: bool) -> Entity {
    world
        .spawn((
            WorkTask,
            TaskOrder(order),
            Name(format!("Task {order}")),
            TaskProgress {
                current: if completed { required } else { 0 },
                required,
                stamina_cost: 2,
                cookie_reward: 10,
                completed,
            },
        ))
        .id()
}

fn spawn_chimera(world: &mut World, name: &str, slot: u32, stamina: i32) -> Entity {
    world
        .spawn((
            Chimera,
            Name(name.to_string()),
            TeamId(1),
            TeamSlot(slot),
            Stats {
                max_stamina: 20,
                stamina,
                efficiency: 5,
                resilience: 0,
            },
            TraitList { traits: Vec::new() },
            ActiveEffects::default(),
        ))
        .id()
}

#[test]
fn target_selector_front_task_should_find_first_unfinished_task() {
    let mut world = test_world();
    let _completed = spawn_task(&mut world, 0, 10, true);
    let expected = spawn_task(&mut world, 1, 10, false);
    let source = spawn_chimera(&mut world, "Little Villain", 0, 20);

    let targets = select_targets(&mut world, TargetSelector::FrontTask, source);

    assert_eq!(targets, vec![EffectTarget::Task(expected)]);
    assert_eq!(front_task_entity(&mut world), Some(expected));
}

#[test]
fn action_order_should_start_from_rightmost_slot() {
    let mut world = test_world();
    let left = spawn_chimera(&mut world, "Little Villain", 0, 20);
    let middle = spawn_chimera(&mut world, "Healer", 3, 20);
    let right = spawn_chimera(&mut world, "Rat Race King", 4, 20);

    assert_eq!(action_order(&mut world), vec![right, middle, left]);
}

#[test]
fn advance_task_should_increase_task_progress() {
    let mut world = test_world();
    let task = spawn_task(&mut world, 0, 20, false);

    advance_task_progress(&mut world, task, 7, "test");

    let progress = world.get::<TaskProgress>(task).unwrap();
    assert_eq!(progress.current, 7);
    assert!(!progress.completed);
}

#[test]
fn task_should_complete_when_progress_reaches_required_value() {
    let mut world = test_world();
    let task = spawn_task(&mut world, 0, 10, false);

    advance_task_progress(&mut world, task, 10, "test");

    let progress = world.get::<TaskProgress>(task).unwrap();
    assert!(progress.completed);
}

#[test]
fn completed_task_should_grant_awoo_cookies() {
    let mut world = test_world();
    let task = spawn_task(&mut world, 0, 10, false);

    advance_task_progress(&mut world, task, 10, "test");

    assert_eq!(world.resource::<WorkStateData>().cookie_score, 10);
    assert_eq!(world.resource::<WorkStateData>().completed_tasks, 1);
}

#[test]
fn chimera_should_skip_action_when_stamina_is_not_enough() {
    let mut world = test_world();
    let chimera = spawn_chimera(&mut world, "Tired Worker", 0, 1);
    let task = spawn_task(&mut world, 0, 10, false);

    perform_work_action(&mut world, chimera);

    assert_eq!(world.get::<TaskProgress>(task).unwrap().current, 0);
    assert_eq!(world.get::<Stats>(chimera).unwrap().stamina, 1);
}

#[test]
fn add_efficiency_should_expire_after_duration() {
    let mut world = test_world();
    let chimera = spawn_chimera(&mut world, "Pressure Monster", 0, 20);
    let effect = Effect::AddEfficiency {
        amount: 2,
        duration: 1,
    };

    crate::combat::resolver::apply_effect(
        &mut world,
        chimera,
        EffectTarget::Chimera(chimera),
        &effect,
        "Pressure Boost",
    );

    assert_eq!(world.get::<Stats>(chimera).unwrap().efficiency, 7);

    crate::combat::resolver::expire_timed_effects(&mut world);

    assert_eq!(world.get::<Stats>(chimera).unwrap().efficiency, 5);
}
