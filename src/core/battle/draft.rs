//! Draft and shop helpers for building a chimera battle lineup.

use crate::core::battle::{
    BattleAbilityId, BattleChimera, BattleRarity, BattleStats, BattleTeam, TeamSide,
};

pub const CHIMERA_PURCHASE_COST: i32 = 3;
pub const DEFAULT_ACTIVE_TEAM_LIMIT: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleChimeraOffer {
    pub name: String,
    pub rarity: BattleRarity,
    pub attack: i32,
    pub hp: i32,
    pub abilities: Vec<BattleAbilityId>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftState {
    pub gold: i32,
    pub team: BattleTeam,
    pub bench: Vec<BattleChimera>,
    pub active_team_limit: usize,
    pub shop: Vec<BattleChimeraOffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftError {
    NotEnoughGold { cost: i32, available: i32 },
    InvalidOfferIndex { index: usize },
    InvalidTeamIndex { index: usize },
    InvalidBenchIndex { index: usize },
    ActiveLineupFull { limit: usize },
    ActiveLineupTooSmall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurchaseOutcome {
    Added {
        chimera_name: String,
    },
    AddedToBench {
        chimera_name: String,
    },
    Merged {
        chimera_name: String,
        level_before: u32,
        level_after: u32,
    },
}

impl BattleChimeraOffer {
    pub fn new(
        name: impl Into<String>,
        rarity: BattleRarity,
        attack: i32,
        hp: i32,
        abilities: Vec<BattleAbilityId>,
    ) -> Self {
        Self {
            name: name.into(),
            rarity,
            attack,
            hp,
            abilities,
            tags: Vec::new(),
        }
    }
}

impl DraftState {
    pub fn new(gold: i32, side: TeamSide, team_name: impl Into<String>) -> Self {
        Self {
            gold,
            team: BattleTeam {
                side,
                name: team_name.into(),
                chimeras: Vec::new(),
                summon_queue: Vec::new(),
            },
            bench: Vec::new(),
            active_team_limit: DEFAULT_ACTIVE_TEAM_LIMIT,
            shop: Vec::new(),
        }
    }

    pub fn with_active_team_limit(mut self, active_team_limit: usize) -> Self {
        self.active_team_limit = active_team_limit.max(1);
        self
    }

    pub fn purchase(&mut self, index: usize) -> Result<PurchaseOutcome, DraftError> {
        if index >= self.shop.len() {
            return Err(DraftError::InvalidOfferIndex { index });
        }

        if self.gold < CHIMERA_PURCHASE_COST {
            return Err(DraftError::NotEnoughGold {
                cost: CHIMERA_PURCHASE_COST,
                available: self.gold,
            });
        }

        self.gold -= CHIMERA_PURCHASE_COST;
        let offer = self.shop.remove(index);

        if let Some(existing) = self.find_chimera_mut(&offer.name) {
            let level_before = existing.level;
            merge_duplicate(existing);
            return Ok(PurchaseOutcome::Merged {
                chimera_name: existing.name.clone(),
                level_before,
                level_after: existing.level,
            });
        }

        let chimera_name = offer.name.clone();
        if self.team.chimeras.len() < self.active_team_limit {
            let slot = next_back_slot(&self.team);
            self.team.chimeras.push(offer.into_chimera(slot));
            Ok(PurchaseOutcome::Added { chimera_name })
        } else {
            self.bench.push(offer.into_chimera(0));
            Ok(PurchaseOutcome::AddedToBench { chimera_name })
        }
    }

    pub fn swap_active_positions(
        &mut self,
        left_position: usize,
        right_position: usize,
    ) -> Result<(), DraftError> {
        let active_indices = sorted_active_indices(&self.team);
        let Some(&left_index) = active_indices.get(left_position) else {
            return Err(DraftError::InvalidTeamIndex {
                index: left_position,
            });
        };
        let Some(&right_index) = active_indices.get(right_position) else {
            return Err(DraftError::InvalidTeamIndex {
                index: right_position,
            });
        };

        let left_slot = self.team.chimeras[left_index].slot;
        self.team.chimeras[left_index].slot = self.team.chimeras[right_index].slot;
        self.team.chimeras[right_index].slot = left_slot;
        Ok(())
    }

    pub fn send_active_to_bench(&mut self, position: usize) -> Result<String, DraftError> {
        if self.team.chimeras.len() <= 1 {
            return Err(DraftError::ActiveLineupTooSmall);
        }

        let active_indices = sorted_active_indices(&self.team);
        let Some(&team_index) = active_indices.get(position) else {
            return Err(DraftError::InvalidTeamIndex { index: position });
        };

        let mut chimera = self.team.chimeras.remove(team_index);
        let chimera_name = chimera.name.clone();
        chimera.slot = 0;
        self.bench.push(chimera);
        normalize_active_slots(&mut self.team);
        Ok(chimera_name)
    }

    pub fn deploy_from_bench(&mut self, bench_index: usize) -> Result<String, DraftError> {
        if self.team.chimeras.len() >= self.active_team_limit {
            return Err(DraftError::ActiveLineupFull {
                limit: self.active_team_limit,
            });
        }
        if bench_index >= self.bench.len() {
            return Err(DraftError::InvalidBenchIndex { index: bench_index });
        }

        let mut chimera = self.bench.remove(bench_index);
        let chimera_name = chimera.name.clone();
        chimera.slot = next_back_slot(&self.team);
        self.team.chimeras.push(chimera);
        Ok(chimera_name)
    }

    fn find_chimera_mut(&mut self, name: &str) -> Option<&mut BattleChimera> {
        if let Some(chimera) = self
            .team
            .chimeras
            .iter_mut()
            .find(|chimera| chimera.name == name)
        {
            return Some(chimera);
        }

        self.bench.iter_mut().find(|chimera| chimera.name == name)
    }
}

impl BattleChimeraOffer {
    fn into_chimera(self, slot: u32) -> BattleChimera {
        BattleChimera {
            name: self.name,
            slot,
            level: 1,
            experience: 0,
            rarity: self.rarity,
            tags: self.tags,
            stats: BattleStats {
                attack: self.attack,
                max_hp: self.hp,
                hp: self.hp,
            },
            abilities: self.abilities,
        }
    }
}

fn merge_duplicate(chimera: &mut BattleChimera) {
    chimera.stats.attack += 1;
    chimera.stats.max_hp += 1;
    chimera.stats.hp += 1;
    chimera.experience += 1;
    apply_level_ups(chimera);
}

fn apply_level_ups(chimera: &mut BattleChimera) {
    while chimera.level < 3 {
        let cost = match chimera.level {
            1 => 2,
            2 => 3,
            _ => break,
        };

        if chimera.experience < cost {
            break;
        }

        chimera.experience -= cost;
        chimera.level += 1;
    }
}

fn next_back_slot(team: &BattleTeam) -> u32 {
    team.chimeras
        .iter()
        .map(|chimera| chimera.slot)
        .max()
        .map(|slot| slot + 1)
        .unwrap_or_default()
}

fn sorted_active_indices(team: &BattleTeam) -> Vec<usize> {
    let mut indices = (0..team.chimeras.len()).collect::<Vec<_>>();
    indices.sort_by_key(|index| team.chimeras[*index].slot);
    indices
}

fn normalize_active_slots(team: &mut BattleTeam) {
    for (slot, index) in sorted_active_indices(team).into_iter().enumerate() {
        team.chimeras[index].slot = slot as u32;
    }
}
