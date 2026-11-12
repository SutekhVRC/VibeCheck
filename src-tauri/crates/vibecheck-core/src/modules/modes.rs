//! Processing modes: how an incoming value becomes a motor level.
//!
//! Each mode is a single-input, single-output [`Processor`] with ports `in`
//! and `out`, meant to run on the graph's fixed tick, reading the input's
//! latest value each time.
//!
//! - Raw and Constant are stateless and behave as they always have.
//! - Smooth averages over a **time window** (it used to average the last N
//!   OSC messages, which made it depend on how often VRChat sent values).
//! - Rate decays **every tick**, so the level winds down when the input stops
//!   moving (it used to decay only when a new message arrived, so it could
//!   stay stuck at its last level).

use std::collections::VecDeque;
use std::time::Duration;

use crate::module::{EventOut, PortSpec, Processor};
use crate::signal::Signal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The motor follows the value. Booleans become 0 or 1.
    Raw,
    /// Averages float values over a time window. Ignores booleans.
    Smooth,
    /// The faster the float value moves, the higher the level; decays over
    /// time. Ignores booleans.
    Rate,
    /// A fixed level while "on" (true, or a float of 0.5 or more), 0 otherwise.
    Constant,
}

/// Per-feature settings the modes read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModeSettings {
    /// Smooth: how far back to average.
    pub smooth_window: Duration,
    /// Rate: how much the level drops per 150 ms (Rate Level).
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
        Mode::Smooth => Box::new(SmoothMode::new(settings.smooth_window, settings.flip)),
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

    fn tick(
        &mut self,
        ins: &[Signal],
        _dt: f32,
        out: &mut [Option<Signal>],
        _: &mut Vec<EventOut>,
    ) {
        out[0] = Some(Signal::Float(ins[0].as_level()));
    }
}

pub struct SmoothMode {
    window: Duration,
    flip: bool,
    /// (value, seconds it was held for), oldest first
    samples: VecDeque<(f64, f32)>,
}

impl SmoothMode {
    pub fn new(window: Duration, flip: bool) -> Self {
        Self {
            window,
            flip,
            samples: VecDeque::new(),
        }
    }

    fn smooth(&mut self, value: f64, dt: f32) -> f64 {
        if is_rest(value, self.flip) {
            // Rest always gets through straight away, so the toy can stop,
            // and the average starts again from scratch
            self.samples.clear();
            return value;
        }

        self.samples.push_back((value, dt));
        // Keep just enough samples to cover the window
        let window = self.window.as_secs_f32();
        let mut held: f32 = self.samples.iter().map(|(_, dt)| dt).sum();
        while self.samples.len() > 1 && held - self.samples[0].1 >= window {
            held -= self.samples.pop_front().unwrap().1;
        }

        let mean = if held > 0.0 {
            self.samples
                .iter()
                .map(|(v, dt)| v * *dt as f64)
                .sum::<f64>()
                / held as f64
        } else {
            self.samples.iter().map(|(v, _)| v).sum::<f64>() / self.samples.len() as f64
        };
        // Rounded to hundredths, as before
        (mean * 100.0).round() / 100.0
    }
}

impl Processor for SmoothMode {
    fn ports(&self) -> Vec<PortSpec> {
        ports()
    }

    fn tick(&mut self, ins: &[Signal], dt: f32, out: &mut [Option<Signal>], _: &mut Vec<EventOut>) {
        if let Signal::Float(value) = ins[0] {
            out[0] = Some(Signal::Float(self.smooth(value, dt)));
        }
    }

    fn reset(&mut self) {
        self.samples.clear();
    }
}

/// Rate Level is "how much the level drops per this long".
const RATE_DECAY_PERIOD: f64 = 0.15;

pub struct RateMode {
    decay: f64,
    flip: bool,
    level: f64,
    last_input: f64,
}

impl RateMode {
    pub fn new(decay: f64, flip: bool) -> Self {
        Self {
            decay,
            flip,
            level: 0.0,
            last_input: 0.0,
        }
    }

    fn rate(&mut self, value: f64, dt: f32) -> f64 {
        // At rest (<= 0, or >= 1 when flipped): reset and send 0. This is 0
        // even when flipped, as before.
        if !self.flip && value <= 0.0 || self.flip && value >= 1.0 {
            self.level = 0.0;
            self.last_input = value;
            return 0.0;
        }

        // Movement since the last tick raises the level
        self.level += (value - self.last_input).abs().clamp(0.0, 1.0);
        self.last_input = value;

        // Decay a little every tick, whether or not the input moved
        self.level -= self.decay * dt as f64 / RATE_DECAY_PERIOD;
        self.level = self.level.clamp(0.0, 1.0);
        self.level
    }
}

