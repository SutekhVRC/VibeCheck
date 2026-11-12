//! Output level tweaks: Idle Speed and Range.
//!
//! A [`Processor`] with ports `in` and `out`, applied to a feature's level
//! after its processing mode and before the safety layer.
//!
//! Flip Input is not applied here: levels stay intensities (0 = off) through
//! the safety layer, so the master intensity means the same for every toy, and
//! the output flips them last (see [`flip`]).

use crate::module::{EventOut, PortSpec, Processor};
use crate::signal::Signal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutputLevels {
    /// Range minimum, 0 to 1.
    pub min: f64,
    /// Range maximum, 0 to 1.
    pub max: f64,
    /// Idle Speed: the level used when the input is exactly 0.
    pub idle: f64,
}

impl OutputLevels {
    /// - Exactly 0 becomes Idle Speed.
    /// - Anything else is clamped to the Range.
    pub fn apply(&self, level: f64) -> f64 {
        if level == 0.0 {
            self.idle
        } else {
            // Tolerate a reversed range instead of panicking
            level.clamp(self.min.min(self.max), self.max.max(self.min))
        }
    }
}

/// Flip Input for toys whose motors run backwards (0 = full power):
/// `1 - level`, rounded to hundredths. Outputs apply this last, after the
/// safety layer.
pub fn flip(level: f64) -> f64 {
    ((1.0 - level) * 100.0).round() / 100.0
}

impl Processor for OutputLevels {
    fn ports(&self) -> Vec<PortSpec> {
        vec![PortSpec::signal_in("in"), PortSpec::signal_out("out")]
    }

    fn tick(
        &mut self,
        ins: &[Signal],
        _dt: f32,
        out: &mut [Option<Signal>],
        _: &mut Vec<EventOut>,
    ) {
        out[0] = Some(Signal::Float(self.apply(ins[0].as_level())));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels() -> OutputLevels {
        OutputLevels {
            min: 0.2,
            max: 0.8,
            idle: 0.1,
        }
    }

    #[test]
    fn zero_is_idle_and_the_rest_is_clamped() {
        let l = levels();
        assert_eq!(l.apply(0.0), 0.1);
        assert_eq!(l.apply(0.05), 0.2);
        assert_eq!(l.apply(0.5), 0.5);
        assert_eq!(l.apply(0.95), 0.8);
    }

    #[test]
    fn flip_inverts_and_rounds() {
        assert_eq!(flip(0.1), 0.9);
        assert_eq!(flip(0.8), 0.2);
        assert_eq!(flip(1.0), 0.0);
        assert_eq!(flip(0.333), 0.67);
    }

    #[test]
    fn reversed_range_does_not_panic() {
        let l = OutputLevels {
            min: 0.8,
            max: 0.2,
            ..levels()
        };
        assert_eq!(l.apply(0.5), 0.5);
    }
}
