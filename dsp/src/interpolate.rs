#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParabolicInterpolation {
    pub refined_tau: f32,
    pub refined_clarity: f32,
}

pub fn parabolic_interpolate(
    nsdf: &[f32],
    peak_index: usize,
    tau_min: usize,
) -> Option<ParabolicInterpolation> {
    if peak_index == 0 || peak_index + 1 >= nsdf.len() {
        return None;
    }

    let left = nsdf[peak_index - 1];
    let center = nsdf[peak_index];
    let right = nsdf[peak_index + 1];
    let denominator = left - 2.0 * center + right;

    if denominator.abs() <= f32::EPSILON {
        return Some(ParabolicInterpolation {
            refined_tau: (tau_min + peak_index) as f32,
            refined_clarity: center,
        });
    }

    let delta = 0.5 * (left - right) / denominator;
    let refined_tau = tau_min as f32 + peak_index as f32 + delta;
    let refined_clarity = center - 0.25 * (left - right) * delta;

    Some(ParabolicInterpolation {
        refined_tau,
        refined_clarity,
    })
}
