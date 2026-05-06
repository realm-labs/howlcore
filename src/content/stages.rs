//! Test stage content for the first work-battle prototype.

use crate::{
    content::chimeras::{test_chimeras, test_trait_database},
    core::{StageDefinition, TaskProgress, WorkTask},
};

/// Creates the test team and Job-Skipping Guard Practice stage.
pub fn test_stage() -> StageDefinition {
    StageDefinition {
        name: "Job-Skipping Guard Practice".to_string(),
        max_round: 5,
        target_cookie_score: 30,
        chimeras: test_chimeras(),
        tasks: vec![
            task(0, "Sort Documents", 18, 2, 10),
            task(1, "Clean Garden", 26, 3, 15),
            task(2, "Deliver Packages", 35, 4, 20),
        ],
        trait_database: test_trait_database(),
        initial_logs: vec![
            "Stage: Job-Skipping Guard Practice".to_string(),
            "Press Space to advance one work round.".to_string(),
        ],
    }
}

fn task(
    order: u32,
    name: &'static str,
    required_progress: i32,
    stamina_cost: i32,
    cookie_reward: i32,
) -> WorkTask {
    WorkTask {
        name: name.to_string(),
        order,
        progress: TaskProgress {
            current: 0,
            required: required_progress,
            stamina_cost,
            cookie_reward,
            completed: false,
        },
    }
}
