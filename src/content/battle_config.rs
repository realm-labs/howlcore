//! RON-backed battle content loading.

use serde::Deserialize;
use std::collections::HashSet;

use crate::core::battle::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleChimera, BattleDefinition,
    BattleEffect, BattleRarity, BattleStats, BattleTargetSelector, BattleTeam, BattleTrigger,
    TeamSide,
};

const ABILITIES_RON: &str = include_str!("../../assets/battle/abilities.ron");
const TEST_BATTLE_RON: &str = include_str!("../../assets/battle/test_battle.ron");

#[derive(Debug, Deserialize)]
pub struct AbilityFile {
    pub abilities: Vec<AbilityConfig>,
}

#[derive(Debug)]
pub enum BattleConfigError {
    Parse(ron::error::SpannedError),
    DuplicateAbility { id: String },
    UnknownAbility { id: String },
}

#[derive(Debug, Deserialize)]
pub struct AbilityConfig {
    pub id: String,
    pub name: String,
    pub trigger: TriggerConfig,
    pub selector: TargetSelectorConfig,
    pub effects: Vec<EffectConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BattleConfig {
    pub name: String,
    pub max_turn: u32,
    pub rng_seed: u64,
    pub challenger: TeamConfig,
    pub defender: TeamConfig,
    pub initial_logs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TeamConfig {
    pub name: String,
    pub chimeras: Vec<ChimeraConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ChimeraConfig {
    pub name: String,
    pub attack: i32,
    pub hp: i32,
    #[serde(default = "default_level")]
    pub level: u32,
    #[serde(default)]
    pub experience: u32,
    #[serde(default = "default_rarity")]
    pub rarity: RarityConfig,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub abilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum TriggerConfig {
    BattleStart,
    TurnStart,
    BeforeDamageTaken,
    AfterDamageTaken,
    OnAllyAttack,
    AfterAttack,
    OnAllyAheadDamaged,
    OnSummon,
    OnKnockdown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum TargetSelectorConfig {
    SelfChimera,
    AttackTarget,
    DamageTarget,
    SummonedChimera,
    KnockedDownChimera,
    FrontEnemy,
    FirstLivingEnemy,
    AllyAhead,
    AllyBehind,
    HighestHpEnemy,
    HighestAttackEnemy,
    AllEnemies,
    AllAllies,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum RarityConfig {
    White,
    Blue,
    Purple,
    Gold,
    Prismatic,
}

#[derive(Debug, Deserialize)]
pub enum EffectConfig {
    Chance {
        percent: u32,
        effects: Vec<EffectConfig>,
    },
    DealDamage {
        amount: i32,
    },
    DealAttackDamagePercent {
        percent: u32,
        minimum: i32,
    },
    Heal {
        amount: i32,
    },
    AddAttack {
        amount: i32,
    },
    ReduceIncomingDamage {
        amount: i32,
        minimum: i32,
    },
    SwapWithTarget,
    QueueSummon {
        name: String,
        attack: i32,
        hp: i32,
        #[serde(default)]
        abilities: Vec<String>,
    },
}

pub fn load_test_battle() -> BattleDefinition {
    load_test_battle_result().expect("embedded battle RON content should load")
}

pub fn load_test_battle_result() -> Result<BattleDefinition, BattleConfigError> {
    let abilities =
        ron::from_str::<AbilityFile>(ABILITIES_RON).map_err(BattleConfigError::Parse)?;
    let battle =
        ron::from_str::<BattleConfig>(TEST_BATTLE_RON).map_err(BattleConfigError::Parse)?;
    validate_config(&abilities, &battle)?;
    Ok(build_battle_definition(abilities, battle))
}

fn validate_config(
    abilities: &AbilityFile,
    battle: &BattleConfig,
) -> Result<(), BattleConfigError> {
    let mut ability_ids = HashSet::new();

    for ability in &abilities.abilities {
        if !ability_ids.insert(ability.id.clone()) {
            return Err(BattleConfigError::DuplicateAbility {
                id: ability.id.clone(),
            });
        }
    }

    for ability in &abilities.abilities {
        validate_effects(&ability.effects, &ability_ids)?;
    }

    for team in [&battle.challenger, &battle.defender] {
        for chimera in &team.chimeras {
            for ability in &chimera.abilities {
                validate_ability_ref(ability, &ability_ids)?;
            }
        }
    }

    Ok(())
}

fn validate_effects(
    effects: &[EffectConfig],
    ability_ids: &HashSet<String>,
) -> Result<(), BattleConfigError> {
    for effect in effects {
        match effect {
            EffectConfig::Chance { effects, .. } => validate_effects(effects, ability_ids)?,
            EffectConfig::QueueSummon { abilities, .. } => {
                for ability in abilities {
                    validate_ability_ref(ability, ability_ids)?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn validate_ability_ref(
    ability: &str,
    ability_ids: &HashSet<String>,
) -> Result<(), BattleConfigError> {
    if ability_ids.contains(ability) {
        Ok(())
    } else {
        Err(BattleConfigError::UnknownAbility {
            id: ability.to_string(),
        })
    }
}

fn build_battle_definition(abilities: AbilityFile, battle: BattleConfig) -> BattleDefinition {
    BattleDefinition {
        name: battle.name,
        max_turn: battle.max_turn,
        challenger: build_team(TeamSide::Challenger, battle.challenger),
        defender: build_team(TeamSide::Defender, battle.defender),
        ability_database: build_ability_database(abilities),
        rng_seed: battle.rng_seed,
        initial_logs: battle.initial_logs,
    }
}

fn build_ability_database(file: AbilityFile) -> BattleAbilityDatabase {
    let mut database = BattleAbilityDatabase::default();

    for ability in file.abilities {
        let def = BattleAbilityDef {
            id: ability_id(ability.id),
            name: static_str(ability.name),
            trigger: ability.trigger.into(),
            selector: ability.selector.into(),
            effects: ability.effects.into_iter().map(Into::into).collect(),
        };
        database.abilities.insert(def.id, def);
    }

    database
}

fn build_team(side: TeamSide, team: TeamConfig) -> BattleTeam {
    BattleTeam {
        side,
        name: team.name,
        chimeras: team
            .chimeras
            .into_iter()
            .enumerate()
            .map(|(slot, chimera)| build_chimera(slot as u32, chimera))
            .collect(),
        summon_queue: Vec::new(),
    }
}

fn build_chimera(slot: u32, chimera: ChimeraConfig) -> BattleChimera {
    BattleChimera {
        name: chimera.name,
        slot,
        level: chimera.level,
        experience: chimera.experience,
        rarity: chimera.rarity.into(),
        tags: chimera.tags,
        stats: BattleStats {
            attack: chimera.attack,
            max_hp: chimera.hp,
            hp: chimera.hp,
        },
        abilities: chimera.abilities.into_iter().map(ability_id).collect(),
    }
}

impl From<TriggerConfig> for BattleTrigger {
    fn from(value: TriggerConfig) -> Self {
        match value {
            TriggerConfig::BattleStart => Self::BattleStart,
            TriggerConfig::TurnStart => Self::TurnStart,
            TriggerConfig::BeforeDamageTaken => Self::BeforeDamageTaken,
            TriggerConfig::AfterDamageTaken => Self::AfterDamageTaken,
            TriggerConfig::OnAllyAttack => Self::OnAllyAttack,
            TriggerConfig::AfterAttack => Self::AfterAttack,
            TriggerConfig::OnAllyAheadDamaged => Self::OnAllyAheadDamaged,
            TriggerConfig::OnSummon => Self::OnSummon,
            TriggerConfig::OnKnockdown => Self::OnKnockdown,
        }
    }
}

impl From<TargetSelectorConfig> for BattleTargetSelector {
    fn from(value: TargetSelectorConfig) -> Self {
        match value {
            TargetSelectorConfig::SelfChimera => Self::SelfChimera,
            TargetSelectorConfig::AttackTarget => Self::AttackTarget,
            TargetSelectorConfig::DamageTarget => Self::DamageTarget,
            TargetSelectorConfig::SummonedChimera => Self::SummonedChimera,
            TargetSelectorConfig::KnockedDownChimera => Self::KnockedDownChimera,
            TargetSelectorConfig::FrontEnemy => Self::FrontEnemy,
            TargetSelectorConfig::FirstLivingEnemy => Self::FirstLivingEnemy,
            TargetSelectorConfig::AllyAhead => Self::AllyAhead,
            TargetSelectorConfig::AllyBehind => Self::AllyBehind,
            TargetSelectorConfig::HighestHpEnemy => Self::HighestHpEnemy,
            TargetSelectorConfig::HighestAttackEnemy => Self::HighestAttackEnemy,
            TargetSelectorConfig::AllEnemies => Self::AllEnemies,
            TargetSelectorConfig::AllAllies => Self::AllAllies,
        }
    }
}

impl From<RarityConfig> for BattleRarity {
    fn from(value: RarityConfig) -> Self {
        match value {
            RarityConfig::White => Self::White,
            RarityConfig::Blue => Self::Blue,
            RarityConfig::Purple => Self::Purple,
            RarityConfig::Gold => Self::Gold,
            RarityConfig::Prismatic => Self::Prismatic,
        }
    }
}

impl From<EffectConfig> for BattleEffect {
    fn from(value: EffectConfig) -> Self {
        match value {
            EffectConfig::Chance { percent, effects } => Self::Chance {
                percent,
                effects: effects.into_iter().map(Into::into).collect(),
            },
            EffectConfig::DealDamage { amount } => Self::DealDamage { amount },
            EffectConfig::DealAttackDamagePercent { percent, minimum } => {
                Self::DealAttackDamagePercent { percent, minimum }
            }
            EffectConfig::Heal { amount } => Self::Heal { amount },
            EffectConfig::AddAttack { amount } => Self::AddAttack { amount },
            EffectConfig::ReduceIncomingDamage { amount, minimum } => {
                Self::ReduceIncomingDamage { amount, minimum }
            }
            EffectConfig::SwapWithTarget => Self::SwapWithTarget,
            EffectConfig::QueueSummon {
                name,
                attack,
                hp,
                abilities,
            } => Self::QueueSummon {
                name: static_str(name),
                attack,
                hp,
                abilities: abilities.into_iter().map(ability_id).collect(),
            },
        }
    }
}

fn default_level() -> u32 {
    1
}

fn default_rarity() -> RarityConfig {
    RarityConfig::White
}

fn ability_id(value: String) -> BattleAbilityId {
    BattleAbilityId(static_str(value))
}

fn static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}
