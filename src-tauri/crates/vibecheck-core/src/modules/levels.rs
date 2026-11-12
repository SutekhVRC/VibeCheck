//! Output level tweaks: Idle Speed, Range and Flip Input.
//!
//! A [`Processor`] with ports `in` and `out`, applied to a feature's level
//! after its processing mode and before the safety layer.

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
    /// Flip Input: `1 - level`, applied last.
    pub flip: bool,
}

impl OutputLevels {
    /// - Exactly 0 becomes Idle Speed.
    /// - Anything else is clamped to the Range.
    /// - With Flip Input, the result becomes `1 - level`, rounded to hundredths.
    pub fn apply(&self, level: f64) -> f64 {
        let level = if level == 0.0 {
            self.idle
        } else {
            // Tolerate a reversed range instead of panicking
            level.clamp(self.min.min(self.max), self.max.max(self.min))
        };
        if self.flip {
            flip(level)
        } else {
            level
        }
    }
}

/// `1 - level`, rounded to hundredths.
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

    fn levels(flip: bool) -> OutputLevels {
        OutputLevels {
            min: 0.2,
            max: 0.8,
            idle: 0.1,
            flip,
        }
    }

    #[test]
    fn zero_is_idle_and_the_rest_is_clamped() {
        let l = levels(false);
        assert_eq!(l.apply(0.0), 0.1);
        assert_eq!(l.apply(0.05), 0.2);
        assert_eq!(l.apply(0.5), 0.5);
        assert_eq!(l.apply(0.95), 0.8);
    }

    #[test]
    fn flip_is_applied_last() {
        let l = levels(true);
        assert_eq!(l.apply(0.0), 0.9); // idle 0.1, flipped
        assert_eq!(l.apply(0.95), 0.2); // clamped to 0.8, flipped
    }

    #[test]
    fn reversed_range_does_not_panic() {
        let l = OutputLevels {
            min: 0.8,
            max: 0.2,
            ..levels(false)
        };
        assert_eq!(l.apply(0.5), 0.5);
    }
}
