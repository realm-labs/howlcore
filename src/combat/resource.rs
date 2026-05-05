//! Resources that describe the current work stage and runtime queues.

use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use bevy::prelude::*;

use crate::combat::data::{TraitDef, TraitId};

#[derive(Resource, Debug, Clone)]
pub struct WorkStateData {
    pub round: u32,
    pub max_round: u32,
    pub cookie_score: i32,
    pub completed_tasks: u32,
    pub target_cookie_score: i32,
    pub is_finished: bool,
}

impl Default for WorkStateData {
    fn default() -> Self {
        Self {
            round: 0,
            max_round: 5,
            cookie_score: 0,
            completed_tasks: 0,
            target_cookie_score: 30,
            is_finished: false,
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct TraitDatabase {
    pub traits: HashMap<TraitId, TraitDef>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct WorkLogs(pub Vec<String>);

#[derive(Resource, Debug, Default, Clone)]
pub struct ActionQueue(pub Vec<Entity>);

#[derive(Resource, Debug, Default, Clone)]
pub struct RoundFlow {
    pub round_requested: bool,
    pub round_started_this_update: bool,
    pub queued_rounds: u32,
}

#[derive(Resource, Clone)]
pub struct TerminalInput {
    space_presses: Arc<AtomicU32>,
}

impl TerminalInput {
    pub fn new(space_presses: Arc<AtomicU32>) -> Self {
        Self { space_presses }
    }

    pub fn record_space_press(&self) {
        self.space_presses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn take_space_presses(&self) -> u32 {
        self.space_presses.swap(0, Ordering::Relaxed)
    }
}