impl Processor for RateMode {
    fn ports(&self) -> Vec<PortSpec> {
        ports()
    }

    fn tick(&mut self, ins: &[Signal], dt: f32, out: &mut [Option<Signal>], _: &mut Vec<EventOut>) {
        if let Signal::Float(value) = ins[0] {
            out[0] = Some(Signal::Float(self.rate(value, dt)));
        }
    }

    fn reset(&mut self) {
        self.level = 0.0;
        self.last_input = 0.0;
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

    fn tick(
        &mut self,
        ins: &[Signal],
        _dt: f32,
        out: &mut [Option<Signal>],
        _: &mut Vec<EventOut>,
    ) {
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

    /// 100 Hz
    const TICK: f32 = 0.01;

    fn settings() -> ModeSettings {
        ModeSettings {
            smooth_window: Duration::from_millis(100),
            rate_decay: 0.3,
            constant_level: 0.65,
            flip: false,
        }
    }

    fn run(p: &mut dyn Processor, input: Signal) -> Option<f64> {
        process_one(p, input, TICK).map(Signal::as_level)
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
    fn smooth_averages_over_its_window() {
        // 100 ms window at 100 Hz: the last 10 ticks
        let mut p = build(Mode::Smooth, settings());
        assert_eq!(run(p.as_mut(), Signal::Float(0.2)), Some(0.2));
        for _ in 0..4 {
            run(p.as_mut(), Signal::Float(0.2));
        }
        // 5 ticks at 0.2, then a step to 0.8: the average moves part way
        assert_eq!(run(p.as_mut(), Signal::Float(0.8)), Some(0.3));
        // Once the window is all 0.8, the output is 0.8
        let mut last = None;
        for _ in 0..10 {
            last = run(p.as_mut(), Signal::Float(0.8));
        }
        assert_eq!(last, Some(0.8));
        // Booleans are ignored
        assert_eq!(run(p.as_mut(), Signal::Bool(true)), None);
    }

    #[test]
    fn smooth_lets_rest_through_and_restarts() {
        let mut p = build(Mode::Smooth, settings());
        for _ in 0..10 {
            run(p.as_mut(), Signal::Float(0.8));
        }
        assert_eq!(run(p.as_mut(), Signal::Float(0.0)), Some(0.0));
        // Starts again from scratch, not from the old average
        assert_eq!(run(p.as_mut(), Signal::Float(0.4)), Some(0.4));
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
        assert_eq!(run(p.as_mut(), Signal::Float(0.4)), Some(0.4));
        assert_eq!(run(p.as_mut(), Signal::Float(1.0)), Some(1.0));
        assert_eq!(run(p.as_mut(), Signal::Float(0.6)), Some(0.6));
    }

    #[test]
    fn rate_rises_with_movement() {
        // No decay, to see the movement add up
        let mut p = RateMode::new(0.0, false);
        assert!((p.rate(0.3, TICK) - 0.3).abs() < 1e-9);
        assert!((p.rate(0.1, TICK) - 0.5).abs() < 1e-9);
        assert_eq!(p.rate(0.9, TICK), 1.0); // capped
    }

    #[test]
    fn rate_winds_down_when_the_input_stops_moving() {
        // 0.3 per 150 ms = 2.0 per second
        let mut p = RateMode::new(0.3, false);
        let start = p.rate(0.8, TICK);
        assert!(start > 0.7);
        // Input held still for 0.5 s: decays all the way to 0
        let mut level = start;
        for _ in 0..50 {
            level = p.rate(0.8, TICK);
        }
        assert_eq!(level, 0.0);
    }

    #[test]
    fn rate_decay_is_per_150ms() {
        let mut p = RateMode::new(0.3, false);
        p.level = 1.0;
        p.last_input = 0.5;
        // 15 ticks of 10 ms = 150 ms: one Rate Level's worth
        let mut level = 0.0;
        for _ in 0..15 {
            level = p.rate(0.5, TICK);
        }
        assert!((level - 0.7).abs() < 1e-6, "{level}");
    }

    #[test]
    fn rate_resets_at_rest() {
        let mut p = RateMode::new(0.3, false);
        p.rate(0.8, TICK);
        assert_eq!(p.rate(0.0, TICK), 0.0);
        assert_eq!(p.level, 0.0);

        let mut flipped = build(
            Mode::Rate,
            ModeSettings {
                flip: true,
                ..settings()
            },
        );
        assert_eq!(run(flipped.as_mut(), Signal::Float(1.0)), Some(0.0));
        assert_eq!(run(flipped.as_mut(), Signal::Bool(true)), None);
    }
}
