//! Two-team chimera battle gameplay core.

pub mod data;
pub mod event;
pub mod model;
pub mod resolver;

pub use data::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleEffect, BattleTargetSelector,
    BattleTrigger,
};
pub use event::{BattleEvent, BattleOutcome};
pub use model::{
    BattleChimera, BattleChimeraId, BattleDefinition, BattleState, BattleStats, BattleTeam,
    TeamSide,
};
