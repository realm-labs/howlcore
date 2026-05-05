//! Target references selected by data-driven traits.

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectTarget {
    Chimera(Entity),
    Task(Entity),
    Global,
}
