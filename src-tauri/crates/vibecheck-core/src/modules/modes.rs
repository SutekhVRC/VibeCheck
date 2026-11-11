//! Processing modes: how an incoming value becomes a motor level.
//!
//! Each mode is a single-input, single-output [`Processor`] with ports `in`
//! and `out`. They are ported from VibeCheck's original mode processor and
//! behave the same, quirks included; the tests pin that behaviour down.
//!
//! These processors expect to be run **once per incoming value** (see
//! [`process_one`](crate::module::process_one)): Smooth averages the last N
//! values received and Rate measures movement between consecutive values.
//! Rate keeps its own wall-clock timer rather than using `dt`, as before.

use std::time::{Duration, Instant};

use crate::event::Event;
use crate::module::{PortSpec, Processor};
use crate::signal::Signal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The motor follows the value. Booleans become 0 or 1.
    Raw,
    /// Averages the last few float values. Ignores booleans.
    Smooth,
    /// The faster the float value moves, the higher the level. Ignores booleans.
    Rate,
    /// A fixed level while "on" (true, or a float of 0.5 or more), 0 otherwise.
    Constant,
}

/// Per-feature settings the modes read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModeSettings {
    /// Smooth: how many values to average (Smooth Level).
    pub smooth_size: usize,
    /// Rate: how much the level drops on each decay step (Rate Level).
    pub rate_decay: f64,
    /// Constant: the level while on (Constant Level).
    pub constant_level: f64,
    /// The feature's Flip Input. Smooth and Rate treat 1.0 as "zero" when set.
    pub flip: bool,
}

/// Builds the processor for `mode`.
pub fn build(mode: Mode, settings: ModeSettings) -> Box<dyn Processor> {
    match mode {
        Mode::Raw => Box::new(RawMode),
        Mode::Smooth => Box::new(SmoothMode::new(settings.smooth_size, settings.flip)),
        Mode::Rate => Box::new(RateMode::new(settings.rate_decay, settings.flip)),
        Mode::Constant => Box::new(ConstantMode::new(settings.constant_level)),
    }
}

fn ports() -> Vec<PortSpec> {
    vec![PortSpec::signal_in("in"), PortSpec::signal_out("out")]
}

/// The "stop" value: 0, or 1 when the feature's input is flipped.
fn is_rest(value: f64, flip: bool) -> bool {
    !flip && value == 0.0 || flip && value == 1.0
}

pub struct RawMode;

impl Processor for RawMode {
    fn ports(&self) -> Vec<PortSpec> {
        ports()
    }

    fn tick(&mut self, ins: &[Signal], _dt: f32, out: &mut [Option<Signal>], _: &mut Vec<Event>) {
        out[0] = Some(Signal::Float(ins[0].as_level()));
    }
}

pub struct SmoothMode {
    size: usize,
    flip: bool,
    queue: Vec<f64>,
}

impl SmoothMode {
    pub fn new(size: usize, flip: bool) -> Self {
        Self {
            size,
            flip,
            queue: Vec::new(),
        }
    }

    fn smooth(&mut self, value: f64) -> Option<f64> {
        let rest = is_rest(value, self.flip);

        if self.queue.len() == self.size {
            if rest {
                // Starting again from rest: forget the collected values
                self.queue.clear();
            } else {
                // Output the mean of the collected values, rounded to
                // hundredths. The incoming value is not part of this average
                // and is dropped; the mean seeds the next window.
                let mean = self.queue.iter().sum::<f64>() / self.queue.len() as f64;
                let smoothed = (mean * 100.0).round() / 100.0;
                self.queue.clear();
                self.queue.push(smoothed);
                return Some(smoothed);
            }
        }

        if rest {
            // Rest always gets through straight away, so the toy can stop
            return Some(value);
        }

        self.queue.push(value);
        None
    }
}

impl Processor for SmoothMode {
    fn ports(&self) -> Vec<PortSpec> {
        ports()
    }

    fn tick(&mut self, ins: &[Signal], _dt: f32, out: &mut [Option<Signal>], _: &mut Vec<Event>) {
        if let Signal::Float(value) = ins[0] {
            out[0] = self.smooth(value).map(Signal::Float);
        }
    }
}

/// How often Rate's level decays while values keep arriving.
const RATE_DECAY_INTERVAL: Duration = Duration::from_millis(150);

pub struct RateMode {
    decay: f64,
    flip: bool,
    level: f64,
    last_input: f64,
    timer: Option<Instant>,
}

impl RateMode {
    pub fn new(decay: f64, flip: bool) -> Self {
        Self {
            decay,
            flip,
            level: 0.0,
            last_input: 0.0,
            timer: None,
        }
    }

    fn rate(&mut self, value: f64, now: Instant) -> f64 {
        // At rest (<= 0, or >= 1 when flipped): reset and send 0. This is 0
        // even when flipped, as before.
        if !self.flip && value <= 0.0 || self.flip && value >= 1.0 {
            self.level = value;
            self.last_input = value;
            return 0.0;
        }

        // Movement since the last value raises the level
        self.level += (value - self.last_input).abs().clamp(0.0, 1.0);
        self.level = self.level.clamp(0.0, 1.0);
        self.last_input = value;

        // Every decay interval, take some off
        if let Some(timer) = self.timer {
            if now.saturating_duration_since(timer) >= RATE_DECAY_INTERVAL {
                self.level = (self.level - self.decay).clamp(0.0, 1.0);
                self.timer = Some(now);
            }
        }
        self.level
    }
}

