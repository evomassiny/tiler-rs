/// Enum that describes how the
/// transcription from a value to a color is done.
pub enum Scale {
    /// Linearly translate a value range to a color range
    Linear { min: f32, max: f32},
    /// Log (ish) translation from a value range to a color range
    Log { min: f32, max: f32},
    /// Exponantial (ish) translation from a value range to a color range
    Exponential { min: f32, max: f32},
    /// Apply no transformation
    Equal,
}

#[inline] // Called once per pixel, I believe it make sense to inline it (might be wrong)
/// Describe how a value will be translated to a color domain
///
/// Returns a float in [0; 1]
///
pub fn normalize(scale: &Scale, value: f32) -> f32 {
    match scale {
        &Scale::Linear{min, max} => {
            if value < min {
                0.
            } else if value > max {
                1.
            } else {
                (value - min) / (max - min)
            }
        },
        &Scale::Log{min, max} => {
            if value < min {
                0.
            } else if value > max {
                1.
            } else {
                (1. + value - min).log(10.) / (1. + max - min).log(10.)
            }
        },
        &Scale::Exponential {min, max} => {
            if value <= min {
                0.
            } else if value >= max {
                1.
            } else {
                ((value - min) / (max - min)).exp() / (1_f32).exp()
            }
        },
        &Scale::Equal => { value },
    }
}
