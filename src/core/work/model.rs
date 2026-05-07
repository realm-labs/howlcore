//! State model for the UI-independent work-battle core.

use std::collections::HashMap;

use crate::core::work::{
    data::{TraitDef, TraitId},
    event::RoundOutcome,
    resolver,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChimeraId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub usize);

#[derive(Debug, Clone)]
pub struct Stats {
    pub max_stamina: i32,
    pub stamina: i32,
    pub efficiency: i32,
    pub resilience: i32,
}

#[derive(Debug, Clone)]
pub struct TimedEfficiencyBonus {
    pub amount: i32,
    pub remaining_rounds: u32,
}

#[derive(Debug, Clone)]
pub struct TimedTrait {
    pub trait_id: TraitId,
    pub remaining_rounds: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ActiveEffects {
    pub efficiency_bonuses: Vec<TimedEfficiencyBonus>,
    pub temporary_traits: Vec<TimedTrait>,
}

#[derive(Debug, Clone)]
pub struct Chimera {
    pub name: String,
    pub team_id: u32,
    pub slot: u32,
    pub stats: Stats,
    pub traits: Vec<TraitId>,
    pub active_effects: ActiveEffects,
}

#[derive(Debug, Clone)]
pub struct TaskProgress {
    pub current: i32,
    pub required: i32,
    pub stamina_cost: i32,
    pub cookie_reward: i32,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct WorkTask {
    pub name: String,
    pub order: u32,
    pub progress: TaskProgress,
}

#[derive(Debug, Default, Clone)]
pub struct TraitDatabase {
    pub traits: HashMap<TraitId, TraitDef>,
}

#[derive(Debug, Clone)]
pub struct StageDefinition {
    pub name: String,
    pub max_round: u32,
    pub target_cookie_score: i32,
    pub chimeras: Vec<Chimera>,
    pub tasks: Vec<WorkTask>,
    pub trait_database: TraitDatabase,
    pub initial_logs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CombatState {
    pub round: u32,
    pub max_round: u32,
    pub cookie_score: i32,
    pub completed_tasks: u32,
    pub target_cookie_score: i32,
    pub is_finished: bool,
    pub chimeras: Vec<Chimera>,
    pub tasks: Vec<WorkTask>,
    pub trait_database: TraitDatabase,
}

impl CombatState {
    pub fn from_stage(stage: StageDefinition) -> Self {
        Self {
            round: 0,
            max_round: stage.max_round,
            cookie_score: 0,
            completed_tasks: 0,
            target_cookie_score: stage.target_cookie_score,
            is_finished: false,
            chimeras: stage.chimeras,
            tasks: stage.tasks,
            trait_database: stage.trait_database,
        }
    }

    pub fn step_round(&mut self) -> RoundOutcome {
        resolver::step_round(self)
    }

    pub fn chimera(&self, id: ChimeraId) -> Option<&Chimera> {
        self.chimeras.get(id.0)
    }

    pub fn chimera_mut(&mut self, id: ChimeraId) -> Option<&mut Chimera> {
        self.chimeras.get_mut(id.0)
    }

    pub fn task(&self, id: TaskId) -> Option<&WorkTask> {
        self.tasks.get(id.0)
    }

    pub fn task_mut(&mut self, id: TaskId) -> Option<&mut WorkTask> {
        self.tasks.get_mut(id.0)
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            round: 0,
            max_round: 5,
            cookie_score: 0,
            completed_tasks: 0,
            target_cookie_score: 30,
            is_finished: false,
            chimeras: Vec::new(),
            tasks: Vec::new(),
            trait_database: TraitDatabase::default(),
        }
    }
}
