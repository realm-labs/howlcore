//! Events emitted by the UI-independent work-battle flow.

use crate::core::model::{ChimeraId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    RoundStarted { round: u32 },
    WorkActionRequested { chimera: ChimeraId },
    EffectApplied { description: String },
    TaskCompleted { task: TaskId, cookie_reward: i32 },
    CookieGained { amount: i32 },
    WorkEnded { victory: bool },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoundOutcome {
    pub events: Vec<CombatEvent>,
    pub logs: Vec<String>,
}

impl RoundOutcome {
    pub fn push_log(&mut self, line: impl Into<String>) {
        self.logs.push(line.into());
    }

    pub fn push_event(&mut self, event: CombatEvent) {
        self.events.push(event);
    }
}
