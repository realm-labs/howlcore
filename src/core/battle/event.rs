//! Events emitted by the two-team chimera battle flow.

use crate::core::battle::{
    data::BattleAbilityId,
    model::{BattleChimera, BattleChimeraId, TeamSide},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleEvent {
    BattleStarted,
    TurnStarted {
        turn: u32,
    },
    BasicAttack {
        attacker: BattleChimeraId,
        target: BattleChimeraId,
        damage: i32,
    },
    DamageApplied {
        target: BattleChimeraId,
        amount: i32,
        hp_before: i32,
        hp_after: i32,
    },
    DamageReduced {
        target: BattleChimeraId,
        amount: i32,
        damage_before: i32,
        damage_after: i32,
    },
    HpRestored {
        target: BattleChimeraId,
        amount: i32,
        hp_before: i32,
        hp_after: i32,
    },
    AttackChanged {
        target: BattleChimeraId,
        amount: i32,
        attack_before: i32,
        attack_after: i32,
    },
    PositionSwapped {
        first: BattleChimeraId,
        second: BattleChimeraId,
    },
    ChimeraQueued {
        side: TeamSide,
        chimera: BattleChimera,
    },
    ChimeraSummoned {
        chimera: BattleChimeraId,
        state: BattleChimera,
    },
    AbilityTriggered {
        source: BattleChimeraId,
        ability: BattleAbilityId,
    },
    ChanceRolled {
        percent: u32,
        roll: u32,
        success: bool,
    },
    ChimeraKnockedDown {
        chimera: BattleChimeraId,
    },
    BattleEnded {
        winner: Option<TeamSide>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BattleOutcome {
    pub events: Vec<BattleEvent>,
}

impl BattleOutcome {
    pub fn push_event(&mut self, event: BattleEvent) {
        self.events.push(event);
    }
}
