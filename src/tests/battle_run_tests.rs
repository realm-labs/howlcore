use crate::core::battle::{
    BATTLE_LOSS_HEALTH_DAMAGE, BATTLE_RUN_HEALTH, BATTLE_SHOP_SIZE, BATTLE_STARTING_GOLD,
    BATTLE_WIN_GOLD_REWARD, BattleAbilityDatabase, BattleChimera, BattleChimeraOffer,
    BattleDefinition, BattleLeader, BattleLeaderEffect, BattleOpponentRound, BattleRarity,
    BattleRunConfig, BattleRunPhase, BattleRunResult, BattleRunReward, BattleRunState,
    BattleRunStep, BattleShopItem, BattleStats, BattleTeam, TeamSide,
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
        equipment: Vec::new(),
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

fn battle_definition() -> BattleDefinition {
    let defender = team(
        TeamSide::Defender,
        "Defender",
        vec![chimera("Loser", 0, 1, 3)],
    );
    BattleDefinition {
        name: "Run Test".to_string(),
        max_turn: 10,
        challenger: team(
            TeamSide::Challenger,
            "Challenger",
            vec![chimera("Winner", 0, 5, 10)],
        ),
        defender: defender.clone(),
        ability_database: BattleAbilityDatabase::default(),
        rng_seed: 1,
        leader: None,
        run: run_config(defender),
        initial_logs: Vec::new(),
    }
}

fn losing_battle_definition() -> BattleDefinition {
    let defender = team(
        TeamSide::Defender,
        "Defender",
        vec![chimera("Winner", 0, 5, 10)],
    );
    BattleDefinition {
        name: "Run Test".to_string(),
        max_turn: 10,
        challenger: team(
            TeamSide::Challenger,
            "Challenger",
            vec![chimera("Loser", 0, 1, 3)],
        ),
        defender: defender.clone(),
        ability_database: BattleAbilityDatabase::default(),
        rng_seed: 1,
        leader: None,
        run: run_config(defender),
        initial_logs: Vec::new(),
    }
}

fn run_config(defender: BattleTeam) -> BattleRunConfig {
    BattleRunConfig {
        opponent_rounds: vec![
            BattleOpponentRound {
                name: "Round 1".to_string(),
                defender: defender.clone(),
                win_rewards: vec![BattleRunReward::AddGold {
                    amount: BATTLE_WIN_GOLD_REWARD,
                }],
                loss_health_damage: BATTLE_LOSS_HEALTH_DAMAGE,
                is_boss: false,
            },
            BattleOpponentRound {
                name: "Round 2".to_string(),
                defender: defender.clone(),
                win_rewards: vec![BattleRunReward::AddGold {
                    amount: BATTLE_WIN_GOLD_REWARD,
                }],
                loss_health_damage: BATTLE_LOSS_HEALTH_DAMAGE,
                is_boss: false,
            },
            BattleOpponentRound {
                name: "Final Round".to_string(),
                defender,
                win_rewards: vec![BattleRunReward::AddGold {
                    amount: BATTLE_WIN_GOLD_REWARD,
                }],
                loss_health_damage: BATTLE_LOSS_HEALTH_DAMAGE,
                is_boss: true,
            },
        ],
        ..BattleRunConfig::default()
    }
}

#[test]
fn run_should_start_battle_from_draft_phase() {
    let mut run = BattleRunState::from_definition(battle_definition());

    let outcome = run.step().unwrap();

    assert_eq!(outcome.step, BattleRunStep::StartedBattle);
    assert!(outcome.battle_events.is_empty());
    assert!(outcome.run_events.is_empty());
    assert_eq!(run.phase, BattleRunPhase::Battle);
    assert!(run.battle.is_some());
}

#[test]
fn run_should_create_initial_shop_offers() {
    let run = BattleRunState::from_definition(battle_definition());

    assert_eq!(run.draft.gold, BATTLE_STARTING_GOLD);
    assert_eq!(run.draft.shop.len(), BATTLE_SHOP_SIZE);
    assert_eq!(run.health, BATTLE_RUN_HEALTH);
    assert_eq!(run.opponents.len(), 3);
}

#[test]
fn run_should_apply_leader_effects() {
    let mut definition = battle_definition();
    definition.leader = Some(BattleLeader {
        name: "Leader".to_string(),
        preferred_shop_tags: Vec::new(),
        shop_bias_every: 0,
        effects: vec![
            BattleLeaderEffect::AddStartingGold { amount: 2 },
            BattleLeaderEffect::AddRunHealth { amount: 1 },
            BattleLeaderEffect::AddWinGoldReward { amount: 2 },
            BattleLeaderEffect::AddTeamStats { attack: 1, hp: 2 },
            BattleLeaderEffect::AddShopOfferStats { attack: 1, hp: 1 },
        ],
    });

    let run = BattleRunState::from_definition(definition);
    let first_chimera = &run.draft.team.chimeras[0];
    let first_offer = match &run.draft.shop[0] {
        BattleShopItem::Chimera(offer) => offer,
        BattleShopItem::Equipment(_) => panic!("expected first shop item to be a chimera"),
    };

    assert_eq!(
        run.leader.as_ref().map(|leader| leader.name.as_str()),
        Some("Leader")
    );
    assert_eq!(run.draft.gold, BATTLE_STARTING_GOLD + 2);
    assert_eq!(run.health, BATTLE_RUN_HEALTH + 1);
    assert_eq!(
        run.opponents[0].win_rewards[0],
        BattleRunReward::AddGold {
            amount: BATTLE_WIN_GOLD_REWARD + 2
        }
    );
    assert_eq!(first_chimera.stats.attack, 6);
    assert_eq!(first_chimera.stats.max_hp, 12);
    assert_eq!(first_chimera.stats.hp, 12);
    assert_eq!(first_offer.attack, 6);
    assert_eq!(first_offer.hp, 6);
}

#[test]
fn run_should_bias_shop_toward_leader_preferred_tags() {
    let mut definition = battle_definition();
    definition.leader = Some(BattleLeader {
        name: "Summon Leader".to_string(),
        preferred_shop_tags: vec!["summon".to_string()],
        shop_bias_every: 2,
        effects: Vec::new(),
    });
    let mut biased =
        BattleChimeraOffer::new("Summon Target", BattleRarity::White, 1, 3, Vec::new());
    biased.tags.push("summon".to_string());
    definition.run.shop_pool = vec![
        BattleShopItem::Chimera(BattleChimeraOffer::new(
            "Normal A",
            BattleRarity::White,
            1,
            3,
            Vec::new(),
        )),
        BattleShopItem::Chimera(BattleChimeraOffer::new(
            "Normal B",
            BattleRarity::White,
            1,
            3,
            Vec::new(),
        )),
        BattleShopItem::Chimera(biased),
    ];

    let run = BattleRunState::from_definition(definition);
    let names = run
        .draft
        .shop
        .iter()
        .map(|item| item.name().to_string())
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["Normal A", "Summon Target", "Normal B"]);
}

