//! State model for two-team chimera battles.

use crate::core::battle::{
    data::BattleAbilityDatabase, data::BattleAbilityId, event::BattleOutcome, resolver,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamSide {
    Challenger,
    Defender,
}

impl TeamSide {
    pub fn opponent(self) -> Self {
        match self {
            Self::Challenger => Self::Defender,
            Self::Defender => Self::Challenger,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BattleChimeraId {
    pub side: TeamSide,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleStats {
    pub attack: i32,
    pub max_hp: i32,
    pub hp: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleChimera {
    pub name: String,
    pub slot: u32,
    pub stats: BattleStats,
    pub abilities: Vec<BattleAbilityId>,
}

impl BattleChimera {
    pub fn is_alive(&self) -> bool {
        self.stats.hp > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleTeam {
    pub side: TeamSide,
    pub name: String,
    pub chimeras: Vec<BattleChimera>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleDefinition {
    pub name: String,
    pub max_turn: u32,
    pub challenger: BattleTeam,
    pub defender: BattleTeam,
    pub ability_database: BattleAbilityDatabase,
    pub initial_logs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleState {
    pub name: String,
    pub turn: u32,
    pub max_turn: u32,
    pub is_finished: bool,
    pub winner: Option<TeamSide>,
    pub challenger: BattleTeam,
    pub defender: BattleTeam,
    pub ability_database: BattleAbilityDatabase,
}

impl BattleState {
    pub fn from_definition(definition: BattleDefinition) -> Self {
        Self {
            name: definition.name,
            turn: 0,
            max_turn: definition.max_turn,
            is_finished: false,
            winner: None,
            challenger: definition.challenger,
            defender: definition.defender,
            ability_database: definition.ability_database,
        }
    }

    pub fn step_turn(&mut self) -> BattleOutcome {
        resolver::step_turn(self)
    }

    pub fn team(&self, side: TeamSide) -> &BattleTeam {
        match side {
            TeamSide::Challenger => &self.challenger,
            TeamSide::Defender => &self.defender,
        }
    }

    pub fn team_mut(&mut self, side: TeamSide) -> &mut BattleTeam {
        match side {
            TeamSide::Challenger => &mut self.challenger,
            TeamSide::Defender => &mut self.defender,
        }
    }

    pub fn chimera(&self, id: BattleChimeraId) -> Option<&BattleChimera> {
        self.team(id.side).chimeras.get(id.index)
    }

    pub fn chimera_mut(&mut self, id: BattleChimeraId) -> Option<&mut BattleChimera> {
        self.team_mut(id.side).chimeras.get_mut(id.index)
    }
}
