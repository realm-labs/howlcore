//! Test stage content for the first work-battle prototype.

use crate::{
    content::chimeras::{ALPHA_COORDINATION, ALPHA_MOMENTUM, test_chimeras, test_trait_database},
    core::work::{
        StageDefinition, TaskProgress, WorkAlphaConfig, WorkOvertimeConfig, WorkReviewPeriod,
        WorkRunConfig, WorkTask,
    },
};

/// Creates the test team and Job-Skipping Guard Practice stage.
pub fn test_stage() -> StageDefinition {
    let opening_tasks = vec![
        task(0, "Sort Documents", 18, 2, 10),
        task(1, "Clean Garden", 26, 3, 15),
        task(2, "Deliver Packages", 35, 4, 20),
    ];
    let promotion_tasks = vec![
        task(0, "Prepare Garden Booth", 24, 2, 12),
        task(1, "Audit Cookie Ledgers", 32, 3, 18),
        task(2, "Deliver Sponsor Letters", 44, 4, 24),
    ];
    let rank_one_tasks = vec![
        task(0, "Review Alpha Reports", 30, 3, 16),
        task(1, "Repair Supply Route", 42, 4, 24),
        task(2, "Host Final Inspection", 54, 5, 32),
    ];

    StageDefinition {
        name: "Job-Skipping Guard Practice".to_string(),
        max_round: 5,
        target_cookie_score: 30,
        chimeras: test_chimeras(),
        tasks: opening_tasks.clone(),
        run: WorkRunConfig {
            starting_rank: 4,
            review_periods: vec![
                WorkReviewPeriod {
                    name: "Opening Review".to_string(),
                    target_rank: 3,
                    required_cookie_score: 30,
                    max_round: 5,
                    tasks: opening_tasks,
                },
                WorkReviewPeriod {
                    name: "Promotion Review".to_string(),
                    target_rank: 2,
                    required_cookie_score: 45,
                    max_round: 5,
                    tasks: promotion_tasks,
                },
                WorkReviewPeriod {
                    name: "Rank 1 Review".to_string(),
                    target_rank: 1,
                    required_cookie_score: 60,
                    max_round: 5,
                    tasks: rank_one_tasks.clone(),
                },
            ],
            overtime: Some(WorkOvertimeConfig {
                max_round: 5,
                required_progress_growth: 8,
                stamina_cost_growth_every: 2,
                cookie_reward_growth: 4,
                tasks: rank_one_tasks,
            }),
            alpha_options: vec![
                WorkAlphaConfig {
                    name: "Field Coordinator".to_string(),
                    chimera_name: "Healer".to_string(),
                    trait_id: ALPHA_COORDINATION,
                },
                WorkAlphaConfig {
                    name: "Momentum Captain".to_string(),
                    chimera_name: "Rat Race King".to_string(),
                    trait_id: ALPHA_MOMENTUM,
                },
            ],
        },
        trait_database: test_trait_database(),
        initial_logs: vec![
            "Stage: Job-Skipping Guard Practice".to_string(),
            "Press Space to advance work weeks toward Rank 1 and Overtime Mode.".to_string(),
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
