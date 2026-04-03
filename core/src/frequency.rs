#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FrequencyHz(pub f32);

impl FrequencyHz {
    pub fn new(value: f32) -> Option<Self> {
        if value.is_finite() && value > 0.0 {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn get(self) -> f32 {
        self.0
    }
}
