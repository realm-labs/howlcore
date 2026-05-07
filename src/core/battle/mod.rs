//! Two-team chimera battle gameplay core.

pub mod data;
pub mod draft;
pub mod event;
pub mod model;
pub mod resolver;

pub use data::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleEffect, BattleTargetSelector,
    BattleTrigger,
};
pub use draft::{
    BattleChimeraOffer, CHIMERA_PURCHASE_COST, DraftError, DraftState, PurchaseOutcome,
};
pub use event::{BattleEvent, BattleOutcome};
pub use model::{
    BattleChimera, BattleChimeraId, BattleDefinition, BattleRarity, BattleRng, BattleState,
    BattleStats, BattleTeam, TeamSide,
};
