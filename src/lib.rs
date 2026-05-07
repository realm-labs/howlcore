//! Howlcore is a small gameplay prototype for studying work-battle systems.

pub mod app_state;
pub mod content;
pub mod core;
pub mod ui;

use bevy::prelude::*;

use content::{battles::test_battle, stages::test_stage};

/// Builds the Bevy debug UI app.
pub fn build_app() -> App {
    ui::build_app(test_stage(), test_battle())
}

#[cfg(test)]
mod tests;
