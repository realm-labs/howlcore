//! Small app-level state markers.

/// High-level mode for future UI expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    WorkPrototype,
}
