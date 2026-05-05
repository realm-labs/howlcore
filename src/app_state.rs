//! Small app-level state markers.
//!
//! The first version is terminal-driven, so most gameplay state lives in combat resources.

/// High-level mode for future UI expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    WorkPrototype,
}
