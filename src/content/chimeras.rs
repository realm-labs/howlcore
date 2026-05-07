//! Test chimera definitions and their data-driven traits.

use crate::core::work::{
    ActiveEffects, Chimera, Effect, Stats, TargetSelector, TraitDatabase, TraitDef, TraitId,
    Trigger,
};

pub const MISCHIEF_WORK: TraitId = TraitId("mischief_work");
pub const PRESSURE_BOOST: TraitId = TraitId("pressure_boost");
pub const DURABLE_WORKER: TraitId = TraitId("durable_worker");
pub const SOOTHING_CARE: TraitId = TraitId("soothing_care");
pub const LEAD_THE_RACE: TraitId = TraitId("lead_the_race");

pub fn test_trait_database() -> TraitDatabase {
    let mut database = TraitDatabase::default();
    let traits = [
        TraitDef {
            id: MISCHIEF_WORK,
            name: "Mischief Work",
            trigger: Trigger::OnWork,
            selector: TargetSelector::FrontTask,
            effects: vec![Effect::AdvanceTaskByEfficiency { bonus: 1 }],
        },
        TraitDef {
            id: PRESSURE_BOOST,
            name: "Pressure Boost",
            trigger: Trigger::RoundStart,
            selector: TargetSelector::SelfChimera,
            effects: vec![Effect::AddEfficiency {
                amount: 2,
                duration: 1,
            }],
        },
        TraitDef {
            id: DURABLE_WORKER,
            name: "Durable Worker",
            trigger: Trigger::OnWork,
            selector: TargetSelector::SelfChimera,
            effects: vec![Effect::RestoreStamina { amount: 1 }],
        },
        TraitDef {
            id: SOOTHING_CARE,
            name: "Soothing Care",
            trigger: Trigger::OnWork,
            selector: TargetSelector::LowestStaminaAlly,
            effects: vec![Effect::RestoreStamina { amount: 4 }],
        },
        TraitDef {
            id: LEAD_THE_RACE,
            name: "Lead the Race",
            trigger: Trigger::OnWork,
            selector: TargetSelector::FrontTask,
            effects: vec![Effect::AdvanceTaskByEfficiency { bonus: 3 }],
        },
    ];

    for trait_def in traits {
        database.traits.insert(trait_def.id, trait_def);
    }

    database
}

pub fn test_chimeras() -> Vec<Chimera> {
    vec![
        chimera("Little Villain", 0, 5, 20, MISCHIEF_WORK),
        chimera("Pressure Monster", 1, 4, 24, PRESSURE_BOOST),
        chimera("Tough Cookie", 2, 3, 32, DURABLE_WORKER),
        chimera("Healer", 3, 2, 26, SOOTHING_CARE),
        chimera("Rat Race King", 4, 6, 18, LEAD_THE_RACE),
    ]
}

fn chimera(
    name: &'static str,
    slot: u32,
    efficiency: i32,
    stamina: i32,
    trait_id: TraitId,
) -> Chimera {
    Chimera {
        name: name.to_string(),
        team_id: 1,
        slot,
        is_active: true,
        stats: Stats {
            max_stamina: stamina,
            stamina,
            efficiency,
            resilience: 0,
        },
        traits: vec![trait_id],
        active_effects: ActiveEffects::default(),
    }
}
