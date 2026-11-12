/*
 * SPS input processor. TODO: rewrite.
 *
 * SPS (https://vrcfury.com/sps/) sends a set of contact parameters per
 * socket or plug, under /avatar/parameters/OGB/<Type>/<Object>/<Leaf>, e.g.
 * /avatar/parameters/OGB/Orf/Mouth/PenOthers. The previous implementation
 * (based on OscGoesBrrr's algorithm: learn each penetrator's length from
 * root/tip samples, then level = 1 - (1 - root) / length) is in git history,
 * in toy_handling/input_processor/penetration_systems/sps/ as of commit fdefa56 (PR #73).
 *
 * Until this is written it reads the SPS parameters but never sets a level,
 * so a feature using SPS only responds to its own parameters and Idle.
 */

use vibecheck_core::Signal;

use crate::toy_handling::input_processor::InputProcessor;

#[derive(Default)]
pub struct SPSProcessor {}

impl InputProcessor for SPSProcessor {
    fn prefixes(&self) -> Vec<String> {
        vec!["/avatar/parameters/OGB/".into()]
    }

    fn update(&mut self, _addr: &str, _value: Signal) {
        // TODO: track the SPS parameters (per socket/plug key and leaf)
    }

    fn tick(&mut self, _dt: f32) -> Option<f64> {
        // TODO: return the level worked out from the tracked parameters
        None
    }

    // Uses the default input filter: an address passes if it contains any
    // tag, so a tag can be a socket's menu label
}
