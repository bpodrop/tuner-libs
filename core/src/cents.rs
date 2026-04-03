#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Cents(pub f32);

impl Cents {
    pub fn between(frequency_hz: f32, target_frequency_hz: f32) -> Option<Self> {
        if frequency_hz <= 0.0 || target_frequency_hz <= 0.0 {
            return None;
        }

        Some(Self(1200.0 * (frequency_hz / target_frequency_hz).log2()))
    }

    pub fn value(self) -> f32 {
        self.0
    }
}
