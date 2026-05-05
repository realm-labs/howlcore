//! Test stage content for the first work-battle prototype.

use bevy::prelude::*;

use crate::{
    combat::{
        component::{Name, TaskOrder, TaskProgress, WorkTask},
        resource::{TraitDatabase, WorkLogs, WorkStateData},
    },
    content::chimeras::{register_test_traits, spawn_test_chimeras},
};

/// Creates the test team and Job-Skipping Guard Practice stage.
pub fn setup_test_stage_system(
    mut commands: Commands,
    mut trait_database: ResMut<TraitDatabase>,
    mut state: ResMut<WorkStateData>,
    mut logs: ResMut<WorkLogs>,
) {
    register_test_traits(&mut trait_database);
    spawn_test_chimeras(&mut commands);

    state.round = 0;
    state.max_round = 5;
    state.cookie_score = 0;
    state.completed_tasks = 0;
    state.target_cookie_score = 30;
    state.is_finished = false;

    spawn_task(&mut commands, 0, "Sort Documents", 18, 2, 10);
    spawn_task(&mut commands, 1, "Clean Garden", 26, 3, 15);
    spawn_task(&mut commands, 2, "Deliver Packages", 35, 4, 20);

    logs.0
        .push("Stage: Job-Skipping Guard Practice".to_string());
    logs.0
        .push("Press Space to advance one work round.".to_string());
}

fn spawn_task(
    commands: &mut Commands,
    order: u32,
    name: &'static str,
    required_progress: i32,
    stamina_cost: i32,
    cookie_reward: i32,
) {
    commands.spawn((
        WorkTask,
        TaskOrder(order),
        Name(name.to_string()),
        TaskProgress {
            current: 0,
            required: required_progress,
            stamina_cost,
            cookie_reward,
            completed: false,
        },
    ));
}
