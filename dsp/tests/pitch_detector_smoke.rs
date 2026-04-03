use std::f32::consts::PI;

use tuner_dsp::{PitchDetector, PitchDetectorConfig};

fn sine_wave(frequency_hz: f32, sample_rate: u32, frame_size: usize, amplitude: f32) -> Vec<f32> {
    (0..frame_size)
        .map(|index| {
            let t = index as f32 / sample_rate as f32;
            amplitude * (2.0 * PI * frequency_hz * t).sin()
        })
        .collect()
}

#[test]
fn detects_110hz_sine_wave() {
    let config = PitchDetectorConfig::default();
    let frame = sine_wave(110.0, config.sample_rate, config.frame_size, 0.8);
    let mut detector = PitchDetector::new(config);

    let result = detector
        .detect_pitch(&frame)
        .expect("expected pitch detection result for 110Hz sine wave");

    assert!((result.frequency_hz - 110.0).abs() <= 1.0);
}
