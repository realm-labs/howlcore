//! Events emitted by the two-team chimera battle flow.

use crate::core::battle::{
    data::BattleAbilityId,
    model::{BattleChimeraId, TeamSide},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleEvent {
    TurnStarted {
        turn: u32,
    },
    BasicAttack {
        attacker: BattleChimeraId,
        target: BattleChimeraId,
        damage: i32,
    },
    DamageDealt {
        target: BattleChimeraId,
        amount: i32,
    },
    DamageReduced {
        target: BattleChimeraId,
        amount: i32,
    },
    HpRestored {
        target: BattleChimeraId,
        amount: i32,
    },
    AttackChanged {
        target: BattleChimeraId,
        amount: i32,
    },
    PositionSwapped {
        first: BattleChimeraId,
        second: BattleChimeraId,
    },
    ChimeraQueued {
        side: TeamSide,
        name: String,
    },
    ChimeraSummoned {
        chimera: BattleChimeraId,
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
    pub logs: Vec<String>,
}

impl BattleOutcome {
    pub fn push_log(&mut self, line: impl Into<String>) {
        self.logs.push(line.into());
    }

    pub fn push_event(&mut self, event: BattleEvent) {
        self.events.push(event);
    }
}
