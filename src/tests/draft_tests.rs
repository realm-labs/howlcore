use crate::core::battle::{
    BattleChimeraOffer, BattleRarity, CHIMERA_PURCHASE_COST, DraftError, DraftState,
    PurchaseOutcome, TeamSide,
};

fn offer(name: &str) -> BattleChimeraOffer {
    BattleChimeraOffer::new(name, BattleRarity::White, 2, 3, Vec::new())
}

#[test]
fn purchase_should_add_new_chimera_to_next_slot() {
    let mut draft = DraftState::new(6, TeamSide::Challenger, "Draft Team");
    draft.shop.push(offer("Tough Cookie"));
    draft.shop.push(offer("Healer"));

    let outcome = draft.purchase(0).unwrap();

    assert_eq!(
        outcome,
        PurchaseOutcome::Added {
            chimera_name: "Tough Cookie".to_string()
        }
    );
    assert_eq!(draft.gold, 6 - CHIMERA_PURCHASE_COST);
    assert_eq!(draft.shop.len(), 1);
    assert_eq!(draft.team.chimeras[0].name, "Tough Cookie");
    assert_eq!(draft.team.chimeras[0].slot, 0);
}

#[test]
fn purchase_should_reject_when_gold_is_not_enough() {
    let mut draft = DraftState::new(2, TeamSide::Challenger, "Draft Team");
    draft.shop.push(offer("Tough Cookie"));

    let result = draft.purchase(0);

    assert_eq!(
        result,
        Err(DraftError::NotEnoughGold {
            cost: CHIMERA_PURCHASE_COST,
            available: 2
        })
    );
    assert_eq!(draft.shop.len(), 1);
}

#[test]
fn duplicate_purchase_should_merge_stats_and_experience() {
    let mut draft = DraftState::new(9, TeamSide::Challenger, "Draft Team");
    draft.shop.push(offer("Tough Cookie"));
    draft.shop.push(offer("Tough Cookie"));

    draft.purchase(0).unwrap();
    let outcome = draft.purchase(0).unwrap();

    assert_eq!(
        outcome,
        PurchaseOutcome::Merged {
            chimera_name: "Tough Cookie".to_string(),
            level_before: 1,
            level_after: 1
        }
    );
    let chimera = &draft.team.chimeras[0];
    assert_eq!(chimera.stats.attack, 3);
    assert_eq!(chimera.stats.max_hp, 4);
    assert_eq!(chimera.stats.hp, 4);
    assert_eq!(chimera.experience, 1);
}

#[test]
fn duplicate_experience_should_raise_level_two_and_three() {
    let mut draft = DraftState::new(30, TeamSide::Challenger, "Draft Team");
    for _ in 0..6 {
        draft.shop.push(offer("Tough Cookie"));
    }

    draft.purchase(0).unwrap();
    draft.purchase(0).unwrap();
    let level_two = draft.purchase(0).unwrap();
    draft.purchase(0).unwrap();
    draft.purchase(0).unwrap();
    let level_three = draft.purchase(0).unwrap();

    assert_eq!(
        level_two,
        PurchaseOutcome::Merged {
            chimera_name: "Tough Cookie".to_string(),
            level_before: 1,
            level_after: 2
        }
    );
    assert_eq!(
        level_three,
        PurchaseOutcome::Merged {
            chimera_name: "Tough Cookie".to_string(),
            level_before: 2,
            level_after: 3
        }
    );

    let chimera = &draft.team.chimeras[0];
    assert_eq!(chimera.level, 3);
    assert_eq!(chimera.experience, 0);
}
