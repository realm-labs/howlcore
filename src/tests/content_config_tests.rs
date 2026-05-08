use crate::{
    content::{
        battle_config::load_test_battle_result,
        battles::{KIND_PRAISER, TOUGH_COOKIE},
    },
    core::battle::{BattleRunReward, BattleTrigger},
};

#[test]
fn test_battle_should_load_from_ron_config() {
    let battle = load_test_battle_result().unwrap();

    assert_eq!(battle.name, "Chimera Scrimmage");
    assert_eq!(battle.max_turn, 20);
    assert_eq!(battle.challenger.chimeras.len(), 4);
    assert_eq!(battle.defender.chimeras.len(), 4);
    assert_eq!(
        battle.leader.as_ref().map(|leader| leader.name.as_str()),
        Some("Field Captain")
    );
    assert_eq!(
        battle
            .leader
            .as_ref()
            .map(|leader| leader.preferred_shop_tags.clone()),
        Some(vec!["support".to_string(), "summon".to_string()])
    );
    assert_eq!(battle.run.active_team_limit, 4);
    assert_eq!(battle.run.opponent_rounds.len(), 3);
    assert_eq!(battle.run.opponent_rounds[0].name, "Opening Match");
    assert_eq!(
        battle.run.opponent_rounds[0].win_rewards[0],
        BattleRunReward::AddGold { amount: 3 }
    );
    assert!(battle.run.opponent_rounds[2].is_boss);
    assert_eq!(battle.run.shop_pool.len(), 5);
    assert!(
        battle
            .ability_database
            .abilities
            .contains_key(&TOUGH_COOKIE)
    );
    assert!(
        battle
            .ability_database
            .abilities
            .contains_key(&KIND_PRAISER)
    );
}

#[test]
fn loaded_abilities_should_preserve_triggers() {
    let battle = load_test_battle_result().unwrap();
    let kind_praiser = battle
        .ability_database
        .abilities
        .get(&KIND_PRAISER)
        .unwrap();

    assert_eq!(kind_praiser.trigger, BattleTrigger::OnKnockdown);
}
