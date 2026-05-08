//! Draft-to-battle run loop for Chimera Battle mode.

use crate::core::battle::{
    BattleAbilityDatabase, BattleAbilityId, BattleChimeraOffer, BattleDefinition,
    BattleEquipmentOffer, BattleLeader, BattleLeaderEffect, BattleOpponentRound, BattleOutcome,
    BattleRarity, BattleRng, BattleRunConfig, BattleRunReward, BattleShopItem, BattleState,
    BattleTeam, DEFAULT_ACTIVE_TEAM_LIMIT, DraftState, TeamSide,
};

pub const BATTLE_WIN_GOLD_REWARD: i32 = 3;
pub const BATTLE_STARTING_GOLD: i32 = 6;
pub const BATTLE_SHOP_SIZE: usize = 3;
pub const BATTLE_RUN_HEALTH: i32 = 3;
pub const BATTLE_LOSS_HEALTH_DAMAGE: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleRunPhase {
    Draft,
    Battle,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleRunError {
    BattleAlreadyActive,
    EmptyChallengerTeam,
    NoDefenderAvailable,
    InvalidShopPhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleRunStep {
    StartedBattle,
    AdvancedBattle,
    BattleResolved { winner: Option<TeamSide> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleRunResult {
    Victory,
    Defeat,
}

#[derive(Debug, Clone)]
pub struct BattleRunState {
    pub phase: BattleRunPhase,
    pub draft: DraftState,
    pub opponents: Vec<BattleOpponentRound>,
    pub battle: Option<BattleState>,
    pub ability_database: BattleAbilityDatabase,
    pub leader: Option<BattleLeader>,
    pub max_turn: u32,
    pub rng_seed: u64,
    pub battle_index: usize,
    pub health: i32,
    pub max_health: i32,
    pub loss_health_damage: i32,
    pub shop_size: usize,
    pub shop_pool: Vec<BattleShopItem>,
    pub next_offer_index: usize,
    pub next_biased_offer_index: usize,
    pub wins: u32,
    pub losses: u32,
    pub result: Option<BattleRunResult>,
}

impl BattleRunState {
    pub fn from_definition(definition: BattleDefinition) -> Self {
        let run_config = effective_run_config(definition.run);
        let mut challenger = definition.challenger;
        let run_config = apply_leader(run_config, &mut challenger, definition.leader.as_ref());
        let mut run = Self {
            phase: BattleRunPhase::Draft,
            draft: DraftState {
                gold: run_config.starting_gold,
                team: challenger,
                bench: Vec::new(),
                equipment_inventory: Vec::new(),
                active_team_limit: run_config.active_team_limit,
                shop: Vec::new(),
            },
            opponents: run_config.opponent_rounds,
            battle: None,
            ability_database: definition.ability_database,
            leader: definition.leader,
            max_turn: definition.max_turn,
            rng_seed: definition.rng_seed,
            battle_index: 0,
            health: run_config.health,
            max_health: run_config.health,
            loss_health_damage: run_config.loss_health_damage,
            shop_size: run_config.shop_size,
            shop_pool: run_config.shop_pool,
            next_offer_index: 0,
            next_biased_offer_index: 0,
            wins: 0,
            losses: 0,
            result: None,
        };
        let _ = run.refresh_shop();
        run
    }

    pub fn refresh_shop(&mut self) -> Result<(), BattleRunError> {
        if self.phase != BattleRunPhase::Draft {
            return Err(BattleRunError::InvalidShopPhase);
        }

        self.draft.shop.clear();
        for slot in 0..self.shop_size {
            let Some(offer) = self.next_shop_offer(slot) else {
                break;
            };
            self.draft.shop.push(offer);
        }
        Ok(())
    }

    pub fn start_battle(&mut self) -> Result<(), BattleRunError> {
        if self.battle.is_some() {
            return Err(BattleRunError::BattleAlreadyActive);
        }
        if self.draft.team.chimeras.is_empty() {
            return Err(BattleRunError::EmptyChallengerTeam);
        }

        let Some(opponent) = self.opponents.get(self.battle_index).cloned() else {
            self.phase = BattleRunPhase::Complete;
            self.result = Some(BattleRunResult::Victory);
            return Err(BattleRunError::NoDefenderAvailable);
        };

        self.battle = Some(BattleState {
            name: format!("Battle {} - {}", self.battle_index + 1, opponent.name),
            turn: 0,
            has_started: false,
            max_turn: self.max_turn,
            is_finished: false,
            winner: None,
            challenger: reset_team_for_battle(self.draft.team.clone()),
            defender: reset_team_for_battle(opponent.defender),
            ability_database: self.ability_database.clone(),
            rng: BattleRng::new(self.rng_seed + self.battle_index as u64),
        });
        self.phase = BattleRunPhase::Battle;
        Ok(())
    }

    pub fn step(&mut self) -> Result<(BattleRunStep, BattleOutcome), BattleRunError> {
        match self.phase {
            BattleRunPhase::Draft => {
                self.start_battle()?;
                Ok((BattleRunStep::StartedBattle, BattleOutcome::default()))
            }
            BattleRunPhase::Battle => {
                let mut outcome = self
                    .battle
                    .as_mut()
                    .map(BattleState::step_turn)
                    .unwrap_or_default();

                if self
                    .battle
                    .as_ref()
                    .is_some_and(|battle| battle.is_finished)
                {
                    let winner = self.battle.as_ref().and_then(|battle| battle.winner);
                    self.resolve_battle(winner, &mut outcome);
                    Ok((BattleRunStep::BattleResolved { winner }, outcome))
                } else {
                    Ok((BattleRunStep::AdvancedBattle, outcome))
                }
            }
            BattleRunPhase::Complete => {
                Ok((BattleRunStep::AdvancedBattle, BattleOutcome::default()))
            }
        }
    }

    fn resolve_battle(&mut self, winner: Option<TeamSide>, outcome: &mut BattleOutcome) {
        let rewards = self
            .current_opponent()
            .map(|opponent| opponent.win_rewards.clone())
            .unwrap_or_default();
        match winner {
            Some(TeamSide::Challenger) => {
                self.wins += 1;
            }
            Some(TeamSide::Defender) | None => {
                let damage = self.current_opponent_loss_damage();
                self.losses += 1;
                self.health = (self.health - damage).max(0);
                outcome.push_log(format!(
                    "Run damage: lost {damage} health. Health: {}/{}.",
                    self.health, self.max_health
                ));
            }
        }

        self.battle = None;
        self.battle_index += 1;
        self.phase = if self.health <= 0 || self.battle_index >= self.opponents.len() {
            self.result = Some(if self.health <= 0 {
                BattleRunResult::Defeat
            } else {
                BattleRunResult::Victory
            });
            BattleRunPhase::Complete
        } else {
            BattleRunPhase::Draft
        };

        if self.phase == BattleRunPhase::Draft {
            let _ = self.refresh_shop();
        }

        if winner == Some(TeamSide::Challenger) {
            self.apply_win_rewards(&rewards, outcome);
        }
    }

    fn next_shop_offer(&mut self, slot: usize) -> Option<BattleShopItem> {
        if self.shop_pool.is_empty() {
            return None;
        }

        if self.should_bias_shop_slot(slot)
            && let Some(offer) = self.next_biased_shop_offer()
        {
            return Some(offer);
        }

        let offer = self.shop_pool[self.next_offer_index % self.shop_pool.len()].clone();
        self.next_offer_index += 1;
        Some(offer)
    }

    fn should_bias_shop_slot(&self, slot: usize) -> bool {
        let Some(leader) = &self.leader else {
            return false;
        };

        leader.shop_bias_every > 0
            && !leader.preferred_shop_tags.is_empty()
            && (slot + 1).is_multiple_of(leader.shop_bias_every)
    }

    fn next_biased_shop_offer(&mut self) -> Option<BattleShopItem> {
        let preferred_tags = &self.leader.as_ref()?.preferred_shop_tags;
        let matching_indices = self
            .shop_pool
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                item.tags()
                    .iter()
                    .any(|tag| preferred_tags.contains(tag))
                    .then_some(index)
            })
            .collect::<Vec<_>>();

        if matching_indices.is_empty() {
            return None;
        }

        let index = matching_indices[self.next_biased_offer_index % matching_indices.len()];
        self.next_biased_offer_index += 1;
        Some(self.shop_pool[index].clone())
    }

    pub fn current_opponent(&self) -> Option<&BattleOpponentRound> {
        self.opponents.get(self.battle_index)
    }

    fn current_opponent_loss_damage(&self) -> i32 {
        self.current_opponent()
            .map(|opponent| opponent.loss_health_damage)
            .unwrap_or(self.loss_health_damage)
    }

    fn apply_win_rewards(&mut self, rewards: &[BattleRunReward], outcome: &mut BattleOutcome) {
        for reward in rewards {
            match reward {
                BattleRunReward::AddGold { amount } => {
                    self.draft.gold += amount;
                    outcome.push_log(format!("Run reward: gained {amount} gold."));
                }
                BattleRunReward::HealRun { amount } => {
                    let before = self.health;
                    self.health = (self.health + amount).clamp(0, self.max_health);
                    outcome.push_log(format!(
                        "Run reward: healed {} health. Health: {}/{}.",
                        self.health - before,
                        self.health,
                        self.max_health
                    ));
                }
                BattleRunReward::AddShopItem { item } => {
                    self.draft.shop.push(item.clone());
                    outcome.push_log(format!("Run reward: added {} to the shop.", item.name()));
                }
            }
        }
    }
}

fn apply_leader(
    mut config: BattleRunConfig,
    challenger: &mut BattleTeam,
    leader: Option<&BattleLeader>,
) -> BattleRunConfig {
    let Some(leader) = leader else {
        return config;
    };

    for effect in &leader.effects {
        match *effect {
            BattleLeaderEffect::AddStartingGold { amount } => {
                config.starting_gold += amount;
            }
            BattleLeaderEffect::AddRunHealth { amount } => {
                config.health += amount;
            }
            BattleLeaderEffect::AddWinGoldReward { amount } => {
                for opponent in &mut config.opponent_rounds {
                    add_gold_to_rewards(&mut opponent.win_rewards, amount);
                }
            }
            BattleLeaderEffect::AddTeamStats { attack, hp } => {
                add_team_stats(challenger, attack, hp);
            }
            BattleLeaderEffect::AddShopOfferStats { attack, hp } => {
                add_shop_offer_stats(&mut config.shop_pool, attack, hp);
            }
        }
    }

    config.starting_gold = config.starting_gold.max(0);
    config.health = config.health.max(1);
    config
}

fn add_gold_to_rewards(rewards: &mut Vec<BattleRunReward>, amount: i32) {
    if let Some(BattleRunReward::AddGold {
        amount: reward_amount,
    }) = rewards
        .iter_mut()
        .find(|reward| matches!(reward, BattleRunReward::AddGold { .. }))
    {
        *reward_amount = (*reward_amount + amount).max(0);
    } else if amount > 0 {
        rewards.push(BattleRunReward::AddGold { amount });
    }
}

fn effective_run_config(mut config: BattleRunConfig) -> BattleRunConfig {
    if config.shop_size == 0 {
        config.shop_size = BATTLE_SHOP_SIZE;
    }
    if config.active_team_limit == 0 {
        config.active_team_limit = DEFAULT_ACTIVE_TEAM_LIMIT;
    }
    if config.health <= 0 {
        config.health = BATTLE_RUN_HEALTH;
    }
    if config.loss_health_damage <= 0 {
        config.loss_health_damage = BATTLE_LOSS_HEALTH_DAMAGE;
    }
    if config.shop_pool.is_empty() {
        config.shop_pool = default_shop_pool();
    }
    config
}

fn add_team_stats(team: &mut BattleTeam, attack: i32, hp: i32) {
    for chimera in &mut team.chimeras {
        chimera.stats.attack = (chimera.stats.attack + attack).max(0);
        chimera.stats.max_hp = (chimera.stats.max_hp + hp).max(1);
        chimera.stats.hp = (chimera.stats.hp + hp).clamp(1, chimera.stats.max_hp);
    }
}

fn add_shop_offer_stats(offers: &mut [BattleShopItem], attack: i32, hp: i32) {
    for offer in offers {
        match offer {
            BattleShopItem::Chimera(offer) => {
                offer.attack = (offer.attack + attack).max(0);
                offer.hp = (offer.hp + hp).max(1);
            }
            BattleShopItem::Equipment(offer) => {
                offer.attack = (offer.attack + attack).max(0);
                offer.hp = (offer.hp + hp).max(0);
            }
        }
    }
}

impl Default for BattleRunConfig {
    fn default() -> Self {
        Self {
            starting_gold: BATTLE_STARTING_GOLD,
            shop_size: BATTLE_SHOP_SIZE,
            active_team_limit: DEFAULT_ACTIVE_TEAM_LIMIT,
            health: BATTLE_RUN_HEALTH,
            loss_health_damage: BATTLE_LOSS_HEALTH_DAMAGE,
            opponent_rounds: Vec::new(),
            shop_pool: default_shop_pool(),
        }
    }
}

fn default_shop_pool() -> Vec<BattleShopItem> {
    vec![
        BattleShopItem::Chimera(BattleChimeraOffer::new(
            "Workaholic",
            BattleRarity::White,
            5,
            5,
            vec![BattleAbilityId("workaholic")],
        )),
        BattleShopItem::Chimera(BattleChimeraOffer::new(
            "Tough Cookie",
            BattleRarity::White,
            2,
            10,
            vec![BattleAbilityId("tough_cookie")],
        )),
        BattleShopItem::Chimera(BattleChimeraOffer::new(
            "Healer",
            BattleRarity::White,
            2,
            6,
            vec![BattleAbilityId("soothing_care")],
        )),
        BattleShopItem::Chimera(BattleChimeraOffer::new(
            "Little Villain",
            BattleRarity::White,
            1,
            2,
            vec![BattleAbilityId("little_villain")],
        )),
        BattleShopItem::Equipment(BattleEquipmentOffer::new(
            "Training Collar",
            BattleRarity::White,
            1,
            2,
        )),
    ]
}

fn reset_team_for_battle(mut team: BattleTeam) -> BattleTeam {
    team.summon_queue.clear();
    for chimera in &mut team.chimeras {
        chimera.stats.hp = chimera.stats.max_hp;
    }
    team
}
