use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DetectionSmoother {
    capacity: usize,
    max_deviation_cents: f32,
    stable_spread_cents: f32,
    transition_confirm_frames: usize,
    frequencies_hz: VecDeque<f32>,
    pending_frequency_hz: Option<f32>,
    pending_count: usize,
}

impl DetectionSmoother {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            max_deviation_cents: 75.0,
            stable_spread_cents: 10.0,
            transition_confirm_frames: 2,
            frequencies_hz: VecDeque::new(),
            pending_frequency_hz: None,
            pending_count: 0,
        }
    }

    pub fn push(&mut self, frequency_hz: f32) -> f32 {
        if let Some(reference_frequency_hz) = self.average() {
            let deviation_cents = 1200.0 * (frequency_hz / reference_frequency_hz).log2().abs();
            if deviation_cents > self.max_deviation_cents {
                return self.handle_large_frequency_jump(frequency_hz, reference_frequency_hz);
            }
        }

        self.clear_pending_transition();
        self.push_frequency(frequency_hz);
        self.average().unwrap_or(frequency_hz)
    }

    fn handle_large_frequency_jump(
        &mut self,
        frequency_hz: f32,
        reference_frequency_hz: f32,
    ) -> f32 {
        if let Some(pending_frequency_hz) = self.pending_frequency_hz {
            let pending_deviation_cents =
                1200.0 * (frequency_hz / pending_frequency_hz).log2().abs();

            if pending_deviation_cents <= self.max_deviation_cents {
                self.pending_count += 1;
                self.pending_frequency_hz =
                    Some((pending_frequency_hz + frequency_hz) / self.pending_count as f32);

                if self.pending_count >= self.transition_confirm_frames {
                    let accepted_frequency_hz = self.pending_frequency_hz.unwrap_or(frequency_hz);
                    self.frequencies_hz.clear();
                    self.clear_pending_transition();
                    self.push_frequency(accepted_frequency_hz);
                    return accepted_frequency_hz;
                }

                return reference_frequency_hz;
            }
        }

        self.pending_frequency_hz = Some(frequency_hz);
        self.pending_count = 1;
        reference_frequency_hz
    }

    fn push_frequency(&mut self, frequency_hz: f32) {
        if self.frequencies_hz.len() == self.capacity {
            self.frequencies_hz.pop_front();
        }
        self.frequencies_hz.push_back(frequency_hz);
    }

    fn clear_pending_transition(&mut self) {
        self.pending_frequency_hz = None;
        self.pending_count = 0;
    }

    pub fn average(&self) -> Option<f32> {
        if self.frequencies_hz.is_empty() {
            return None;
        }

        Some(self.frequencies_hz.iter().sum::<f32>() / self.frequencies_hz.len() as f32)
    }

    pub fn is_stable(&self) -> bool {
        if self.frequencies_hz.len() < self.capacity {
            return false;
        }

        let min_frequency_hz = self
            .frequencies_hz
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min);
        let max_frequency_hz = self
            .frequencies_hz
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let spread_cents = 1200.0 * (max_frequency_hz / min_frequency_hz).log2().abs();

        spread_cents <= self.stable_spread_cents
    }
}

#[cfg(test)]
mod tests {
    use super::DetectionSmoother;

    #[test]
    fn rejects_large_frequency_outlier() {
        let mut smoother = DetectionSmoother::new(3);

        assert_eq!(smoother.push(110.0), 110.0);
        assert!((smoother.push(110.5) - 110.25).abs() < 0.01);

        let smoothed = smoother.push(180.0);

        assert!((smoothed - 110.25).abs() < 0.01);
    }

    #[test]
    fn accepts_large_frequency_jump_after_consistent_frames() {
        let mut smoother = DetectionSmoother::new(3);

        assert_eq!(smoother.push(82.41), 82.41);
        assert!((smoother.push(82.50) - 82.455).abs() < 0.02);

        let first_jump = smoother.push(146.83);
        let second_jump = smoother.push(146.90);

        assert!((first_jump - 82.455).abs() < 0.02);
        assert!((second_jump - 146.865).abs() < 0.05);
    }

    #[test]
    fn reports_stable_when_window_is_tight() {
        let mut smoother = DetectionSmoother::new(3);

        smoother.push(110.0);
        smoother.push(110.1);
        smoother.push(109.95);

        assert!(smoother.is_stable());
    }

    #[test]
    fn reports_unstable_until_window_is_full() {
        let mut smoother = DetectionSmoother::new(3);

        smoother.push(110.0);
        smoother.push(110.1);

        assert!(!smoother.is_stable());
    }
}
