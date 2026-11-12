/*
 * TPS input processor. TODO: rewrite.
 *
 * TPS (https://www.poiyomi.com/TPS/wizard) sends its parameters under
 * /avatar/parameters/TPS_Internal/. The previous implementation used the
 * value of any parameter ending in Depth_In or RootRoot as the level, and
 * ignored input filter tags. It's in git history, in
 * toy_handling/input_processor/penetration_systems/tps/ as of commit fdefa56 (PR #73).
 *
 * Until this is written it reads the TPS parameters but never sets a level,
 * so a feature using TPS only responds to its own parameters and Idle.
 */

use vibecheck_core::Signal;

use crate::toy_handling::input_processor::InputProcessor;

#[derive(Default)]
pub struct TPSProcessor {}

impl InputProcessor for TPSProcessor {
    fn prefixes(&self) -> Vec<String> {
        vec!["/avatar/parameters/TPS_Internal/".into()]
    }

    fn update(&mut self, _addr: &str, _value: Signal) {
        // TODO: track the TPS parameters
    }

    fn tick(&mut self, _dt: f32) -> Option<f64> {
        // TODO: return the level worked out from the tracked parameters
        None
    }

    // TODO: decide whether TPS uses input filter tags. The previous
    // version ignored them (override input_filter to return true)
}
