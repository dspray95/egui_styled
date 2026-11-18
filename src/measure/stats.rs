use egui::Id;

/// Counters for debugging and tuning. Read with [`crate::Styled::measure_stats`].
#[derive(Clone, Debug, Default)]
pub struct MeasureStats {
    /// Renders of a never-measured container (drawn invisibly).
    pub misses: u64,
    /// Changes settled by a same-frame extra pass.
    pub discards: u64,
    /// Changes that fell back to a repaint (blank or stale for one frame).
    pub repaints: u64,
    /// Containers that hit `max_changes` and stopped requesting passes.
    pub freezes: u64,
    /// Kind and id of the last container to freeze.
    pub last_freeze: Option<(&'static str, Id)>,
}
