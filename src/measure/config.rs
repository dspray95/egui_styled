/// Tuning for measured containers. Set with [`crate::Styled::set_measure_config`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasureConfig {
    /// Rerun the pass when a measurement changes, so it is never shown stale or blank.
    /// UI code then runs twice on those frames, and the first pass is discarded.
    pub multipass: bool,
    /// Consecutive changing frames a container may rerun before falling back to a repaint.
    pub max_streak: u32,
    /// Consecutive changing frames before a container stops requesting passes or repaints.
    pub max_changes: u32,
    /// At most `budget.0` rerun frames in any `budget.1`-frame window, across the context.
    pub budget: (u32, u32),
}

impl Default for MeasureConfig {
    fn default() -> Self {
        Self {
            multipass: true,
            max_streak: 1,
            max_changes: 180,
            budget: (4, 16),
        }
    }
}
