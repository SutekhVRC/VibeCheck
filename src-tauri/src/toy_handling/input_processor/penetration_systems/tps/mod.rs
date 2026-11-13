/*
 * TPS input processor.
 *
 * TPS (https://www.poiyomi.com/TPS/wizard) sends its parameters under
 * /avatar/parameters/TPS_Internal/<Type>/<Id>/<Contact>:
 *
 * | <Type> | What it is  | Depth parameter |
 * |--------|-------------|-----------------|
 * | Orf    | orifice     | Depth_In        |
 * | Pen    | penetrator  | RootRoot        |
 *
 * Both are already a depth from 0 to 1, so they're used as they are. Each
 * orifice and penetrator (each <Id>) is tracked separately and the feature
 * follows the deepest one. Other TPS parameters are ignored.
 *
 * As before, either depth parameter is accepted whatever the <Type>, and
 * input filter tags don't apply to TPS (they're for SPS zones).
 */

use std::collections::HashMap;

use vibecheck_core::Signal;

use crate::toy_handling::input_processor::InputProcessor;

const PREFIX: &str = "/avatar/parameters/TPS_Internal/";
const DEPTH_PARAMETERS: [&str; 2] = ["Depth_In", "RootRoot"];

#[derive(Default)]
pub struct TPSProcessor {
    /// Latest depth per orifice or penetrator, keyed by `<Type>/<Id>`.
    depths: HashMap<String, f64>,
}

impl TPSProcessor {
    /// `<Type>/<Id>` for a TPS depth parameter, or `None` for any other
    /// address.
    fn depth_key(addr: &str) -> Option<&str> {
        let param = addr.strip_prefix(PREFIX)?;
        let (key, contact) = param.rsplit_once('/')?;
        let has_type_and_id = key
            .split_once('/')
            .is_some_and(|(t, id)| !t.is_empty() && !id.is_empty());
        (has_type_and_id && DEPTH_PARAMETERS.contains(&contact)).then_some(key)
    }
}

impl InputProcessor for TPSProcessor {
    fn prefixes(&self) -> Vec<String> {
        vec![PREFIX.to_owned()]
    }

    fn update(&mut self, addr: &str, value: Signal) {
        let Some(key) = Self::depth_key(addr) else {
            return;
        };
        if let Signal::Float(depth) = value {
            if depth.is_finite() {
                self.depths.insert(key.to_owned(), depth);
            }
        }
    }

    fn tick(&mut self, _dt: f32) -> Option<f64> {
        // Nothing until the first depth arrives, so Idle Speed doesn't start
        // on its own
        if self.depths.is_empty() {
            return None;
        }
        let deepest = self.depths.values().copied().fold(0.0, f64::max);
        Some(deepest.clamp(0.0, 1.0))
    }

    fn reset(&mut self) {
        self.depths.clear();
    }

    fn input_filter(&self, _addr: &str, _tags: &[String]) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn send(p: &mut TPSProcessor, param: &str, value: Signal) {
        p.update(&format!("{PREFIX}{param}"), value);
    }

    #[test]
    fn orifice_and_penetrator_depths_are_used_as_they_are() {
        let mut p = TPSProcessor::default();
        assert_eq!(p.tick(0.01), None, "nothing before the first depth");

        send(&mut p, "Orf/0/Depth_In", Signal::Float(0.4));
        assert_eq!(p.tick(0.01), Some(0.4));

        let mut p = TPSProcessor::default();
        send(&mut p, "Pen/0/RootRoot", Signal::Float(0.7));
        assert_eq!(p.tick(0.01), Some(0.7));
    }

    #[test]
    fn other_parameters_are_ignored() {
        let mut p = TPSProcessor::default();
        send(&mut p, "Orf/0/Depth_Out", Signal::Float(0.9));
        send(&mut p, "Orf/0/Depth_In", Signal::Bool(true));
        send(&mut p, "Orf/0/Depth_In", Signal::Float(f64::NAN));
        send(&mut p, "Depth_In", Signal::Float(0.9));
        send(&mut p, "Orf//Depth_In", Signal::Float(0.9));
        p.update("/avatar/parameters/OGB/Orf/0/Depth_In", Signal::Float(0.9));
        assert_eq!(p.tick(0.01), None);
    }

    #[test]
    fn the_deepest_of_several_follows_each_one() {
        let mut p = TPSProcessor::default();
        send(&mut p, "Orf/Mouth/Depth_In", Signal::Float(0.3));
        send(&mut p, "Orf/Hips/Depth_In", Signal::Float(0.8));
        assert_eq!(p.tick(0.01), Some(0.8));
        // Pulling out of one leaves the other
        send(&mut p, "Orf/Hips/Depth_In", Signal::Float(0.0));
        assert_eq!(p.tick(0.01), Some(0.3));
        // Over 1 is capped
        send(&mut p, "Orf/Mouth/Depth_In", Signal::Float(1.5));
        assert_eq!(p.tick(0.01), Some(1.0));
    }

    #[test]
    fn reset_forgets_every_depth() {
        let mut p = TPSProcessor::default();
        send(&mut p, "Orf/0/Depth_In", Signal::Float(0.5));
        p.reset();
        assert_eq!(p.tick(0.01), None);
    }

    #[test]
    fn input_filter_tags_do_not_apply() {
        let p = TPSProcessor::default();
        let tags = vec!["Mouth".to_owned()];
        assert!(p.input_filter(&format!("{PREFIX}Orf/0/Depth_In"), &tags));
    }
}
