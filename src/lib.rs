//! Howlcore is a small Bevy gameplay prototype for studying work-battle systems.

pub mod app_state;
pub mod combat;
pub mod content;

use bevy::prelude::*;
use combat::system::{
    build_action_queue_system, check_work_end_system, end_round_system,
    execute_action_queue_system, flush_work_logs_system, read_space_input_system,
    setup_terminal_input_system, start_round_system,
};
use content::stages::setup_test_stage_system;

/// Builds the Bevy app used by both `cargo run` and tests that want a full schedule.
pub fn build_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_event::<combat::event::RoundStarted>()
        .add_event::<combat::event::WorkActionRequested>()
        .add_event::<combat::event::EffectApplied>()
        .add_event::<combat::event::TaskCompleted>()
        .add_event::<combat::event::CookieGained>()
        .add_event::<combat::event::WorkEnded>()
        .init_resource::<combat::resource::WorkStateData>()
        .init_resource::<combat::resource::TraitDatabase>()
        .init_resource::<combat::resource::WorkLogs>()
        .init_resource::<combat::resource::ActionQueue>()
        .init_resource::<combat::resource::RoundFlow>()
        .configure_sets(
            Update,
            (
                combat::system::CombatSet::Input,
                combat::system::CombatSet::StartRound,
                combat::system::CombatSet::BuildActionQueue,
                combat::system::CombatSet::ExecuteAction,
                combat::system::CombatSet::ResolveEffect,
                combat::system::CombatSet::EndRound,
                combat::system::CombatSet::CheckEnd,
                combat::system::CombatSet::Log,
            )
                .chain(),
        )
        .add_systems(
            Startup,
            (setup_terminal_input_system, setup_test_stage_system).chain(),
        )
        .add_systems(
            Update,
            read_space_input_system.in_set(combat::system::CombatSet::Input),
        )
        .add_systems(
            Update,
            start_round_system.in_set(combat::system::CombatSet::StartRound),
        )
        .add_systems(
            Update,
            build_action_queue_system.in_set(combat::system::CombatSet::BuildActionQueue),
        )
        .add_systems(
            Update,
            execute_action_queue_system.in_set(combat::system::CombatSet::ExecuteAction),
        )
        .add_systems(
            Update,
            end_round_system.in_set(combat::system::CombatSet::EndRound),
        )
        .add_systems(
            Update,
            check_work_end_system.in_set(combat::system::CombatSet::CheckEnd),
        )
        .add_systems(
            Update,
            flush_work_logs_system.in_set(combat::system::CombatSet::Log),
        );

    app
}

#[cfg(test)]
mod tests;
