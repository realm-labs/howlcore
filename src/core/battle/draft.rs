//! Draft and shop helpers for building a chimera battle lineup.

use crate::core::battle::{
    BattleAbilityId, BattleChimera, BattleRarity, BattleStats, BattleTeam, TeamSide,
};

pub const CHIMERA_PURCHASE_COST: i32 = 3;

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
    pub shop: Vec<BattleChimeraOffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DraftError {
    NotEnoughGold { cost: i32, available: i32 },
    InvalidOfferIndex { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurchaseOutcome {
    Added {
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
            shop: Vec::new(),
        }
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

        if let Some(existing) = self
            .team
            .chimeras
            .iter_mut()
            .find(|chimera| chimera.name == offer.name)
        {
            let level_before = existing.level;
            merge_duplicate(existing);
            return Ok(PurchaseOutcome::Merged {
                chimera_name: existing.name.clone(),
                level_before,
                level_after: existing.level,
            });
        }

        let chimera_name = offer.name.clone();
        let slot = next_back_slot(&self.team);
        self.team.chimeras.push(offer.into_chimera(slot));
        Ok(PurchaseOutcome::Added { chimera_name })
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
