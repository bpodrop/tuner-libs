#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PeakCandidate {
    pub tau: usize,
    pub clarity: f32,
}

const CLARITY_RELATIVE_CUTOFF: f32 = 0.90;
const MULTIPLE_TAU_TOLERANCE_RATIO: f32 = 0.04;
const HARMONIC_CLARITY_MARGIN: f32 = 0.08;

pub fn find_local_maxima(nsdf: &[f32], tau_min: usize) -> Vec<PeakCandidate> {
    let mut peaks = Vec::new();
    if nsdf.len() < 3 {
        return peaks;
    }

    for index in 1..(nsdf.len() - 1) {
        let previous = nsdf[index - 1];
        let current = nsdf[index];
        let next = nsdf[index + 1];

        if current > 0.0 && current >= previous && current >= next {
            peaks.push(PeakCandidate {
                tau: tau_min + index,
                clarity: current,
            });
        }
    }

    peaks
}

pub fn filter_peak_candidates(
    peaks: &[PeakCandidate],
    min_clarity: f32,
    tau_min: usize,
    tau_max: usize,
) -> Vec<PeakCandidate> {
    peaks
        .iter()
        .copied()
        .filter(|peak| peak.clarity >= min_clarity && peak.tau >= tau_min && peak.tau <= tau_max)
        .collect()
}

pub fn select_best_peak(peaks: &[PeakCandidate]) -> Option<PeakCandidate> {
    let finite_peaks = peaks
        .iter()
        .copied()
        .filter(|peak| peak.clarity.is_finite())
        .collect::<Vec<_>>();

    let strongest_peak = finite_peaks
        .iter()
        .copied()
        .max_by(|left, right| left.clarity.total_cmp(&right.clarity))?;
    let clarity_cutoff = strongest_peak.clarity * CLARITY_RELATIVE_CUTOFF;

    finite_peaks
        .into_iter()
        .filter(|peak| !is_likely_harmonic_peak(*peak, peaks))
        .filter(|peak| peak.clarity >= clarity_cutoff)
        .min_by_key(|peak| peak.tau)
        .or(Some(strongest_peak))
}

fn is_likely_harmonic_peak(candidate: PeakCandidate, peaks: &[PeakCandidate]) -> bool {
    [2.0_f32, 3.0_f32].into_iter().any(|multiplier| {
        let target_tau = candidate.tau as f32 * multiplier;
        let tolerance = target_tau * MULTIPLE_TAU_TOLERANCE_RATIO;

        peaks.iter().copied().any(|other_peak| {
            other_peak.tau > candidate.tau
                && (other_peak.tau as f32 - target_tau).abs() <= tolerance
                && other_peak.clarity >= candidate.clarity + HARMONIC_CLARITY_MARGIN
        })
    })
}

#[cfg(test)]
mod tests {
    use super::{PeakCandidate, select_best_peak};

    #[test]
    fn prefers_earliest_peak_close_to_strongest_peak() {
        let peaks = [
            PeakCandidate {
                tau: 200,
                clarity: 0.91,
            },
            PeakCandidate {
                tau: 100,
                clarity: 0.86,
            },
            PeakCandidate {
                tau: 300,
                clarity: 0.80,
            },
        ];

        let selected = select_best_peak(&peaks).unwrap();

        assert_eq!(selected.tau, 100);
    }

    #[test]
    fn rejects_weak_early_harmonic_when_far_below_strongest_peak() {
        let peaks = [
            PeakCandidate {
                tau: 100,
                clarity: 0.60,
            },
            PeakCandidate {
                tau: 200,
                clarity: 0.92,
            },
        ];

        let selected = select_best_peak(&peaks).unwrap();

        assert_eq!(selected.tau, 200);
    }

    #[test]
    fn skips_candidate_when_double_period_peak_is_significantly_stronger() {
        let peaks = [
            PeakCandidate {
                tau: 135,
                clarity: 0.83,
            },
            PeakCandidate {
                tau: 271,
                clarity: 0.93,
            },
            PeakCandidate {
                tau: 405,
                clarity: 0.84,
            },
        ];

        let selected = select_best_peak(&peaks).unwrap();

        assert_eq!(selected.tau, 271);
    }
}
