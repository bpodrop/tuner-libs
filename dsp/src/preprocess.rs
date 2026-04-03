#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreprocessResult {
    pub rms: f32,
}

pub fn remove_dc_offset(frame: &[f32]) -> Vec<f32> {
    if frame.is_empty() {
        return Vec::new();
    }

    let mean = frame.iter().sum::<f32>() / frame.len() as f32;
    frame.iter().map(|sample| sample - mean).collect()
}

pub fn rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }

    let energy = frame.iter().map(|sample| sample * sample).sum::<f32>() / frame.len() as f32;
    energy.sqrt()
}

pub fn apply_hann_window(frame: &[f32]) -> Vec<f32> {
    let len = frame.len();
    if len <= 1 {
        return frame.to_vec();
    }

    frame
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let phase = (2.0 * core::f32::consts::PI * index as f32) / (len as f32 - 1.0);
            let window = 0.5 * (1.0 - phase.cos());
            sample * window
        })
        .collect()
}

pub fn preprocess_frame(frame: &[f32], apply_hann: bool) -> (Vec<f32>, PreprocessResult) {
    let dc_free = remove_dc_offset(frame);
    let processed = if apply_hann {
        apply_hann_window(&dc_free)
    } else {
        dc_free
    };

    let metrics = PreprocessResult {
        rms: rms(&processed),
    };

    (processed, metrics)
}
