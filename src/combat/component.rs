//! ECS components used by chimeras and work tasks.

use bevy::prelude::*;

use crate::combat::data::TraitId;

#[derive(Component, Debug, Clone, Copy)]
pub struct Chimera;

#[derive(Component, Debug, Clone)]
pub struct Name(pub String);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamId(pub u32);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TeamSlot(pub u32);

#[derive(Component, Debug, Clone)]
pub struct Stats {
    pub max_stamina: i32,
    pub stamina: i32,
    pub efficiency: i32,
    pub resilience: i32,
}

#[derive(Component, Debug, Clone)]
pub struct TraitList {
    pub traits: Vec<TraitId>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct WorkTask;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskOrder(pub u32);

#[derive(Component, Debug, Clone)]
pub struct TaskProgress {
    pub current: i32,
    pub required: i32,
    pub stamina_cost: i32,
    pub cookie_reward: i32,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct TimedEfficiencyBonus {
    pub amount: i32,
    pub remaining_rounds: u32,
}

#[derive(Debug, Clone)]
pub struct TimedTrait {
    pub trait_id: TraitId,
    pub remaining_rounds: u32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct ActiveEffects {
    pub efficiency_bonuses: Vec<TimedEfficiencyBonus>,
    pub temporary_traits: Vec<TimedTrait>,
}
