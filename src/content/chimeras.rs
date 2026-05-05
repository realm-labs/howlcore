//! Test chimera definitions and their data-driven traits.

use bevy::prelude::*;

use crate::combat::{
    component::{ActiveEffects, Chimera, Name, Stats, TeamId, TeamSlot, TraitList},
    data::{Effect, TargetSelector, TraitDef, TraitId, Trigger},
    resource::TraitDatabase,
};

pub const MISCHIEF_WORK: TraitId = TraitId("mischief_work");
pub const PRESSURE_BOOST: TraitId = TraitId("pressure_boost");
pub const DURABLE_WORKER: TraitId = TraitId("durable_worker");
pub const SOOTHING_CARE: TraitId = TraitId("soothing_care");
pub const LEAD_THE_RACE: TraitId = TraitId("lead_the_race");

pub fn register_test_traits(database: &mut TraitDatabase) {
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
        database.traits.insert(trait_def.id.clone(), trait_def);
    }
}

pub fn spawn_test_chimeras(commands: &mut Commands) {
    spawn_chimera(commands, "Little Villain", 0, 5, 20, MISCHIEF_WORK);
    spawn_chimera(commands, "Pressure Monster", 1, 4, 24, PRESSURE_BOOST);
    spawn_chimera(commands, "Tough Cookie", 2, 3, 32, DURABLE_WORKER);
    spawn_chimera(commands, "Healer", 3, 2, 26, SOOTHING_CARE);
    spawn_chimera(commands, "Rat Race King", 4, 6, 18, LEAD_THE_RACE);
}

fn spawn_chimera(
    commands: &mut Commands,
    name: &'static str,
    slot: u32,
    efficiency: i32,
    stamina: i32,
    trait_id: TraitId,
) {
    commands.spawn((
        Chimera,
        Name(name.to_string()),
        TeamId(1),
        TeamSlot(slot),
        Stats {
            max_stamina: stamina,
            stamina,
            efficiency,
            resilience: 0,
        },
        TraitList {
            traits: vec![trait_id],
        },
        ActiveEffects::default(),
    ));
}
