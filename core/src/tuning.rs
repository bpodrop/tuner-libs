use crate::{Cents, Note};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresetId {
    EStandard,
    DropD,
    EbStandard,
    DStandard,
    DropC,
    OpenG,
    OpenD,
    Dadgad,
}

impl PresetId {
    pub const ALL: [PresetId; 8] = [
        PresetId::EStandard,
        PresetId::DropD,
        PresetId::EbStandard,
        PresetId::DStandard,
        PresetId::DropC,
        PresetId::OpenG,
        PresetId::OpenD,
        PresetId::Dadgad,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::EStandard => "e-standard",
            Self::DropD => "drop-d",
            Self::EbStandard => "eb-standard",
            Self::DStandard => "d-standard",
            Self::DropC => "drop-c",
            Self::OpenG => "open-g",
            Self::OpenD => "open-d",
            Self::Dadgad => "dadgad",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::EStandard => "E Standard",
            Self::DropD => "Drop D",
            Self::EbStandard => "Eb Standard",
            Self::DStandard => "D Standard",
            Self::DropC => "Drop C",
            Self::OpenG => "Open G",
            Self::OpenD => "Open D",
            Self::Dadgad => "DADGAD",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "e-standard" | "estandard" | "standard" | "standard-e" => Some(Self::EStandard),
            "drop-d" | "dropd" => Some(Self::DropD),
            "eb-standard" | "ebstandard" | "dsharp-standard" | "d#-standard" => {
                Some(Self::EbStandard)
            }
            "d-standard" | "dstandard" => Some(Self::DStandard),
            "drop-c" | "dropc" => Some(Self::DropC),
            "open-g" | "openg" => Some(Self::OpenG),
            "open-d" | "opend" => Some(Self::OpenD),
            "dadgad" => Some(Self::Dadgad),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetString {
    pub index: u8,
    pub display_number: u8,
    pub label: &'static str,
    pub note: Note,
    pub frequency_hz: f32,
}

impl TargetString {
    pub fn note_name(self) -> &'static str {
        self.label
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningPreset {
    pub id: PresetId,
    pub display_name: &'static str,
    pub strings: [TargetString; 6],
}

impl TuningPreset {
    pub fn string_by_index(&self, index: u8) -> Option<&TargetString> {
        self.strings.iter().find(|string| string.index == index)
    }

    pub fn lowest_frequency_hz(&self) -> f32 {
        self.strings[0].frequency_hz
    }

    pub fn highest_frequency_hz(&self) -> f32 {
        self.strings[self.strings.len() - 1].frequency_hz
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PresetMatch {
    pub preset_id: PresetId,
    pub matched_string: TargetString,
    pub cents_from_target: f32,
    pub absolute_cents: f32,
}

const fn string(
    index: u8,
    display_number: u8,
    label: &'static str,
    midi: i32,
    frequency_hz: f32,
) -> TargetString {
    TargetString {
        index,
        display_number,
        label,
        note: Note::from_midi(midi),
        frequency_hz,
    }
}

pub const E_STANDARD: TuningPreset = TuningPreset {
    id: PresetId::EStandard,
    display_name: "E Standard",
    strings: [
        string(0, 6, "E2", 40, 82.41),
        string(1, 5, "A2", 45, 110.00),
        string(2, 4, "D3", 50, 146.83),
        string(3, 3, "G3", 55, 196.00),
        string(4, 2, "B3", 59, 246.94),
        string(5, 1, "E4", 64, 329.63),
    ],
};

pub const DROP_D: TuningPreset = TuningPreset {
    id: PresetId::DropD,
    display_name: "Drop D",
    strings: [
        string(0, 6, "D2", 38, 73.42),
        string(1, 5, "A2", 45, 110.00),
        string(2, 4, "D3", 50, 146.83),
        string(3, 3, "G3", 55, 196.00),
        string(4, 2, "B3", 59, 246.94),
        string(5, 1, "E4", 64, 329.63),
    ],
};

pub const EB_STANDARD: TuningPreset = TuningPreset {
    id: PresetId::EbStandard,
    display_name: "Eb Standard",
    strings: [
        string(0, 6, "Eb2", 39, 77.78),
        string(1, 5, "Ab2", 44, 103.83),
        string(2, 4, "Db3", 49, 138.59),
        string(3, 3, "Gb3", 54, 185.00),
        string(4, 2, "Bb3", 58, 233.08),
        string(5, 1, "Eb4", 63, 311.13),
    ],
};

pub const D_STANDARD: TuningPreset = TuningPreset {
    id: PresetId::DStandard,
    display_name: "D Standard",
    strings: [
        string(0, 6, "D2", 38, 73.42),
        string(1, 5, "G2", 43, 98.00),
        string(2, 4, "C3", 48, 130.81),
        string(3, 3, "F3", 53, 174.61),
        string(4, 2, "A3", 57, 220.00),
        string(5, 1, "D4", 62, 293.66),
    ],
};

pub const DROP_C: TuningPreset = TuningPreset {
    id: PresetId::DropC,
    display_name: "Drop C",
    strings: [
        string(0, 6, "C2", 36, 65.41),
        string(1, 5, "G2", 43, 98.00),
        string(2, 4, "C3", 48, 130.81),
        string(3, 3, "F3", 53, 174.61),
        string(4, 2, "A3", 57, 220.00),
        string(5, 1, "D4", 62, 293.66),
    ],
};

pub const OPEN_G: TuningPreset = TuningPreset {
    id: PresetId::OpenG,
    display_name: "Open G",
    strings: [
        string(0, 6, "D2", 38, 73.42),
        string(1, 5, "G2", 43, 98.00),
        string(2, 4, "D3", 50, 146.83),
        string(3, 3, "G3", 55, 196.00),
        string(4, 2, "B3", 59, 246.94),
        string(5, 1, "D4", 62, 293.66),
    ],
};

pub const OPEN_D: TuningPreset = TuningPreset {
    id: PresetId::OpenD,
    display_name: "Open D",
    strings: [
        string(0, 6, "D2", 38, 73.42),
        string(1, 5, "A2", 45, 110.00),
        string(2, 4, "D3", 50, 146.83),
        string(3, 3, "F#3", 54, 185.00),
        string(4, 2, "A3", 57, 220.00),
        string(5, 1, "D4", 62, 293.66),
    ],
};

pub const DADGAD: TuningPreset = TuningPreset {
    id: PresetId::Dadgad,
    display_name: "DADGAD",
    strings: [
        string(0, 6, "D2", 38, 73.42),
        string(1, 5, "A2", 45, 110.00),
        string(2, 4, "D3", 50, 146.83),
        string(3, 3, "G3", 55, 196.00),
        string(4, 2, "A3", 57, 220.00),
        string(5, 1, "D4", 62, 293.66),
    ],
};

pub const STANDARD_TUNING: [TargetString; 6] = E_STANDARD.strings;

const ALL_PRESETS: [TuningPreset; 8] = [
    E_STANDARD,
    DROP_D,
    EB_STANDARD,
    D_STANDARD,
    DROP_C,
    OPEN_G,
    OPEN_D,
    DADGAD,
];

pub fn all_presets() -> &'static [TuningPreset] {
    &ALL_PRESETS
}

pub fn default_preset() -> &'static TuningPreset {
    &E_STANDARD
}

pub fn preset_by_id(id: PresetId) -> &'static TuningPreset {
    match id {
        PresetId::EStandard => &E_STANDARD,
        PresetId::DropD => &DROP_D,
        PresetId::EbStandard => &EB_STANDARD,
        PresetId::DStandard => &D_STANDARD,
        PresetId::DropC => &DROP_C,
        PresetId::OpenG => &OPEN_G,
        PresetId::OpenD => &OPEN_D,
        PresetId::Dadgad => &DADGAD,
    }
}

pub fn match_frequency_to_preset(
    measured_hz: f32,
    preset: &TuningPreset,
    max_distance_cents: f32,
) -> Option<PresetMatch> {
    let mut best_match: Option<PresetMatch> = None;

    for target_string in preset.strings {
        let cents_from_target = Cents::between(measured_hz, target_string.frequency_hz)?.value();
        let absolute_cents = cents_from_target.abs();

        let candidate = PresetMatch {
            preset_id: preset.id,
            matched_string: target_string,
            cents_from_target,
            absolute_cents,
        };

        let is_better = best_match
            .as_ref()
            .map(|current| candidate.absolute_cents < current.absolute_cents)
            .unwrap_or(true);

        if is_better {
            best_match = Some(candidate);
        }
    }

    match best_match {
        Some(candidate) if candidate.absolute_cents <= max_distance_cents => Some(candidate),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{PresetId, all_presets, default_preset, match_frequency_to_preset, preset_by_id};

    #[test]
    fn parses_known_preset_ids() {
        assert_eq!(PresetId::parse("e-standard"), Some(PresetId::EStandard));
        assert_eq!(PresetId::parse("drop-d"), Some(PresetId::DropD));
        assert_eq!(PresetId::parse("eb-standard"), Some(PresetId::EbStandard));
        assert_eq!(PresetId::parse("d-standard"), Some(PresetId::DStandard));
        assert_eq!(PresetId::parse("drop-c"), Some(PresetId::DropC));
        assert_eq!(PresetId::parse("open-g"), Some(PresetId::OpenG));
        assert_eq!(PresetId::parse("open-d"), Some(PresetId::OpenD));
        assert_eq!(PresetId::parse("dadgad"), Some(PresetId::Dadgad));
        assert_eq!(PresetId::parse("unknown"), None);
    }

    #[test]
    fn returns_default_preset() {
        assert_eq!(default_preset().id, PresetId::EStandard);
        assert_eq!(preset_by_id(PresetId::DropD).display_name, "Drop D");
        assert_eq!(preset_by_id(PresetId::DropC).display_name, "Drop C");
    }

    #[test]
    fn exposes_all_presets() {
        assert_eq!(all_presets().len(), 8);
    }

    #[test]
    fn matches_frequency_to_closest_string() {
        let preset = preset_by_id(PresetId::EStandard);
        let matched = match_frequency_to_preset(109.8, preset, 100.0).unwrap();

        assert_eq!(matched.matched_string.label, "A2");
        assert!(matched.absolute_cents < 10.0);
    }

    #[test]
    fn matches_drop_c_low_string() {
        let preset = preset_by_id(PresetId::DropC);
        let matched = match_frequency_to_preset(65.5, preset, 100.0).unwrap();

        assert_eq!(matched.matched_string.label, "C2");
    }

    #[test]
    fn rejects_frequency_outside_match_window() {
        let preset = preset_by_id(PresetId::EStandard);
        assert!(match_frequency_to_preset(98.0, preset, 20.0).is_none());
    }
}
