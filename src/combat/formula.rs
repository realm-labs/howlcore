//! Small formulas used by the resolver.

/// Clamps stamina into the valid range.
pub fn clamp_stamina(value: i32, max_stamina: i32) -> i32 {
    value.clamp(0, max_stamina)
}

/// The base amount of task progress a chimera contributes when working.
pub fn base_work_progress(efficiency: i32) -> i32 {
    efficiency.max(0)
}
