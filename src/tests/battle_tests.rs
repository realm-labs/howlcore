use crate::core::battle::{
    BattleAbilityDatabase, BattleAbilityDef, BattleAbilityId, BattleChimera, BattleChimeraId,
    BattleEffect, BattleEvent, BattleRarity, BattleRng, BattleState, BattleStats,
    BattleTargetSelector, BattleTeam, BattleTrigger, TeamSide,
    resolver::{front_chimera_id, living_chimera_count},
};

fn chimera(name: &str, slot: u32, attack: i32, hp: i32) -> BattleChimera {
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
        summon_queue: Vec::new(),
    }
}

fn battle_state(challenger: Vec<BattleChimera>, defender: Vec<BattleChimera>) -> BattleState {
    BattleState {
        name: "Test Battle".to_string(),
        turn: 0,
        has_started: false,
        max_turn: 10,
        is_finished: false,
        winner: None,
        challenger: team(TeamSide::Challenger, "Challenger", challenger),
        defender: team(TeamSide::Defender, "Defender", defender),
        ability_database: BattleAbilityDatabase::default(),
        rng: BattleRng::new(1),
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

#[test]
fn chance_effect_should_use_deterministic_rng() {
    let lucky_hit = BattleAbilityId("lucky_hit");
    let mut state = battle_state_with_abilities(
        vec![chimera_with_ability("Lucky", 0, 1, 8, lucky_hit)],
        vec![chimera("Target", 0, 1, 10)],
        vec![BattleAbilityDef {
            id: lucky_hit,
            name: "Lucky Hit",
            trigger: BattleTrigger::AfterAttack,
            selector: BattleTargetSelector::FirstLivingEnemy,
            effects: vec![BattleEffect::Chance {
                percent: 100,
                effects: vec![BattleEffect::DealDamage { amount: 2 }],
            }],
        }],
    );

    let outcome = state.step_turn();

    assert_eq!(state.defender.chimeras[0].stats.hp, 7);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        BattleEvent::ChanceRolled {
            percent: 100,
            success: true,
            ..
        }
    )));
}

#[test]
fn failed_chance_effect_should_not_apply_nested_effects() {
    let unlucky_hit = BattleAbilityId("unlucky_hit");
    let mut state = battle_state_with_abilities(
        vec![chimera_with_ability("Unlucky", 0, 1, 8, unlucky_hit)],
        vec![chimera("Target", 0, 1, 10)],
        vec![BattleAbilityDef {
            id: unlucky_hit,
            name: "Unlucky Hit",
            trigger: BattleTrigger::AfterAttack,
            selector: BattleTargetSelector::FirstLivingEnemy,
            effects: vec![BattleEffect::Chance {
                percent: 0,
                effects: vec![BattleEffect::DealDamage { amount: 2 }],
            }],
        }],
    );

    let outcome = state.step_turn();

    assert_eq!(state.defender.chimeras[0].stats.hp, 9);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        BattleEvent::ChanceRolled {
            percent: 0,
            success: false,
            ..
        }
    )));
}

#[test]
fn damaged_chimera_can_swap_with_ally_behind() {
    let absentee_freak = BattleAbilityId("absentee_freak");
    let mut state = battle_state_with_abilities(
        vec![chimera("Attacker", 0, 1, 10)],
        vec![
            chimera_with_ability("Absentee Freak", 0, 1, 5, absentee_freak),
            chimera("Back Ally", 1, 1, 5),
        ],
        vec![BattleAbilityDef {
            id: absentee_freak,
            name: "Absentee Freak",
            trigger: BattleTrigger::AfterDamageTaken,
            selector: BattleTargetSelector::AllyBehind,
            effects: vec![BattleEffect::SwapWithTarget],
        }],
    );

    state.step_turn();

    assert_eq!(state.defender.chimeras[0].slot, 1);
    assert_eq!(state.defender.chimeras[1].slot, 0);
}

