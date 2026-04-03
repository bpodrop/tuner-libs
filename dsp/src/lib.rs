//! DSP primitives shared by native and web hosts.
//!
//! `web-audio` marks web-host compatibility for downstream consumers.
//! `native-audio` enables CPAL capture and may be enabled alongside `web-audio`
//! in workspace-wide checks that unify feature sets.

#[cfg(feature = "native-audio")]
pub mod audio;
pub mod interpolate;
pub mod nsdf;
pub mod peak_detection;
pub mod pitch_detector;
pub mod preprocess;
pub mod smoothing;

#[cfg(feature = "native-audio")]
pub use audio::{AudioCapture, AudioInputKind, AudioStart};
pub use pitch_detector::{PitchDetector, PitchDetectorConfig};
