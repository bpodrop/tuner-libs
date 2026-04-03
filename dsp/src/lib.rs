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
