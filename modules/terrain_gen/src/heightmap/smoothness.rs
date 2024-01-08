use bevy::prelude::*;

#[derive(Reflect, Default, Debug, Clone, PartialEq, Eq)]
#[reflect(Default)]
pub enum SmoothFunction {
    Identity,
    #[default]
    Smoothstep,
    SmoothStart,
    SmoothEnd,
    UnitIdentity,
    SquaredIdentity,
    Absolute,
}

impl SmoothFunction {
    pub fn get_smoothed(&self, value: f64, constant: f64) -> f64 {
        match self {
            SmoothFunction::Identity => value,
            SmoothFunction::Smoothstep => {
                if value > constant {
                    value - constant / 2.
                } else {
                    value * value * value * (1. - value * 0.5 / constant) / constant / constant
                }
            }
            SmoothFunction::SmoothStart => value * value * constant,
            SmoothFunction::SmoothEnd => constant - (1.0 - value) * (1.0 - value),
            SmoothFunction::UnitIdentity => value * value * (constant - value),
            SmoothFunction::SquaredIdentity => (value * value + constant).sqrt(),
            SmoothFunction::Absolute => (value - constant).abs(),
        }
    }
}