#[test]
fn run_should_use_definition_run_config() {
    let mut definition = battle_definition();
    definition.run = BattleRunConfig {
        starting_gold: 12,
        shop_size: 2,
        active_team_limit: 3,
        health: 5,
        loss_health_damage: 2,
        opponent_rounds: vec![
            BattleOpponentRound {
                name: "Round 1".to_string(),
                defender: team(TeamSide::Defender, "One", vec![chimera("One", 0, 1, 2)]),
                win_rewards: vec![BattleRunReward::AddGold { amount: 4 }],
                loss_health_damage: 2,
                is_boss: false,
            },
            BattleOpponentRound {
                name: "Round 2".to_string(),
                defender: team(TeamSide::Defender, "Two", vec![chimera("Two", 0, 1, 2)]),
                win_rewards: vec![BattleRunReward::AddGold { amount: 4 }],
                loss_health_damage: 2,
                is_boss: true,
            },
        ],
        shop_pool: BattleRunConfig::default().shop_pool,
    };

    let run = BattleRunState::from_definition(definition);

    assert_eq!(run.draft.gold, 12);
    assert_eq!(run.draft.shop.len(), 2);
    assert_eq!(run.health, 5);
    assert_eq!(run.draft.active_team_limit, 3);
    assert_eq!(run.loss_health_damage, 2);
    assert_eq!(run.opponents.len(), 2);
}

#[test]
fn run_should_use_configured_opponent_rounds() {
    let mut definition = battle_definition();
    definition.run.opponent_rounds = vec![
        BattleOpponentRound {
            name: "Qualifier".to_string(),
            defender: team(
                TeamSide::Defender,
                "Qualifier Defender",
                vec![chimera("Weak", 0, 1, 2)],
            ),
            win_rewards: vec![BattleRunReward::AddGold { amount: 7 }],
            loss_health_damage: 1,
            is_boss: false,
        },
        BattleOpponentRound {
            name: "Final".to_string(),
            defender: team(
                TeamSide::Defender,
                "Final Defender",
                vec![chimera("Boss", 0, 2, 3)],
            ),
            win_rewards: vec![BattleRunReward::AddGold { amount: 9 }],
            loss_health_damage: 2,
            is_boss: true,
        },
    ];

    let mut run = BattleRunState::from_definition(definition);

    assert_eq!(run.opponents.len(), 2);
    assert_eq!(run.current_opponent().unwrap().name, "Qualifier");
    assert!(!run.current_opponent().unwrap().is_boss);

    run.start_battle().unwrap();
    assert_eq!(
        run.battle.as_ref().map(|battle| battle.name.as_str()),
        Some("Battle 1 - Qualifier")
    );
    let _outcome = run.step().unwrap();

    assert_eq!(run.draft.gold, BATTLE_STARTING_GOLD + 7);
    assert_eq!(run.current_opponent().unwrap().name, "Final");
    assert!(run.current_opponent().unwrap().is_boss);
}