#[test]
fn ability_can_queue_and_deploy_summoned_chimera() {
    let ruthless_demon = BattleAbilityId("ruthless_demon");
    let mut state = battle_state_with_abilities(
        vec![chimera("Attacker", 0, 1, 10)],
        vec![chimera_with_ability(
            "Ruthless Demon",
            0,
            1,
            5,
            ruthless_demon,
        )],
        vec![BattleAbilityDef {
            id: ruthless_demon,
            name: "Ruthless Demon",
            trigger: BattleTrigger::AfterDamageTaken,
            selector: BattleTargetSelector::SelfChimera,
            effects: vec![BattleEffect::QueueSummon {
                name: "Pressure Monster",
                attack: 2,
                hp: 3,
                abilities: Vec::new(),
            }],
        }],
    );

    let outcome = state.step_turn();

    assert_eq!(state.defender.chimeras.len(), 2);
    assert_eq!(state.defender.chimeras[1].name, "Pressure Monster");
    assert_eq!(state.defender.chimeras[1].slot, 1);
    assert!(
        outcome
            .events
            .iter()
            .any(|event| matches!(event, BattleEvent::ChimeraSummoned { .. }))
    );
}

#[test]
fn battle_start_abilities_should_trigger_once_before_first_turn() {
    let opening_shot = BattleAbilityId("opening_shot");
    let mut state = battle_state_with_abilities(
        vec![chimera_with_ability("Starter", 0, 1, 8, opening_shot)],
        vec![chimera("Target", 0, 1, 10)],
        vec![BattleAbilityDef {
            id: opening_shot,
            name: "Opening Shot",
            trigger: BattleTrigger::BattleStart,
            selector: BattleTargetSelector::FrontEnemy,
            effects: vec![BattleEffect::DealDamage { amount: 2 }],
        }],
    );

    state.step_turn();
    state.step_turn();

    assert_eq!(state.defender.chimeras[0].stats.hp, 6);
}

#[test]
fn on_summon_abilities_should_target_summoned_chimera() {
    let summoner = BattleAbilityId("summoner");
    let trainer = BattleAbilityId("trainer");
    let mut state = battle_state_with_abilities(
        vec![chimera("Attacker", 0, 1, 10)],
        vec![
            chimera_with_ability("Summoner", 0, 1, 5, summoner),
            chimera_with_ability("Trainer", 1, 1, 5, trainer),
        ],
        vec![
            BattleAbilityDef {
                id: summoner,
                name: "Summoner",
                trigger: BattleTrigger::AfterDamageTaken,
                selector: BattleTargetSelector::SelfChimera,
                effects: vec![BattleEffect::QueueSummon {
                    name: "Token",
                    attack: 1,
                    hp: 2,
                    abilities: Vec::new(),
                }],
            },
            BattleAbilityDef {
                id: trainer,
                name: "Trainer",
                trigger: BattleTrigger::OnSummon,
                selector: BattleTargetSelector::SummonedChimera,
                effects: vec![BattleEffect::AddAttack { amount: 2 }],
            },
        ],
    );

    state.step_turn();

    assert_eq!(state.defender.chimeras[2].name, "Token");
    assert_eq!(state.defender.chimeras[2].stats.attack, 3);
}

#[test]
fn on_knockdown_abilities_should_trigger_for_living_chimeras() {
    let kind_praiser = BattleAbilityId("kind_praiser");
    let mut state = battle_state_with_abilities(
        vec![
            chimera("Front", 0, 3, 5),
            chimera_with_ability("Kind Praiser", 1, 1, 5, kind_praiser),
        ],
        vec![chimera("Target", 0, 1, 3)],
        vec![BattleAbilityDef {
            id: kind_praiser,
            name: "Kind Praiser",
            trigger: BattleTrigger::OnKnockdown,
            selector: BattleTargetSelector::AllAllies,
            effects: vec![BattleEffect::AddAttack { amount: 1 }],
        }],
    );

    state.step_turn();

    assert_eq!(state.challenger.chimeras[0].stats.attack, 4);
    assert_eq!(state.challenger.chimeras[1].stats.attack, 2);
}
