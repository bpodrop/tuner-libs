use std::f32::consts::PI;

use tuner_web_bridge::{new_detector, next_output, push_samples, reset, shutdown};

fn sine_wave(frequency_hz: f32, sample_rate: u32, frame_size: usize, amplitude: f32) -> Vec<f32> {
    (0..frame_size)
        .map(|index| {
            let t = index as f32 / sample_rate as f32;
            amplitude * (2.0 * PI * frequency_hz * t).sin()
        })
        .collect()
}

#[test]
fn detector_lifecycle_and_outputs_follow_contract() {
    let detector_id = new_detector(44_100, 4096, 1024);

    let frame = sine_wave(110.0, 44_100, 4096, 0.8);
    let produced = push_samples(detector_id, &frame);

    assert_eq!(produced, 1);

    let output = next_output(detector_id).expect("expected a pitch output");
    assert!(output.frequency_hz > 80.0 && output.frequency_hz < 140.0);
    assert!(output.confidence > 0.0);

    assert!(next_output(detector_id).is_none());

    assert!(reset(detector_id));
    assert!(next_output(detector_id).is_none());

    assert!(shutdown(detector_id));
    assert!(!shutdown(detector_id));
}

#[test]
fn unknown_detector_operations_are_safe() {
    let unknown_detector_id = u32::MAX;

    assert_eq!(push_samples(unknown_detector_id, &[0.0, 1.0, 0.0]), 0);
    assert!(next_output(unknown_detector_id).is_none());
    assert!(!reset(unknown_detector_id));
    assert!(!shutdown(unknown_detector_id));
}