impl Processor for RateMode {
    fn ports(&self) -> Vec<PortSpec> {
        ports()
    }

    fn tick(&mut self, ins: &[Signal], _dt: f32, out: &mut [Option<Signal>], _: &mut Vec<Event>) {
        let now = Instant::now();
        // The timer starts on the first value of any type, as before
        self.timer.get_or_insert(now);
        if let Signal::Float(value) = ins[0] {
            out[0] = Some(Signal::Float(self.rate(value, now)));
        }
    }
}

pub struct ConstantMode {
    level: f64,
}

impl ConstantMode {
    pub fn new(level: f64) -> Self {
        Self { level }
    }
}

impl Processor for ConstantMode {
    fn ports(&self) -> Vec<PortSpec> {
        ports()
    }

    fn tick(&mut self, ins: &[Signal], _dt: f32, out: &mut [Option<Signal>], _: &mut Vec<Event>) {
        let on = match ins[0] {
            Signal::Float(v) => v >= 0.5,
            Signal::Bool(b) => b,
        };
        out[0] = Some(Signal::Float(if on { self.level } else { 0.0 }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::process_one;

    fn settings() -> ModeSettings {
        ModeSettings {
            smooth_size: 2,
            rate_decay: 0.4,
            constant_level: 0.65,
            flip: false,
        }
    }

    fn run(p: &mut dyn Processor, input: Signal) -> Option<f64> {
        process_one(p, input, 0.0).map(Signal::as_level)
    }

    #[test]
    fn raw_passes_values_through() {
        let mut p = build(Mode::Raw, settings());
        assert_eq!(run(p.as_mut(), Signal::Float(0.42)), Some(0.42));
        assert_eq!(run(p.as_mut(), Signal::Bool(true)), Some(1.0));
        assert_eq!(run(p.as_mut(), Signal::Bool(false)), Some(0.0));
    }

    #[test]
    fn constant_is_on_from_half_or_true() {
        let mut p = build(Mode::Constant, settings());
        assert_eq!(run(p.as_mut(), Signal::Float(0.5)), Some(0.65));
        assert_eq!(run(p.as_mut(), Signal::Float(0.49)), Some(0.0));
        assert_eq!(run(p.as_mut(), Signal::Bool(true)), Some(0.65));
        assert_eq!(run(p.as_mut(), Signal::Bool(false)), Some(0.0));
    }

    #[test]
    fn smooth_matches_original_window_behaviour() {
        let mut p = build(Mode::Smooth, settings());
        // Collecting
        assert_eq!(run(p.as_mut(), Signal::Float(0.2)), None);
        assert_eq!(run(p.as_mut(), Signal::Float(0.4)), None);
        // Full: outputs the mean of the window; this value (0.9) is dropped
        assert_eq!(run(p.as_mut(), Signal::Float(0.9)), Some(0.3));
        // The mean seeds the next window
        assert_eq!(run(p.as_mut(), Signal::Float(0.5)), None);
        assert_eq!(run(p.as_mut(), Signal::Float(0.7)), Some(0.4));
        // Zero always gets through; a part-filled window is kept
        assert_eq!(run(p.as_mut(), Signal::Float(0.0)), Some(0.0));
        assert_eq!(run(p.as_mut(), Signal::Float(0.6)), None);
        // Zero on a full window ([0.4, 0.6]) clears it instead of averaging
        assert_eq!(run(p.as_mut(), Signal::Float(0.0)), Some(0.0));
        assert_eq!(run(p.as_mut(), Signal::Float(0.8)), None);
        assert_eq!(run(p.as_mut(), Signal::Float(0.6)), None);
        assert_eq!(run(p.as_mut(), Signal::Float(0.1)), Some(0.7));
        // Booleans are ignored
        assert_eq!(run(p.as_mut(), Signal::Bool(true)), None);
    }

    #[test]
    fn smooth_with_flip_treats_one_as_rest() {
        let mut p = build(
            Mode::Smooth,
            ModeSettings {
                flip: true,
                ..settings()
            },
        );
        assert_eq!(run(p.as_mut(), Signal::Float(1.0)), Some(1.0));
        assert_eq!(run(p.as_mut(), Signal::Float(0.0)), None);
    }

    #[test]
    fn rate_rises_with_movement_and_resets_at_rest() {
        let mut p = RateMode::new(0.4, false);
        let t0 = Instant::now();
        p.timer = Some(t0);

        // Movement adds up: 0 -> 0.3 -> 0.1 is 0.3 + 0.2
        assert!((p.rate(0.3, t0) - 0.3).abs() < 1e-9);
        assert!((p.rate(0.1, t0) - 0.5).abs() < 1e-9);
        // Capped at 1
        assert_eq!(p.rate(0.9, t0), 1.0);

        // After the decay interval, the decay is taken off and the timer resets
        let later = t0 + RATE_DECAY_INTERVAL;
        assert!((p.rate(0.9, later) - 0.6).abs() < 1e-9);
        assert_eq!(p.timer, Some(later));

        // Rest resets and sends 0
        assert_eq!(p.rate(0.0, later), 0.0);
        assert_eq!(p.level, 0.0);
    }

    #[test]
    fn rate_with_flip_sends_zero_at_one() {
        let mut p = build(
            Mode::Rate,
            ModeSettings {
                flip: true,
                ..settings()
            },
        );
        assert_eq!(run(p.as_mut(), Signal::Float(1.0)), Some(0.0));
        assert_eq!(run(p.as_mut(), Signal::Bool(true)), None);
    }
}
