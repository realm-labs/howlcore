//! Multi-week run loop for Work Assignment mode.

use crate::core::work::{
    Chimera, CombatState, RoundOutcome, StageDefinition, WorkOvertimeConfig, WorkReviewPeriod,
    WorkTask, resolver::all_tasks_completed,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkRunPhase {
    Review,
    Overtime,
    Complete,
}

#[derive(Debug, Clone)]
pub struct WorkRunState {
    pub phase: WorkRunPhase,
    pub stage_name: String,
    pub assignment: CombatState,
    pub review_periods: Vec<WorkReviewPeriod>,
    pub current_period: usize,
    pub weeks_elapsed: u32,
    pub current_rank: u32,
    pub total_cookies: i32,
    pub overtime: Option<WorkOvertimeConfig>,
    pub overtime_cycle: u32,
    pub overtime_cookies: i32,
    base_chimeras: Vec<Chimera>,
}

impl WorkRunState {
    pub fn from_stage(stage: StageDefinition) -> Self {
        let first_assignment = if let Some(period) = stage.run.review_periods.first() {
            assignment_from_parts(
                stage.chimeras.clone(),
                period.max_round,
                period.required_cookie_score,
                period.tasks.clone(),
                stage.trait_database.clone(),
            )
        } else {
            CombatState::from_stage(stage.clone())
        };

        let phase = if stage.run.review_periods.is_empty() {
            if stage.run.overtime.is_some() {
                WorkRunPhase::Overtime
            } else {
                WorkRunPhase::Review
            }
        } else {
            WorkRunPhase::Review
        };

        let mut run = Self {
            phase,
            stage_name: stage.name.clone(),
            assignment: first_assignment,
            review_periods: stage.run.review_periods,
            current_period: 0,
            weeks_elapsed: 0,
            current_rank: stage.run.starting_rank,
            total_cookies: 0,
            overtime: stage.run.overtime,
            overtime_cycle: 0,
            overtime_cookies: 0,
            base_chimeras: stage.chimeras,
        };

        if run.review_periods.is_empty() && run.overtime.is_some() {
            run.start_overtime_cycle(None);
        }

        run
    }

    pub fn step(&mut self) -> RoundOutcome {
        let mut outcome = RoundOutcome::default();

        if self.phase == WorkRunPhase::Complete {
            return outcome;
        }

        if self.assignment.is_finished {
            self.resolve_finished_assignment(&mut outcome);
            return outcome;
        }

        outcome = self.assignment.step_round();
        if self.assignment.is_finished {
            self.resolve_finished_assignment(&mut outcome);
        }
        outcome
    }

    pub fn current_review_period(&self) -> Option<&WorkReviewPeriod> {
        self.review_periods.get(self.current_period)
    }

    fn resolve_finished_assignment(&mut self, outcome: &mut RoundOutcome) {
        match self.phase {
            WorkRunPhase::Review => self.resolve_review_period(outcome),
            WorkRunPhase::Overtime => self.resolve_overtime_cycle(outcome),
            WorkRunPhase::Complete => {}
        }
    }

    fn resolve_review_period(&mut self, outcome: &mut RoundOutcome) {
        let Some(period) = self.current_review_period().cloned() else {
            self.unlock_overtime_or_complete(outcome);
            return;
        };

        self.weeks_elapsed += 1;
        self.total_cookies += self.assignment.cookie_score;
        let promoted = self.assignment.cookie_score >= period.required_cookie_score;
        if promoted {
            self.current_rank = period.target_rank;
            self.current_period += 1;
        }

        outcome.push_log(format!(
            "Review finished: {} cookies this week, {} total.",
            self.assignment.cookie_score, self.total_cookies
        ));
        if promoted {
            outcome.push_log(format!(
                "Ranking Board: promoted to rank {} after {}.",
                self.current_rank, period.name
            ));
        } else {
            outcome.push_log(format!(
                "Ranking Board: rank {} held. Need {} cookies for {}.",
                self.current_rank, period.required_cookie_score, period.name
            ));
        }

        if self.current_period >= self.review_periods.len() {
            self.unlock_overtime_or_complete(outcome);
        } else {
            self.start_review_period();
        }
    }

    fn resolve_overtime_cycle(&mut self, outcome: &mut RoundOutcome) {
        self.overtime_cookies += self.assignment.cookie_score;
        outcome.push_log(format!(
            "Overtime cycle {} finished: {} cookies, {} overtime total.",
            self.overtime_cycle, self.assignment.cookie_score, self.overtime_cookies
        ));

        if all_tasks_completed(&self.assignment) && self.has_usable_overtime_stamina() {
            let chimeras = self
                .assignment
                .chimeras
                .iter()
                .cloned()
                .map(|mut chimera| {
                    chimera.is_active = chimera.stats.stamina > 0;
                    chimera
                })
                .collect::<Vec<_>>();
            self.start_overtime_cycle(Some(chimeras));
        } else {
            self.phase = WorkRunPhase::Complete;
            outcome.push_log(format!(
                "Overtime ended after {} cycle(s). Final overtime cookies: {}.",
                self.overtime_cycle, self.overtime_cookies
            ));
        }
    }

    fn unlock_overtime_or_complete(&mut self, outcome: &mut RoundOutcome) {
        if self.overtime.is_some() {
            self.phase = WorkRunPhase::Overtime;
            self.overtime_cycle = 0;
            self.overtime_cookies = 0;
            outcome.push_log("Ranking Board: rank 1 reached. Overtime Mode unlocked.");
            self.start_overtime_cycle(None);
        } else {
            self.phase = WorkRunPhase::Complete;
            outcome.push_log(format!(
                "Work run complete. Final rank: {}. Total cookies: {}.",
                self.current_rank, self.total_cookies
            ));
        }
    }

    fn start_review_period(&mut self) {
        let Some(period) = self.current_review_period().cloned() else {
            return;
        };

        self.assignment.max_round = period.max_round;
        self.assignment.target_cookie_score = period.required_cookie_score;
        self.assignment.cookie_score = 0;
        self.assignment.completed_tasks = 0;
        self.assignment.round = 0;
        self.assignment.is_finished = false;
        self.assignment.chimeras = self.base_chimeras.clone();
        self.assignment.tasks = period.tasks;
    }

    fn start_overtime_cycle(&mut self, chimeras: Option<Vec<Chimera>>) {
        let Some(overtime) = self.overtime.clone() else {
            return;
        };

        self.phase = WorkRunPhase::Overtime;
        self.overtime_cycle += 1;
        let growth = self.overtime_cycle.saturating_sub(1);
        let target_cookie_score = overtime
            .tasks
            .iter()
            .map(|task| task.progress.cookie_reward + overtime.cookie_reward_growth * growth as i32)
            .sum();
        self.assignment.max_round = overtime.max_round;
        self.assignment.target_cookie_score = target_cookie_score;
        self.assignment.cookie_score = 0;
        self.assignment.completed_tasks = 0;
        self.assignment.round = 0;
        self.assignment.is_finished = false;
        self.assignment.chimeras = chimeras.unwrap_or_else(|| self.base_chimeras.clone());
        self.assignment.tasks = scale_overtime_tasks(&overtime, self.overtime_cycle);
    }

    fn has_usable_overtime_stamina(&self) -> bool {
        self.assignment.chimeras.iter().any(|chimera| {
            self.overtime
                .as_ref()
                .and_then(|overtime| {
                    overtime
                        .tasks
                        .iter()
                        .map(|task| task.progress.stamina_cost)
                        .min()
                })
                .is_some_and(|minimum_cost| chimera.stats.stamina >= minimum_cost)
        })
    }
}

fn assignment_from_parts(
    chimeras: Vec<Chimera>,
    max_round: u32,
    target_cookie_score: i32,
    tasks: Vec<WorkTask>,
    trait_database: crate::core::work::TraitDatabase,
) -> CombatState {
    CombatState {
        round: 0,
        max_round,
        cookie_score: 0,
        completed_tasks: 0,
        target_cookie_score,
        is_finished: false,
        chimeras,
        tasks,
        trait_database,
    }
}

fn scale_overtime_tasks(config: &WorkOvertimeConfig, cycle: u32) -> Vec<WorkTask> {
    let growth = cycle.saturating_sub(1);
    let stamina_growth = growth
        .checked_div(config.stamina_cost_growth_every)
        .unwrap_or_default();

    config
        .tasks
        .iter()
        .cloned()
        .map(|mut task| {
            task.progress.current = 0;
            task.progress.completed = false;
            task.progress.required += config.required_progress_growth * growth as i32;
            task.progress.stamina_cost += stamina_growth as i32;
            task.progress.cookie_reward += config.cookie_reward_growth * growth as i32;
            task
        })
        .collect()
}
