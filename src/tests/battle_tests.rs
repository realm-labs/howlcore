use crate::core::battle::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleChimera, BattleChimeraId,
    BattleEffect, BattleState, BattleStats, BattleTargetSelector, BattleTeam, BattleTrigger,
    TeamSide,
    resolver::{front_chimera_id, living_chimera_count},
};

fn chimera(name: &str, slot: u32, attack: i32, hp: i32) -> BattleChimera {
    BattleChimera {
        name: name.to_string(),
        slot,
        stats: BattleStats {
            attack,
            max_hp: hp,
            hp,
        },
        abilities: Vec::new(),
    }
}

fn chimera_with_ability(
    name: &str,
    slot: u32,
    attack: i32,
    hp: i32,
    ability: BattleAbilityId,
) -> BattleChimera {
    BattleChimera {
        abilities: vec![ability],
        ..chimera(name, slot, attack, hp)
    }
}

fn team(side: TeamSide, name: &str, chimeras: Vec<BattleChimera>) -> BattleTeam {
    BattleTeam {
        side,
        name: name.to_string(),
        chimeras,
    }
}

fn battle_state(challenger: Vec<BattleChimera>, defender: Vec<BattleChimera>) -> BattleState {
    BattleState {
        name: "Test Battle".to_string(),
        turn: 0,
        max_turn: 10,
        is_finished: false,
        winner: None,
        challenger: team(TeamSide::Challenger, "Challenger", challenger),
        defender: team(TeamSide::Defender, "Defender", defender),
        ability_database: BattleAbilityDatabase::default(),
    }
}

fn battle_state_with_abilities(
    challenger: Vec<BattleChimera>,
    defender: Vec<BattleChimera>,
    abilities: Vec<BattleAbilityDef>,
) -> BattleState {
    let mut state = battle_state(challenger, defender);
    state.ability_database.abilities = abilities
        .into_iter()
        .map(|ability| (ability.id, ability))
        .collect();
    state
}

#[test]
fn front_chimera_should_be_lowest_living_slot() {
    let mut state = battle_state(
        vec![chimera("Back", 2, 1, 3), chimera("Front", 0, 1, 0)],
        vec![chimera("Enemy", 0, 1, 3)],
    );

    let front = front_chimera_id(&state, TeamSide::Challenger);

    assert_eq!(
        front,
        Some(BattleChimeraId {
            side: TeamSide::Challenger,
            index: 0
        })
    );

    state.challenger.chimeras[0].stats.hp = 0;
    assert_eq!(front_chimera_id(&state, TeamSide::Challenger), None);
}

#[test]
fn front_chimeras_should_attack_each_other_in_the_same_turn() {
    let mut state = battle_state(
        vec![chimera("Rat Race King", 0, 3, 6)],
        vec![chimera("Tough Cookie", 0, 2, 5)],
    );

    state.step_turn();

    assert_eq!(state.challenger.chimeras[0].stats.hp, 4);
    assert_eq!(state.defender.chimeras[0].stats.hp, 2);
    assert!(!state.is_finished);
}

#[test]
fn knocked_down_front_should_be_replaced_next_turn() {
    let mut state = battle_state(
        vec![
            chimera("Fragile Front", 0, 4, 2),
            chimera("Second", 1, 3, 8),
        ],
        vec![chimera("Enemy Front", 0, 5, 6)],
    );

    state.step_turn();

    assert_eq!(state.challenger.chimeras[0].stats.hp, 0);
    assert_eq!(living_chimera_count(&state, TeamSide::Challenger), 1);
    assert_eq!(
        front_chimera_id(&state, TeamSide::Challenger),
        Some(BattleChimeraId {
            side: TeamSide::Challenger,
            index: 1
        })
    );
}

#[test]
fn battle_should_end_when_one_team_has_no_living_chimeras() {
    let mut state = battle_state(
        vec![chimera("Winner", 0, 3, 5)],
        vec![chimera("Loser", 0, 2, 3)],
    );

    state.step_turn();

    assert!(state.is_finished);
    assert_eq!(state.winner, Some(TeamSide::Challenger));
}

#[test]
fn tough_cookie_ability_should_reduce_incoming_damage() {
    let tough_cookie = BattleAbilityId("tough_cookie");
    let mut state = battle_state_with_abilities(
        vec![chimera("Attacker", 0, 3, 5)],
        vec![chimera_with_ability("Tough Cookie", 0, 2, 10, tough_cookie)],
        vec![BattleAbilityDef {
            id: tough_cookie,
            name: "Tough Cookie",
            trigger: BattleTrigger::BeforeDamageTaken,
            selector: BattleTargetSelector::SelfChimera,
            effects: vec![BattleEffect::ReduceIncomingDamage {
                amount: 1,
                minimum: 1,
            }],
        }],
    );

    state.step_turn();

    assert_eq!(state.defender.chimeras[0].stats.hp, 8);
}

#[test]
fn healer_should_restore_hp_when_ally_ahead_is_damaged() {
    let soothing_care = BattleAbilityId("soothing_care");
    let mut state = battle_state_with_abilities(
        vec![chimera("Attacker", 0, 3, 10)],
        vec![
            chimera("Front Ally", 0, 1, 5),
            chimera_with_ability("Healer", 1, 1, 4, soothing_care),
        ],
        vec![BattleAbilityDef {
            id: soothing_care,
            name: "Soothing Care",
            trigger: BattleTrigger::OnAllyAheadDamaged,
            selector: BattleTargetSelector::DamageTarget,
            effects: vec![BattleEffect::Heal { amount: 1 }],
        }],
    );

    state.step_turn();

    assert_eq!(state.defender.chimeras[0].stats.hp, 3);
}

#[test]
fn workaholic_should_follow_up_after_attacking() {
    let workaholic = BattleAbilityId("workaholic");
    let mut state = battle_state_with_abilities(
        vec![chimera_with_ability("Workaholic", 0, 5, 8, workaholic)],
        vec![chimera("Target", 0, 1, 10)],
        vec![BattleAbilityDef {
            id: workaholic,
            name: "Workaholic",
            trigger: BattleTrigger::AfterAttack,
            selector: BattleTargetSelector::FirstLivingEnemy,
            effects: vec![BattleEffect::DealAttackDamagePercent {
                percent: 20,
                minimum: 1,
            }],
        }],
    );

    state.step_turn();

    assert_eq!(state.defender.chimeras[0].stats.hp, 4);
}
