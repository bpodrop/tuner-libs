use tuner_core::{Note, NoteName};

#[test]
fn maps_440_hz_to_a4() {
    let note = Note::from_frequency(440.0).expect("440 Hz should map to a note");
    assert_eq!(note.midi(), 69);
    assert_eq!(note.note_name(), NoteName::A);
    assert_eq!(note.label(), "A4");
}
