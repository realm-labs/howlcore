//! Two-team chimera battle gameplay core.

pub mod data;
pub mod draft;
pub mod event;
pub mod model;
pub mod resolver;
pub mod run;

pub use data::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleEffect, BattleTargetSelector,
    BattleTrigger,
};
pub use draft::{
    BattleChimeraOffer, CHIMERA_PURCHASE_COST, DEFAULT_ACTIVE_TEAM_LIMIT, DraftError, DraftState,
    PurchaseOutcome,
};
pub use event::{BattleEvent, BattleOutcome};
pub use model::{
    BattleChimera, BattleChimeraId, BattleDefinition, BattleLeader, BattleLeaderEffect,
    BattleRarity, BattleRng, BattleRunConfig, BattleState, BattleStats, BattleTeam, TeamSide,
};
pub use run::{
    BATTLE_LOSS_HEALTH_DAMAGE, BATTLE_RUN_HEALTH, BATTLE_RUN_ROUNDS, BATTLE_SHOP_SIZE,
    BATTLE_STARTING_GOLD, BATTLE_WIN_GOLD_REWARD, BattleRunError, BattleRunPhase, BattleRunState,
    BattleRunStep,
};
