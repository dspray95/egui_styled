/// One container's measurement, stored in `ctx.data` under its cache id.
#[derive(Clone, Debug)]
pub(crate) struct Measured<T> {
    pub value: Option<T>,       // None until first measure
    pub streak: u32,            // Consecutive frames with changes
    pub last_change_frame: u64, //cumulative frame number of last change
}

impl<T> Default for Measured<T> {
    fn default() -> Self {
        Self {
            value: None,
            streak: 0,
            last_change_frame: 0,
        }
    }
}

impl<T> Measured<T> {
    // Store new if it differs from stored value, return streak after storing or
    // None if no change
    pub fn update(&mut self, new: T, frame: u64, changed: impl Fn(&T, &T) -> bool) -> Option<u32> {
        if let Some(old) = &self.value
            && !changed(old, &new)
        {
            return None;
        }

        // First measurement is a miss, not a change: it doesn't count towards the streak.
        self.streak = if self.value.is_none() {
            0
        } else {
            match frame.saturating_sub(self.last_change_frame) {
                0 => self.streak,
                1 => self.streak.saturating_add(1),
                _ => 1,
            }
        };
        self.value = Some(new);
        self.last_change_frame = frame;
        Some(self.streak)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn differs(a: &f32, b: &f32) -> bool {
        (a - b).abs() > 0.5
    }

    #[test]
    fn first_measurement_is_reported_but_not_counted() {
        let mut m = Measured::default();
        assert_eq!(m.update(10.0, 0, differs), Some(0));
        assert_eq!(m.update(20.0, 1, differs), Some(1));
    }

    #[test]
    fn same_frame_second_pass_keeps_streak() {
        let mut m = Measured::default();
        m.update(10.0, 5, differs);
        m.update(20.0, 6, differs);
        assert_eq!(m.update(30.0, 6, differs), Some(1));
        assert_eq!(m.update(30.0, 6, differs), None);
    }

    #[test]
    fn consecutive_frames_grow_streak_and_a_gap_resets_it() {
        let mut m = Measured::default();
        m.update(1.0, 10, differs);
        m.update(2.0, 11, differs);
        assert_eq!(m.update(3.0, 12, differs), Some(2));
        // Quiet frame 13, change on 14.
        assert_eq!(m.update(3.0, 13, differs), None);
        assert_eq!(m.update(4.0, 14, differs), Some(1));
    }

    #[test]
    fn change_within_epsilon_is_ignored() {
        let mut m = Measured::default();
        m.update(10.0, 1, differs);
        assert_eq!(m.update(10.3, 2, differs), None);
    }
}
