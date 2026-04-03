pub mod cents;
pub mod frequency;
pub mod note;
pub mod tuning;

pub use cents::Cents;
pub use frequency::FrequencyHz;
pub use note::{Note, NoteEstimate, NoteName};
pub use tuning::{
    DROP_D, E_STANDARD, PresetId, PresetMatch, STANDARD_TUNING, TargetString, TuningPreset,
    all_presets, default_preset, match_frequency_to_preset, preset_by_id,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasuredPitch {
    pub frequency_hz: f32,
    pub confidence: f32,
    pub clarity: f32,
    pub rms: f32,
}

pub type PitchDetectionResult = MeasuredPitch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunerMode {
    Chromatic,
    Preset(PresetId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiState {
    NoSignal,
    Searching,
    Unstable,
    TooLow,
    InTune,
    TooHigh,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuningTarget {
    pub note_name: String,
    pub frequency_hz: f32,
    pub preset_id: Option<PresetId>,
    pub string: Option<TargetString>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TunerOutput {
    pub mode: TunerMode,
    pub measured_frequency_hz: Option<f32>,
    pub confidence: f32,
    pub detected_note: Option<NoteEstimate>,
    pub display_cents: Option<f32>,
    pub target: Option<TuningTarget>,
    pub ui_state: UiState,
}
