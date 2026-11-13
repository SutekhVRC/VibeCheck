//! Processing modes: how an incoming value becomes a motor level.
//!
//! Each mode is a single-input, single-output [`Processor`] with ports `in`
//! and `out`, meant to run on the graph's fixed tick, reading the input's
//! latest value each time.
//!
//! - Raw and Constant are stateless and behave as they always have.
//! - Smooth averages over a **time window** (it used to average the last N
//!   OSC messages, which made it depend on how often VRChat sent values).
//! - Rate follows **how fast** the input moves (stroke speed), averaged over a
//!   moment, ignores jitter, and winds down smoothly when the movement stops.
//!   It used to add up raw movement, dropped to 0 whenever the input reached
//!   0, and wound down in a straight line.

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
    /// The faster the float value moves, the higher the level; winds down
    /// when it stops moving. Ignores booleans.
    Rate,
    /// A fixed level while "on" (true, or a float of 0.5 or more), 0 otherwise.
    Constant,
}

/// Per-feature settings the modes read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModeSettings {
    /// Smooth: how far back to average.
    pub smooth_window: Duration,
    /// Rate: how quickly the level winds down once movement stops (the
    /// feature's Rate Level / wind-down speed). Higher stops sooner.
    pub rate_wind_down: f64,
    /// Rate: how strongly speed turns into level. 1 gives full power at
    /// about one full in-and-out stroke per second.
    pub rate_sensitivity: f64,
    /// Constant: the level while on (Constant Level).
    pub constant_level: f64,
    /// The feature's Flip Input. Smooth treats 1.0 as "zero" when set. Rate
    /// doesn't need it: speed is the same either way.
    pub flip: bool,
}

