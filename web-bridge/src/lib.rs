use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};

use tuner_core::PitchDetectionResult;
use tuner_dsp::{PitchDetector, PitchDetectorConfig};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectionOutput {
    pub frequency_hz: f32,
    pub confidence: f32,
    pub clarity: f32,
    pub rms: f32,
}

impl From<PitchDetectionResult> for DetectionOutput {
    fn from(value: PitchDetectionResult) -> Self {
        Self {
            frequency_hz: value.frequency_hz,
            confidence: value.confidence,
            clarity: value.clarity,
            rms: value.rms,
        }
    }
}

#[derive(Debug)]
struct DetectorHandle {
    detector: PitchDetector,
    config: PitchDetectorConfig,
    pending_samples: Vec<f32>,
    outputs: VecDeque<DetectionOutput>,
}

#[derive(Debug, Default)]
struct DetectorRegistry {
    next_id: u32,
    detectors: HashMap<u32, DetectorHandle>,
}

impl DetectorRegistry {
    fn allocate_id(&mut self) -> u32 {
        self.next_id = self.next_id.wrapping_add(1);
        if self.next_id == 0 {
            self.next_id = 1;
        }

        while self.detectors.contains_key(&self.next_id) {
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
        }

        self.next_id
    }
}

fn registry() -> &'static Mutex<DetectorRegistry> {
    static REGISTRY: OnceLock<Mutex<DetectorRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(DetectorRegistry::default()))
}

fn is_valid_config(sample_rate: u32, frame_size: usize, hop_size: usize) -> bool {
    sample_rate > 0 && frame_size > 0 && hop_size > 0 && hop_size <= frame_size
}

pub fn new_detector(sample_rate: u32, frame_size: usize, hop_size: usize) -> u32 {
    if !is_valid_config(sample_rate, frame_size, hop_size) {
        return 0;
    }

    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");

    let id = lock.allocate_id();
    let config = PitchDetectorConfig {
        sample_rate,
        frame_size,
        hop_size,
        ..PitchDetectorConfig::default()
    };

    lock.detectors.insert(
        id,
        DetectorHandle {
            detector: PitchDetector::new(config),
            config,
            pending_samples: Vec::new(),
            outputs: VecDeque::new(),
        },
    );

    id
}

pub fn push_samples(detector_id: u32, samples: &[f32]) -> usize {
    if detector_id == 0 || samples.is_empty() {
        return 0;
    }

    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return 0;
    };

    if !is_valid_config(
        handle.config.sample_rate,
        handle.config.frame_size,
        handle.config.hop_size,
    ) {
        return 0;
    }

    handle.pending_samples.extend_from_slice(samples);

    let mut produced = 0;
    while handle.pending_samples.len() >= handle.config.frame_size {
        let frame = &handle.pending_samples[..handle.config.frame_size];
        if let Some(output) = handle.detector.detect_pitch(frame) {
            handle.outputs.push_back(output.into());
            produced += 1;
        }

        let advance = handle.config.hop_size.min(handle.config.frame_size).max(1);
        handle.pending_samples.drain(..advance);
    }

    produced
}

pub fn next_output(detector_id: u32) -> Option<DetectionOutput> {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let handle = lock.detectors.get_mut(&detector_id)?;
    handle.outputs.pop_front()
}

pub fn reset(detector_id: u32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    let Some(handle) = lock.detectors.get_mut(&detector_id) else {
        return false;
    };

    handle.detector = PitchDetector::new(handle.config);
    handle.pending_samples.clear();
    handle.outputs.clear();

    true
}

pub fn shutdown(detector_id: u32) -> bool {
    let mut lock = registry()
        .lock()
        .expect("detector registry lock should not be poisoned");
    lock.detectors.remove(&detector_id).is_some()
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_name = new_detector)]
    pub fn wasm_new_detector(sample_rate: u32, frame_size: usize, hop_size: usize) -> u32 {
        super::new_detector(sample_rate, frame_size, hop_size)
    }

    #[wasm_bindgen(js_name = push_samples)]
    pub fn wasm_push_samples(detector_id: u32, samples: Vec<f32>) -> usize {
        super::push_samples(detector_id, &samples)
    }

    #[wasm_bindgen(js_name = next_output)]
    pub fn wasm_next_output(detector_id: u32) -> Option<Vec<f32>> {
        super::next_output(detector_id).map(|output| {
            vec![
                output.frequency_hz,
                output.confidence,
                output.clarity,
                output.rms,
            ]
        })
    }

    #[wasm_bindgen(js_name = reset)]
    pub fn wasm_reset(detector_id: u32) -> bool {
        super::reset(detector_id)
    }

    #[wasm_bindgen(js_name = shutdown)]
    pub fn wasm_shutdown(detector_id: u32) -> bool {
        super::shutdown(detector_id)
    }
}
