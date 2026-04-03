use tuner_core::PitchDetectionResult;

use crate::interpolate::parabolic_interpolate;
use crate::nsdf::compute_nsdf;
use crate::peak_detection::{filter_peak_candidates, find_local_maxima, select_best_peak};
use crate::preprocess::preprocess_frame;
use crate::smoothing::DetectionSmoother;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchDetectorConfig {
    pub sample_rate: u32,
    pub frame_size: usize,
    pub hop_size: usize,
    pub min_frequency_hz: f32,
    pub max_frequency_hz: f32,
    pub min_rms: f32,
    pub min_clarity: f32,
    pub smoothing_window_size: usize,
    pub apply_hann_window: bool,
}

impl Default for PitchDetectorConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44_100,
            frame_size: 4096,
            hop_size: 1024,
            min_frequency_hz: 60.0,
            max_frequency_hz: 400.0,
            min_rms: 0.01,
            min_clarity: 0.60,
            smoothing_window_size: 3,
            apply_hann_window: true,
        }
    }
}

impl PitchDetectorConfig {
    pub fn tau_range(&self) -> (usize, usize) {
        let tau_min = (self.sample_rate as f32 / self.max_frequency_hz).floor() as usize;
        let tau_max = (self.sample_rate as f32 / self.min_frequency_hz).ceil() as usize;
        (tau_min.max(1), tau_max.max(1))
    }
}

#[derive(Debug, Clone)]
pub struct PitchDetector {
    config: PitchDetectorConfig,
    smoother: DetectionSmoother,
}

impl PitchDetector {
    pub fn new(config: PitchDetectorConfig) -> Self {
        Self {
            smoother: DetectionSmoother::new(config.smoothing_window_size),
            config,
        }
    }

    pub fn config(&self) -> &PitchDetectorConfig {
        &self.config
    }

    pub fn detect_pitch(&mut self, frame: &[f32]) -> Option<PitchDetectionResult> {
        if frame.len() < self.config.frame_size {
            return None;
        }

        let (tau_min, tau_max) = self.config.tau_range();
        if tau_max >= frame.len() {
            return None;
        }

        let (processed, metrics) = preprocess_frame(frame, self.config.apply_hann_window);
        if metrics.rms < self.config.min_rms {
            return None;
        }

        let nsdf = compute_nsdf(&processed, tau_min, tau_max);
        let peaks = find_local_maxima(&nsdf, tau_min);
        let filtered_peaks =
            filter_peak_candidates(&peaks, self.config.min_clarity, tau_min, tau_max);
        let peak = select_best_peak(&filtered_peaks)?;
        let peak_index = peak.tau.checked_sub(tau_min)?;

        let interpolation = parabolic_interpolate(&nsdf, peak_index, tau_min)?;
        if interpolation.refined_tau <= 0.0 {
            return None;
        }

        let raw_frequency_hz = self.config.sample_rate as f32 / interpolation.refined_tau;
        if raw_frequency_hz < self.config.min_frequency_hz
            || raw_frequency_hz > self.config.max_frequency_hz
        {
            return None;
        }

        let smoothed_frequency_hz = self.smoother.push(raw_frequency_hz);

        Some(PitchDetectionResult {
            frequency_hz: smoothed_frequency_hz,
            confidence: interpolation.refined_clarity.clamp(0.0, 1.0),
            clarity: interpolation.refined_clarity,
            rms: metrics.rms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PitchDetector, PitchDetectorConfig};
    use std::f32::consts::PI;

    fn sine_wave(
        frequency_hz: f32,
        sample_rate: u32,
        frame_size: usize,
        amplitude: f32,
    ) -> Vec<f32> {
        (0..frame_size)
            .map(|index| {
                let t = index as f32 / sample_rate as f32;
                amplitude * (2.0 * PI * frequency_hz * t).sin()
            })
            .collect()
    }

    fn guitar_like_wave(
        frequency_hz: f32,
        sample_rate: u32,
        frame_size: usize,
        fundamental_amplitude: f32,
    ) -> Vec<f32> {
        (0..frame_size)
            .map(|index| {
                let t = index as f32 / sample_rate as f32;
                let envelope = (1.0 - index as f32 / frame_size as f32).max(0.2);
                let fundamental =
                    envelope * fundamental_amplitude * (2.0 * PI * frequency_hz * t).sin();
                let second_harmonic = envelope * 0.55 * (2.0 * PI * frequency_hz * 2.0 * t).sin();
                let third_harmonic = envelope * 0.30 * (2.0 * PI * frequency_hz * 3.0 * t).sin();
                let pick_transient = if index < 64 {
                    0.15 * (2.0 * PI * 1_800.0 * t).sin()
                } else {
                    0.0
                };
                let pseudo_noise =
                    0.015 * (2.0 * PI * 733.0 * t).sin() + 0.01 * (2.0 * PI * 1_177.0 * t).sin();

                fundamental + second_harmonic + third_harmonic + pick_transient + pseudo_noise
            })
            .collect()
    }

    fn noisy_frame(frame_size: usize, amplitude: f32) -> Vec<f32> {
        let mut state = 0x1234_5678_u32;

        (0..frame_size)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let normalized = (state as f32 / u32::MAX as f32) * 2.0 - 1.0;
                amplitude * normalized
            })
            .collect()
    }

    fn approx_eq(left: f32, right: f32, epsilon: f32) {
        assert!(
            (left - right).abs() <= epsilon,
            "left={left}, right={right}, epsilon={epsilon}"
        );
    }

    #[test]
    fn detects_a2_from_sine_wave() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(110.0, config.sample_rate, config.frame_size, 0.8);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();
        approx_eq(result.frequency_hz, 110.0, 1.0);
        assert!(result.clarity >= config.min_clarity);
    }

    #[test]
    fn detects_low_e_string_from_sine_wave() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(82.41, config.sample_rate, config.frame_size, 0.8);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();
        approx_eq(result.frequency_hz, 82.41, 1.0);
    }

    #[test]
    fn detects_high_e_string_from_sine_wave() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(329.63, config.sample_rate, config.frame_size, 0.8);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();
        approx_eq(result.frequency_hz, 329.63, 1.5);
    }

    #[test]
    fn rejects_silence() {
        let config = PitchDetectorConfig::default();
        let frame = vec![0.0; config.frame_size];
        assert!(PitchDetector::new(config).detect_pitch(&frame).is_none());
    }

    #[test]
    fn rejects_too_low_amplitude() {
        let config = PitchDetectorConfig::default();
        let frame = sine_wave(110.0, config.sample_rate, config.frame_size, 0.001);
        assert!(PitchDetector::new(config).detect_pitch(&frame).is_none());
    }

    #[test]
    fn detects_fundamental_on_harmonic_rich_low_e_signal() {
        let config = PitchDetectorConfig::default();
        let frame = guitar_like_wave(82.41, config.sample_rate, config.frame_size, 0.45);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();

        approx_eq(result.frequency_hz, 82.41, 1.5);
    }

    #[test]
    fn detects_fundamental_on_harmonic_rich_d3_signal() {
        let config = PitchDetectorConfig::default();
        let frame = guitar_like_wave(146.83, config.sample_rate, config.frame_size, 0.38);
        let result = PitchDetector::new(config).detect_pitch(&frame).unwrap();

        approx_eq(result.frequency_hz, 146.83, 1.5);
    }

    #[test]
    fn rejects_non_periodic_noise() {
        let config = PitchDetectorConfig::default();
        let frame = noisy_frame(config.frame_size, 0.12);

        assert!(PitchDetector::new(config).detect_pitch(&frame).is_none());
    }
}