/// Builds the processor for `mode`.
pub fn build(mode: Mode, settings: ModeSettings) -> Box<dyn Processor> {
    match mode {
        Mode::Raw => Box::new(RawMode),
        Mode::Smooth => Box::new(SmoothMode::new(settings.smooth_window, settings.flip)),
        Mode::Rate => Box::new(RateMode::new(
            settings.rate_sensitivity,
            settings.rate_wind_down,
        )),
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

/// Speed (in full ranges per second) that gives full power at sensitivity 1:
/// about one full in-and-out stroke per second.
const RATE_FULL_SPEED: f64 = 2.0;
/// How long speed is averaged over (time constant, seconds). Smooths out
/// values arriving in bursts (OSC is slower than the graph tick).
const RATE_SPEED_SMOOTHING: f64 = 0.15;
/// Movement smaller than this, from the last position that counted, is
/// jitter and ignored. Slow real movement still counts, in steps of this.
const RATE_DEADBAND: f64 = 0.02;
/// Wind-down speed W means the level falls to about a third (1/e) in
/// `RATE_WIND_DOWN_PERIOD / W` seconds. The old Rate Level dropped the level
/// by W per this long, so saved values feel about the same.
const RATE_WIND_DOWN_PERIOD: f64 = 0.15;
/// Levels below this are sent as 0, so the toy actually stops
const RATE_FLOOR: f64 = 0.01;

/// The faster the input moves, the higher the level.
///
/// - **Speed:** distance moved per second, averaged over a short moment, as a
///   share of `RATE_FULL_SPEED`, times the sensitivity.
/// - **Jitter:** movement within `RATE_DEADBAND` of the last counted position
///   doesn't count.
/// - **Wind-down:** when the speed drops, the level eases off exponentially
///   rather than following it straight down.
/// - **No rest reset:** reaching 0 (or 1) is just a position, so stroking
///   that pulls all the way out keeps its rhythm.
/// - The first value after creation or reset only sets the starting
///   position, so a jump to the first value isn't counted as movement.
pub struct RateMode {
    sensitivity: f64,
    wind_down: f64,
    /// Last position that counted as movement
    anchor: Option<f64>,
    /// Smoothed speed, full ranges per second
    speed: f64,
    level: f64,
}

impl RateMode {
    pub fn new(sensitivity: f64, wind_down: f64) -> Self {
        Self {
            sensitivity: if sensitivity.is_finite() {
                sensitivity.max(0.0)
            } else {
                1.0
            },
            wind_down: if wind_down.is_finite() {
                wind_down.max(0.01)
            } else {
                0.4
            },
            anchor: None,
            speed: 0.0,
            level: 0.0,
        }
    }

    fn rate(&mut self, value: f64, dt: f32) -> f64 {
        let dt = (dt as f64).max(1e-4);

        let moved = match self.anchor {
            Some(anchor) if (value - anchor).abs() >= RATE_DEADBAND => {
                self.anchor = Some(value);
                (value - anchor).abs()
            }
            Some(_) => 0.0,
            None => {
                self.anchor = Some(value);
                0.0
            }
        };

        // Average the speed over a moment
        let smoothing = 1.0 - (-dt / RATE_SPEED_SMOOTHING).exp();
        self.speed += (moved / dt - self.speed) * smoothing;

        // Rise with the speed straight away; fall no faster than the wind-down
        let target = (self.speed * self.sensitivity / RATE_FULL_SPEED).clamp(0.0, 1.0);
        let wound_down = self.level * (-dt * self.wind_down / RATE_WIND_DOWN_PERIOD).exp();
        self.level = target.max(wound_down);
        if self.level < RATE_FLOOR {
            self.level = 0.0;
        }
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
        self.anchor = None;
        self.speed = 0.0;
        self.level = 0.0;
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
            rate_wind_down: 0.4,
            rate_sensitivity: 1.0,
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

    /// Feeds a triangle wave between 0 and 1 (full strokes) for `seconds`,
    /// with OSC-like updates every 30 ms on a 100 Hz tick. `strokes_per_second`
    /// full in-and-out strokes per second. Returns every level.
    fn stroke(p: &mut RateMode, strokes_per_second: f64, seconds: f64) -> Vec<f64> {
        let mut levels = Vec::new();
        let mut value = 0.0;
        let ticks = (seconds / TICK as f64).round() as usize;
        for tick in 0..ticks {
            let t = tick as f64 * TICK as f64;
            // A new value arrives every third tick, like OSC at ~33 Hz
            if tick % 3 == 0 {
                let phase = (t * strokes_per_second).fract();
                value = if phase < 0.5 {
                    phase * 2.0
                } else {
                    2.0 - phase * 2.0
                };
            }
            levels.push(p.rate(value, TICK));
        }
        levels
    }

    /// Mean level over the second half of a run (once it's settled)
    fn settled(levels: &[f64]) -> f64 {
        let half = &levels[levels.len() / 2..];
        half.iter().sum::<f64>() / half.len() as f64
    }

    #[test]
    fn rate_follows_stroke_speed() {
        // One full in-and-out stroke per second: 2 ranges/s, full power
        let fast = settled(&stroke(&mut RateMode::new(1.0, 0.4), 1.0, 3.0));
        // A quarter of that: about a quarter of the power
        let slow = settled(&stroke(&mut RateMode::new(1.0, 0.4), 0.25, 6.0));
        assert!(fast > 0.85, "fast {fast}");
        assert!((0.15..0.4).contains(&slow), "slow {slow}");
    }

    #[test]
    fn rate_sensitivity_scales_the_level() {
        let normal = settled(&stroke(&mut RateMode::new(1.0, 0.4), 0.25, 6.0));
        let doubled = settled(&stroke(&mut RateMode::new(2.0, 0.4), 0.25, 6.0));
        assert!(
            (doubled / normal - 2.0).abs() < 0.3,
            "{normal} -> {doubled}"
        );
    }

    #[test]
    fn rate_keeps_going_through_full_pull_outs() {
        // Every stroke reaches 0; the old Rate dropped to 0 each time
        let levels = stroke(&mut RateMode::new(1.0, 0.4), 0.5, 4.0);
        let lowest = levels[levels.len() / 2..]
            .iter()
            .cloned()
            .fold(1.0, f64::min);
        assert!(lowest > 0.3, "dipped to {lowest}");
    }

    #[test]
    fn rate_ignores_jitter() {
        let mut p = RateMode::new(1.0, 0.4);
        for i in 0..300 {
            let jitter = if i % 2 == 0 { 0.009 } else { -0.009 };
            assert_eq!(p.rate(0.5 + jitter, TICK), 0.0);
        }
    }

    #[test]
    fn rate_winds_down_smoothly_then_stops() {
        let mut p = RateMode::new(1.0, 0.4);
        stroke(&mut p, 1.0, 2.0);
        let start = p.level;
        // Held still where the stroke ended: falls every tick, never jumps,
        // and reaches 0
        let held = p.anchor.unwrap();
        let mut previous = start;
        let mut after_wind_down_period = 0.0;
        for tick in 1..=300 {
            let level = p.rate(held, TICK);
            assert!(level <= previous, "rose while still");
            assert!(previous - level < 0.1, "dropped too fast");
            if tick == 38 {
                // 0.15 s / 0.4 = 0.375 s: about a third left (1/e)
                after_wind_down_period = level / start;
            }
            previous = level;
        }
        assert!(
            (0.25..0.5).contains(&after_wind_down_period),
            "{after_wind_down_period}"
        );
        assert_eq!(previous, 0.0);
    }

    #[test]
    fn rate_first_value_and_reset_count_no_movement() {
        let mut p = RateMode::new(1.0, 0.4);
        // The first value is only a starting point, even far from 0
        assert_eq!(p.rate(0.9, TICK), 0.0);
        p.rate(0.5, TICK);
        assert!(p.level > 0.0);
        p.reset();
        assert_eq!(p.rate(0.1, TICK), 0.0);
    }

    #[test]
    fn rate_ignores_booleans() {
        let mut p = build(Mode::Rate, settings());
        assert_eq!(run(p.as_mut(), Signal::Bool(true)), None);
        assert_eq!(run(p.as_mut(), Signal::Float(0.3)), Some(0.0));
    }
}
