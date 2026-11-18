use std::collections::VecDeque;

/// Sliding window of frames that did a rerun
#[derive(Clone, Debug, Default)]
pub(super) struct Budget {
    frames: VecDeque<u64>,
}

impl Budget {
    /// Whether `frame` may have an extra pass. Records the spend if so.
    /// When refused, containers fall back to the two-frame repaint path.
    pub(super) fn try_spend(&mut self, frame: u64, (max, window): (u32, u32)) -> bool {
        if self.frames.back() == Some(&frame) {
            return true;
        }

        while self
            .frames
            .front()
            .is_some_and(|&f| f + u64::from(window) <= frame)
        {
            self.frames.pop_front();
        }
        if self.frames.len() >= max as usize {
            return false;
        }
        self.frames.push_back(frame);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_max_frames_per_window_then_refuses() {
        let mut b = Budget::default();
        let limit = (2, 10);
        assert!(b.try_spend(0, limit));
        assert!(b.try_spend(1, limit));
        assert!(!b.try_spend(2, limit));
        // Frame 0 leaves the window at frame 10.
        assert!(b.try_spend(10, limit));
    }

    #[test]
    fn charges_once_per_frame() {
        let mut b = Budget::default();
        let limit = (1, 10);
        assert!(b.try_spend(5, limit));
        assert!(
            b.try_spend(5, limit),
            "second container in the same frame is free"
        );
        assert!(!b.try_spend(6, limit));
    }
}
