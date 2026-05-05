//! Events emitted by the work-battle flow.

use bevy::prelude::*;

#[derive(Event, Debug, Clone, Copy)]
pub struct RoundStarted {
    pub round: u32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct WorkActionRequested {
    pub chimera: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct EffectApplied {
    pub description: String,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct TaskCompleted {
    pub task: Entity,
    pub cookie_reward: i32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct CookieGained {
    pub amount: i32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct WorkEnded {
    pub victory: bool,
}
