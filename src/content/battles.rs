//! Test battle content for the two-team chimera battle prototype.

use crate::core::battle::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleChimera, BattleDefinition,
    BattleEffect, BattleStats, BattleTargetSelector, BattleTeam, BattleTrigger, TeamSide,
};

pub const WORKAHOLIC: BattleAbilityId = BattleAbilityId("workaholic");
pub const TOUGH_COOKIE: BattleAbilityId = BattleAbilityId("tough_cookie");
pub const SOOTHING_CARE: BattleAbilityId = BattleAbilityId("soothing_care");

pub fn test_battle() -> BattleDefinition {
    BattleDefinition {
        name: "Chimera Scrimmage".to_string(),
        max_turn: 20,
        challenger: team(
            TeamSide::Challenger,
            "Challenger",
            vec![
                chimera("Rat Race King", 0, 3, 8, &[]),
                chimera("Healer", 1, 2, 6, &[SOOTHING_CARE]),
                chimera("Workaholic", 2, 5, 5, &[WORKAHOLIC]),
            ],
        ),
        defender: team(
            TeamSide::Defender,
            "Defender",
            vec![
                chimera("Tough Cookie", 0, 2, 10, &[TOUGH_COOKIE]),
                chimera("Pressure Monster", 1, 4, 5, &[]),
                chimera("Old Honest", 2, 2, 7, &[]),
            ],
        ),
        ability_database: test_battle_ability_database(),
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
            effects: vec![BattleEffect::DealAttackDamagePercent {
                percent: 20,
                minimum: 1,
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
        stats: BattleStats {
            attack,
            max_hp: hp,
            hp,
        },
        abilities: abilities.to_vec(),
    }
}
