//! Test battle content for the two-team chimera battle prototype.

use crate::core::battle::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleChimera, BattleDefinition,
    BattleEffect, BattleRarity, BattleStats, BattleTargetSelector, BattleTeam, BattleTrigger,
    TeamSide,
};

pub const WORKAHOLIC: BattleAbilityId = BattleAbilityId("workaholic");
pub const TOUGH_COOKIE: BattleAbilityId = BattleAbilityId("tough_cookie");
pub const SOOTHING_CARE: BattleAbilityId = BattleAbilityId("soothing_care");
pub const ABSENTEE_FREAK: BattleAbilityId = BattleAbilityId("absentee_freak");
pub const RUTHLESS_DEMON: BattleAbilityId = BattleAbilityId("ruthless_demon");
pub const LITTLE_VILLAIN: BattleAbilityId = BattleAbilityId("little_villain");
pub const KIND_PRAISER: BattleAbilityId = BattleAbilityId("kind_praiser");
pub const SUMMON_TRAINER: BattleAbilityId = BattleAbilityId("summon_trainer");

pub fn test_battle() -> BattleDefinition {
    BattleDefinition {
        name: "Chimera Scrimmage".to_string(),
        max_turn: 20,
        challenger: team(
            TeamSide::Challenger,
            "Challenger",
            vec![
                chimera("Rat Race King", 0, 3, 8, &[KIND_PRAISER]),
                chimera("Healer", 1, 2, 6, &[SOOTHING_CARE]),
                chimera("Workaholic", 2, 5, 5, &[WORKAHOLIC]),
                chimera("Absentee Freak", 3, 1, 3, &[ABSENTEE_FREAK]),
            ],
        ),
        defender: team(
            TeamSide::Defender,
            "Defender",
            vec![
                chimera("Tough Cookie", 0, 2, 10, &[TOUGH_COOKIE, SUMMON_TRAINER]),
                chimera("Pressure Monster", 1, 4, 5, &[]),
                chimera("Ruthless Demon", 2, 4, 7, &[RUTHLESS_DEMON]),
                chimera("Little Villain", 3, 1, 2, &[LITTLE_VILLAIN]),
            ],
        ),
        ability_database: test_battle_ability_database(),
        rng_seed: 1,
        initial_logs: vec![
            "Battle: Chimera Scrimmage".to_string(),
            "Each turn, both front chimeras attack each other.".to_string(),
        ],
    }
}

pub fn test_battle_ability_database() -> BattleAbilityDatabase {
    let mut database = BattleAbilityDatabase::default();
    let abilities = [
        BattleAbilityDef {
            id: WORKAHOLIC,
            name: "Workaholic",
            trigger: BattleTrigger::AfterAttack,
            selector: BattleTargetSelector::FirstLivingEnemy,
            effects: vec![BattleEffect::Chance {
                percent: 30,
                effects: vec![BattleEffect::DealAttackDamagePercent {
                    percent: 20,
                    minimum: 1,
                }],
            }],
        },
        BattleAbilityDef {
            id: TOUGH_COOKIE,
            name: "Tough Cookie",
            trigger: BattleTrigger::BeforeDamageTaken,
            selector: BattleTargetSelector::SelfChimera,
            effects: vec![BattleEffect::ReduceIncomingDamage {
                amount: 1,
                minimum: 1,
            }],
        },
        BattleAbilityDef {
            id: SOOTHING_CARE,
            name: "Soothing Care",
            trigger: BattleTrigger::OnAllyAheadDamaged,
            selector: BattleTargetSelector::DamageTarget,
            effects: vec![BattleEffect::Heal { amount: 1 }],
        },
        BattleAbilityDef {
            id: ABSENTEE_FREAK,
            name: "Absentee Freak",
            trigger: BattleTrigger::AfterDamageTaken,
            selector: BattleTargetSelector::AllyBehind,
            effects: vec![BattleEffect::SwapWithTarget],
        },
        BattleAbilityDef {
            id: RUTHLESS_DEMON,
            name: "Ruthless Demon",
            trigger: BattleTrigger::AfterDamageTaken,
            selector: BattleTargetSelector::SelfChimera,
            effects: vec![BattleEffect::QueueSummon {
                name: "Pressure Monster",
                attack: 2,
                hp: 3,
                abilities: Vec::new(),
            }],
        },
        BattleAbilityDef {
            id: LITTLE_VILLAIN,
            name: "Little Villain",
            trigger: BattleTrigger::BattleStart,
            selector: BattleTargetSelector::FrontEnemy,
            effects: vec![BattleEffect::DealDamage { amount: 1 }],
        },
        BattleAbilityDef {
            id: KIND_PRAISER,
            name: "Kind Praiser",
            trigger: BattleTrigger::OnKnockdown,
            selector: BattleTargetSelector::AllAllies,
            effects: vec![BattleEffect::AddAttack { amount: 1 }],
        },
        BattleAbilityDef {
            id: SUMMON_TRAINER,
            name: "Summon Trainer",
            trigger: BattleTrigger::OnSummon,
            selector: BattleTargetSelector::SummonedChimera,
            effects: vec![BattleEffect::AddAttack { amount: 1 }],
        },
    ];

    for ability in abilities {
        database.abilities.insert(ability.id, ability);
    }

    database
}

fn team(side: TeamSide, name: &'static str, chimeras: Vec<BattleChimera>) -> BattleTeam {
    BattleTeam {
        side,
        name: name.to_string(),
        chimeras,
        summon_queue: Vec::new(),
    }
}

fn chimera(
    name: &'static str,
    slot: u32,
    attack: i32,
    hp: i32,
    abilities: &[BattleAbilityId],
) -> BattleChimera {
    BattleChimera {
        name: name.to_string(),
        slot,
        level: 1,
        experience: 0,
        rarity: BattleRarity::White,
        tags: Vec::new(),
        stats: BattleStats {
            attack,
            max_hp: hp,
            hp,
        },
        abilities: abilities.to_vec(),
    }
}
