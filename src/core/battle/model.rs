//! State model for two-team chimera battles.

use crate::core::battle::{
    data::BattleAbilityDatabase, data::BattleAbilityId, draft::BattleChimeraOffer,
    event::BattleOutcome, resolver,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BattleRarity {
    White,
    Blue,
    Purple,
    Gold,
    Prismatic,
}

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
    pub level: u32,
    pub experience: u32,
    pub rarity: BattleRarity,
    pub tags: Vec<String>,
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
    pub summon_queue: Vec<BattleChimera>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleDefinition {
    pub name: String,
    pub max_turn: u32,
    pub challenger: BattleTeam,
    pub defender: BattleTeam,
    pub ability_database: BattleAbilityDatabase,
    pub rng_seed: u64,
    pub leader: Option<BattleLeader>,
    pub run: BattleRunConfig,
    pub initial_logs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleLeader {
    pub name: String,
    pub effects: Vec<BattleLeaderEffect>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleLeaderEffect {
    AddStartingGold { amount: i32 },
    AddRunHealth { amount: i32 },
    AddWinGoldReward { amount: i32 },
    AddTeamStats { attack: i32, hp: i32 },
    AddShopOfferStats { attack: i32, hp: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleRunConfig {
    pub starting_gold: i32,
    pub shop_size: usize,
    pub active_team_limit: usize,
    pub health: i32,
    pub loss_health_damage: i32,
    pub win_gold_reward: i32,
    pub opponent_rounds: Vec<BattleOpponentRound>,
    pub shop_pool: Vec<BattleChimeraOffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOpponentRound {
    pub name: String,
    pub defender: BattleTeam,
    pub win_gold_reward: i32,
    pub loss_health_damage: i32,
    pub is_boss: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleState {
    pub name: String,
    pub turn: u32,
    pub has_started: bool,
    pub max_turn: u32,
    pub is_finished: bool,
    pub winner: Option<TeamSide>,
    pub challenger: BattleTeam,
    pub defender: BattleTeam,
    pub ability_database: BattleAbilityDatabase,
    pub rng: BattleRng,
}

impl BattleState {
    pub fn from_definition(definition: BattleDefinition) -> Self {
        Self {
            name: definition.name,
            turn: 0,
            has_started: false,
            max_turn: definition.max_turn,
            is_finished: false,
            winner: None,
            challenger: definition.challenger,
            defender: definition.defender,
            ability_database: definition.ability_database,
            rng: BattleRng::new(definition.rng_seed),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleRng {
    state: u64,
}

impl BattleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    pub fn roll_percent(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as u32 % 100) + 1
    }
}
