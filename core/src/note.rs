use crate::cents::Cents;

const NOTE_NAMES: [NoteName; 12] = [
    NoteName::C,
    NoteName::CSharp,
    NoteName::D,
    NoteName::DSharp,
    NoteName::E,
    NoteName::F,
    NoteName::FSharp,
    NoteName::G,
    NoteName::GSharp,
    NoteName::A,
    NoteName::ASharp,
    NoteName::B,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteName {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl NoteName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::CSharp => "C#",
            Self::D => "D",
            Self::DSharp => "D#",
            Self::E => "E",
            Self::F => "F",
            Self::FSharp => "F#",
            Self::G => "G",
            Self::GSharp => "G#",
            Self::A => "A",
            Self::ASharp => "A#",
            Self::B => "B",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    midi: i32,
}

impl Note {
    pub const fn from_midi(midi: i32) -> Self {
        Self { midi }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        let trimmed = label.trim();
        if trimmed.len() < 2 {
            return None;
        }

        let normalized = trimmed.to_ascii_uppercase();
        let split_index =
            normalized.find(|character: char| character.is_ascii_digit() || character == '-')?;
        let (name_part, octave_part) = normalized.split_at(split_index);
        let note_name = match name_part {
            "C" => NoteName::C,
            "C#" => NoteName::CSharp,
            "D" => NoteName::D,
            "D#" => NoteName::DSharp,
            "E" => NoteName::E,
            "F" => NoteName::F,
            "F#" => NoteName::FSharp,
            "G" => NoteName::G,
            "G#" => NoteName::GSharp,
            "A" => NoteName::A,
            "A#" => NoteName::ASharp,
            "B" => NoteName::B,
            _ => return None,
        };
        let octave = octave_part.parse::<i32>().ok()?;
        let midi = (octave + 1) * 12
            + match note_name {
                NoteName::C => 0,
                NoteName::CSharp => 1,
                NoteName::D => 2,
                NoteName::DSharp => 3,
                NoteName::E => 4,
                NoteName::F => 5,
                NoteName::FSharp => 6,
                NoteName::G => 7,
                NoteName::GSharp => 8,
                NoteName::A => 9,
                NoteName::ASharp => 10,
                NoteName::B => 11,
            };

        Some(Self::from_midi(midi))
    }

    pub fn from_frequency(frequency_hz: f32) -> Option<Self> {
        if frequency_hz <= 0.0 || !frequency_hz.is_finite() {
            return None;
        }

        let midi = (69.0 + 12.0 * (frequency_hz / 440.0).log2()).round() as i32;
        Some(Self::from_midi(midi))
    }

    pub fn midi(self) -> i32 {
        self.midi
    }

    pub fn note_name(self) -> NoteName {
        let index = self.midi.rem_euclid(12) as usize;
        NOTE_NAMES[index]
    }

    pub fn octave(self) -> i32 {
        (self.midi / 12) - 1
    }

    pub fn label(self) -> String {
        format!("{}{}", self.note_name().as_str(), self.octave())
    }

    pub fn target_frequency_hz(self) -> f32 {
        440.0 * 2.0_f32.powf((self.midi as f32 - 69.0) / 12.0)
    }

    pub fn estimate(frequency_hz: f32) -> Option<NoteEstimate> {
        let note = Self::from_frequency(frequency_hz)?;
        let target_frequency_hz = note.target_frequency_hz();
        let cents_offset = Cents::between(frequency_hz, target_frequency_hz)?.value();

        Some(NoteEstimate {
            note,
            note_name: note.label(),
            midi: note.midi(),
            target_frequency_hz,
            cents_offset,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteEstimate {
    pub note: Note,
    pub note_name: String,
    pub midi: i32,
    pub target_frequency_hz: f32,
    pub cents_offset: f32,
}

#[cfg(test)]
mod tests {
    use super::{Note, NoteName};
    use crate::cents::Cents;

    fn approx_eq(left: f32, right: f32, epsilon: f32) {
        assert!(
            (left - right).abs() <= epsilon,
            "left={left}, right={right}, epsilon={epsilon}"
        );
    }

    #[test]
    fn maps_440_hz_to_a4() {
        let note = Note::from_frequency(440.0).unwrap();
        assert_eq!(note.midi(), 69);
        assert_eq!(note.note_name(), NoteName::A);
        assert_eq!(note.label(), "A4");
    }

    #[test]
    fn maps_standard_guitar_notes() {
        assert_eq!(Note::from_frequency(82.41).unwrap().label(), "E2");
        assert_eq!(Note::from_frequency(110.0).unwrap().label(), "A2");
    }

    #[test]
    fn parses_note_labels() {
        assert_eq!(Note::from_label("E2").unwrap().midi(), 40);
        assert_eq!(Note::from_label("a#3").unwrap().label(), "A#3");
        assert!(Note::from_label("Hb2").is_none());
    }

    #[test]
    fn maps_midi_to_note_name() {
        assert_eq!(Note::from_midi(40).label(), "E2");
        assert_eq!(Note::from_midi(45).label(), "A2");
        assert_eq!(Note::from_midi(64).label(), "E4");
    }

    #[test]
    fn computes_cents_offset() {
        let cents = Cents::between(441.0, 440.0).unwrap();
        approx_eq(cents.value(), 3.93, 0.05);
    }

    #[test]
    fn creates_note_estimate() {
        let estimate = Note::estimate(329.63).unwrap();
        assert_eq!(estimate.note_name, "E4");
        approx_eq(estimate.target_frequency_hz, 329.63, 0.1);
        approx_eq(estimate.cents_offset, 0.0, 0.1);
    }
}