#[test]
fn run_should_apply_structured_win_rewards() {
    let mut definition = battle_definition();
    definition.run.opponent_rounds = vec![BattleOpponentRound {
        name: "Reward Round".to_string(),
        defender: team(
            TeamSide::Defender,
            "Reward Defender",
            vec![chimera("Weak", 0, 1, 2)],
        ),
        win_rewards: vec![
            BattleRunReward::AddGold { amount: 5 },
            BattleRunReward::HealRun { amount: 1 },
            BattleRunReward::AddShopItem {
                item: BattleRunConfig::default().shop_pool[0].clone(),
            },
        ],
        loss_health_damage: 1,
        is_boss: false,
    }];
    let mut run = BattleRunState::from_definition(definition);
    run.health = BATTLE_RUN_HEALTH - 1;
    run.start_battle().unwrap();

    let outcome = run.step().unwrap();

    assert_eq!(run.draft.gold, BATTLE_STARTING_GOLD + 5);
    assert_eq!(run.health, BATTLE_RUN_HEALTH);
    assert!(
        run.draft
            .shop
            .iter()
            .any(|item| item.name() == "Workaholic")
    );
    assert!(
        outcome
            .timeline
            .lines()
            .iter()
            .any(|line| line.contains("healed"))
    );
    assert!(
        outcome
            .timeline
            .lines()
            .iter()
            .any(|line| line.contains("added Workaholic"))
    );
}

#[test]
fn run_should_refresh_shop_in_draft_phase() {
    let mut run = BattleRunState::from_definition(battle_definition());
    let first_names = run
        .draft
        .shop
        .iter()
        .map(|item| item.name().to_string())
        .collect::<Vec<_>>();

    run.refresh_shop().unwrap();
    let refreshed_names = run
        .draft
        .shop
        .iter()
        .map(|item| item.name().to_string())
        .collect::<Vec<_>>();

    assert_ne!(first_names, refreshed_names);
    assert_eq!(refreshed_names.len(), BATTLE_SHOP_SIZE);
}

#[test]
fn run_should_reward_win_and_return_to_draft_when_defenders_remain() {
    let mut run = BattleRunState::from_definition(battle_definition());
    run.start_battle().unwrap();

    let outcome = run.step().unwrap();

    assert_eq!(
        outcome.step,
        BattleRunStep::BattleResolved {
            winner: Some(TeamSide::Challenger)
        }
    );
    assert!(
        outcome
            .timeline
            .lines()
            .iter()
            .any(|line| line.contains("Run reward"))
    );
    assert_eq!(
        run.draft.gold,
        BATTLE_STARTING_GOLD + BATTLE_WIN_GOLD_REWARD
    );
    assert_eq!(run.wins, 1);
    assert_eq!(run.phase, BattleRunPhase::Draft);
    assert_eq!(run.battle_index, 1);
    assert!(run.battle.is_none());
    assert_eq!(run.draft.shop.len(), BATTLE_SHOP_SIZE);
}

#[test]
fn run_step_should_emit_timeline_snapshots_for_playback() {
    let mut run = BattleRunState::from_definition(battle_definition());
    run.start_battle().unwrap();

    let outcome = run.step().unwrap();
    let damage_frame = outcome
        .timeline
        .frames
        .iter()
        .find(|frame| frame.line.contains("Loser took"))
        .expect("expected defender damage frame");
    let snapshot = damage_frame
        .snapshot
        .as_ref()
        .expect("damage frame should include a battle snapshot");

    assert_eq!(snapshot.state.defender.chimeras[0].stats.hp, 0);
    assert_eq!(run.phase, BattleRunPhase::Draft);
}

#[test]
fn run_should_finish_after_final_defender_is_defeated() {
    let mut run = BattleRunState::from_definition(battle_definition());
    run.opponents.truncate(1);
    run.start_battle().unwrap();

    let _outcome = run.step().unwrap();

    assert_eq!(run.wins, 1);
    assert_eq!(run.phase, BattleRunPhase::Complete);
    assert_eq!(run.result, Some(BattleRunResult::Victory));
}

#[test]
fn run_should_lose_health_after_defeat() {
    let mut run = BattleRunState::from_definition(losing_battle_definition());
    run.start_battle().unwrap();

    let outcome = run.step().unwrap();

    assert_eq!(
        outcome.step,
        BattleRunStep::BattleResolved {
            winner: Some(TeamSide::Defender)
        }
    );
    assert!(
        outcome
            .timeline
            .lines()
            .iter()
            .any(|line| line.contains("Run damage"))
    );
    assert_eq!(run.health, BATTLE_RUN_HEALTH - BATTLE_LOSS_HEALTH_DAMAGE);
    assert_eq!(run.losses, 1);
    assert_eq!(run.phase, BattleRunPhase::Draft);
}

#[test]
fn run_should_complete_when_health_reaches_zero() {
    let mut run = BattleRunState::from_definition(losing_battle_definition());
    run.health = BATTLE_LOSS_HEALTH_DAMAGE;
    run.start_battle().unwrap();

    let _outcome = run.step().unwrap();

    assert_eq!(run.health, 0);
    assert_eq!(run.phase, BattleRunPhase::Complete);
    assert_eq!(run.result, Some(BattleRunResult::Defeat));
}
