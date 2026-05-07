//! Single-team task-assignment gameplay core.

pub mod data;
pub mod event;
pub mod formula;
pub mod log;
pub mod model;
pub mod resolver;

pub use data::{Effect, TargetSelector, TraitDef, TraitId, Trigger};
pub use event::{CombatEvent, RoundOutcome};
pub use model::{
    ActiveEffects, Chimera, ChimeraId, CombatState, StageDefinition, Stats, TaskId, TaskProgress,
    TimedEfficiencyBonus, TimedTrait, TraitDatabase, WorkTask,
};
