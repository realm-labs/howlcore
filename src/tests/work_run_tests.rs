use crate::core::work::{
    ActiveEffects, Chimera, StageDefinition, Stats, TaskProgress, TraitDatabase,
    WorkOvertimeConfig, WorkReviewPeriod, WorkRunConfig, WorkRunPhase, WorkRunState, WorkTask,
};

fn chimera(stamina: i32, efficiency: i32) -> Chimera {
    Chimera {
        name: "Worker".to_string(),
        team_id: 1,
        slot: 0,
        is_active: true,
        stats: Stats {
            max_stamina: stamina,
            stamina,
            efficiency,
            resilience: 0,
        },
        traits: Vec::new(),
        active_effects: ActiveEffects::default(),
    }
}

fn task(name: &str, required: i32, cost: i32, reward: i32) -> WorkTask {
    WorkTask {
        name: name.to_string(),
        order: 0,
        progress: TaskProgress {
            current: 0,
            required,
            stamina_cost: cost,
            cookie_reward: reward,
            completed: false,
        },
    }
}

fn stage(run: WorkRunConfig) -> StageDefinition {
    StageDefinition {
        name: "Test Stage".to_string(),
        max_round: 3,
        target_cookie_score: 10,
        chimeras: vec![chimera(8, 8)],
        tasks: vec![task("Default", 4, 1, 10)],
        run,
        trait_database: TraitDatabase::default(),
        initial_logs: Vec::new(),
    }
}

#[test]
fn review_period_should_promote_rank_and_unlock_overtime() {
    let run_config = WorkRunConfig {
        starting_rank: 2,
        review_periods: vec![WorkReviewPeriod {
            name: "Rank 1 Review".to_string(),
            target_rank: 1,
            required_cookie_score: 10,
            max_round: 3,
            tasks: vec![task("Review", 4, 1, 10)],
        }],
        overtime: Some(WorkOvertimeConfig {
            max_round: 3,
            required_progress_growth: 2,
            stamina_cost_growth_every: 2,
            cookie_reward_growth: 3,
            tasks: vec![task("Overtime", 4, 1, 10)],
        }),
    };
    let mut run = WorkRunState::from_stage(stage(run_config));

    let outcome = run.step();

    assert_eq!(run.current_rank, 1);
    assert_eq!(run.phase, WorkRunPhase::Overtime);
    assert_eq!(run.overtime_cycle, 1);
    assert_eq!(run.assignment.tasks[0].name, "Overtime");
    assert!(
        outcome
            .logs
            .iter()
            .any(|line| line.contains("Overtime Mode unlocked"))
    );
}

#[test]
fn failed_review_period_should_retry_same_target() {
    let run_config = WorkRunConfig {
        starting_rank: 3,
        review_periods: vec![WorkReviewPeriod {
            name: "Promotion Review".to_string(),
            target_rank: 2,
            required_cookie_score: 20,
            max_round: 1,
            tasks: vec![task("Too Hard", 99, 1, 10)],
        }],
        overtime: None,
    };
    let mut run = WorkRunState::from_stage(stage(run_config));

    let _ = run.step();

    assert_eq!(run.current_rank, 3);
    assert_eq!(run.current_period, 0);
    assert_eq!(run.phase, WorkRunPhase::Review);
    assert_eq!(run.weeks_elapsed, 1);
    assert_eq!(run.assignment.round, 0);
}

#[test]
fn overtime_should_carry_stamina_between_cleared_cycles() {
    let run_config = WorkRunConfig {
        starting_rank: 1,
        review_periods: Vec::new(),
        overtime: Some(WorkOvertimeConfig {
            max_round: 3,
            required_progress_growth: 0,
            stamina_cost_growth_every: 0,
            cookie_reward_growth: 0,
            tasks: vec![task("Overtime", 4, 2, 10)],
        }),
    };
    let mut custom_stage = stage(run_config);
    custom_stage.chimeras = vec![chimera(5, 8)];
    let mut run = WorkRunState::from_stage(custom_stage);

    let _ = run.step();

    assert_eq!(run.phase, WorkRunPhase::Overtime);
    assert_eq!(run.overtime_cycle, 2);
    assert_eq!(run.overtime_cookies, 10);
    assert_eq!(run.assignment.chimeras[0].stats.stamina, 3);
}
