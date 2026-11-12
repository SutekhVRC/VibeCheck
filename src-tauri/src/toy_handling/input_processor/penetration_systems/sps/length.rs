/*
 * Learns how long a plug is from a legacy SPS socket's root and tip contacts.
 *
 * A legacy socket reports how close the plug's root and tip are to it, from
 * 0 (out of range) to 1 (at the socket). The receivers are 1 m spheres, so
 * tip - root is the plug's length in metres. That only holds while both
 * readings are from the same moment and the tip is still outside: once the
 * tip is in, it reads 1 and the difference is only a lower bound.
 */

use std::collections::VecDeque;

/// Below this, the contact counts as out of range.
const OUT_OF_RANGE: f64 = 0.01;
/// A root this close means the plug is already deep in; not a useful reading.
const ROOT_TOO_CLOSE: f64 = 0.95;
/// Above this, the tip counts as inside the socket.
pub const TIP_INSIDE: f64 = 0.99;
/// Shorter than this is a broken or backwards reading.
const MIN_LENGTH: f64 = 0.02;
/// How many recent readings to keep.
const MAX_READINGS: usize = 8;
/// How many readings are needed before trusting them over the lower bound.
const MIN_READINGS: usize = 4;

#[derive(Debug, Default)]
pub struct LengthLearner {
    /// Readings taken with the tip outside, oldest first.
    readings: VecDeque<f64>,
    /// The longest reading taken with the tip already inside.
    lower_bound: Option<f64>,
}

impl LengthLearner {
    /// Takes one root/tip reading. `None` means the value isn't known.
    pub fn observe(&mut self, root: Option<f64>, tip: Option<f64>) {
        let (Some(root), Some(tip)) = (root, tip) else {
            self.forget();
            return;
        };
        if root < OUT_OF_RANGE || tip < OUT_OF_RANGE {
            // The plug left, and the next one may be a different length
            self.forget();
            return;
        }
        let length = tip - root;
        if root > ROOT_TOO_CLOSE || length < MIN_LENGTH {
            return;
        }
        if tip > TIP_INSIDE {
            self.lower_bound = Some(self.lower_bound.map_or(length, |b| b.max(length)));
        } else {
            self.readings.push_back(length);
            if self.readings.len() > MAX_READINGS {
                self.readings.pop_front();
            }
        }
    }

    /// The plug's length in metres, if known: the median of the recent
    /// readings once there are enough, otherwise the lower bound.
    ///
    /// The median ignores readings taken while only one of root or tip had
    /// updated, which are off in either direction.
    pub fn length(&self) -> Option<f64> {
        if self.readings.len() < MIN_READINGS {
            return self.lower_bound;
        }
        let mut sorted: Vec<f64> = self.readings.iter().copied().collect();
        sorted.sort_by(f64::total_cmp);
        let mid = sorted.len() / 2;
        Some(if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        })
    }

    pub fn forget(&mut self) {
        self.readings.clear();
        self.lower_bound = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Option<f64>, b: f64) -> bool {
        a.is_some_and(|a| (a - b).abs() < 1e-9)
    }

    #[test]
    fn needs_a_few_readings_and_ignores_odd_ones() {
        let mut l = LengthLearner::default();
        l.observe(Some(0.3), Some(0.5));
        l.observe(Some(0.4), Some(0.6));
        l.observe(Some(0.5), Some(0.7));
        assert_eq!(l.length(), None, "three readings aren't enough");

        // One reading where only the tip had updated
        l.observe(Some(0.5), Some(0.9));
        l.observe(Some(0.6), Some(0.8));
        assert!(close(l.length(), 0.2), "{:?}", l.length());
    }

    #[test]
    fn readings_with_the_tip_inside_are_only_a_fallback() {
        let mut l = LengthLearner::default();
        l.observe(Some(0.85), Some(1.0));
        l.observe(Some(0.9), Some(1.0));
        assert!(close(l.length(), 0.15), "the longest one counts");

        for root in [0.3, 0.35, 0.4, 0.45] {
            l.observe(Some(root), Some(root + 0.25));
        }
        assert!(close(l.length(), 0.25), "real readings replace it");
    }

    #[test]
    fn unusable_readings_are_skipped_and_leaving_forgets() {
        let mut l = LengthLearner::default();
        l.observe(Some(0.8), Some(1.0));
        l.observe(Some(0.97), Some(1.0)); // root too close
        l.observe(Some(0.5), Some(0.51)); // too short
        assert!(close(l.length(), 0.2));

        l.observe(Some(0.0), Some(0.3));
        assert_eq!(l.length(), None);
        l.observe(Some(0.8), Some(1.0));
        l.observe(None, Some(1.0));
        assert_eq!(l.length(), None);
    }
}
