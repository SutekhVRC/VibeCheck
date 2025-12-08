use log::debug;

use crate::toy_handling::toyops::LevelTweaks;

pub enum SmoothParser {
    Smoothed(f64),
    SkipZero(f64),
    Smoothing,
}

#[inline(always)]
pub fn parse_smoothing(
    smooth_queue: &mut Vec<f64>,
    feature_levels: LevelTweaks,
    mut float_level: f64,
    flip_float: bool,
) -> SmoothParser {
    debug!("!flip_float && *float_level == 0.0: [{}] || [{}] flip_float && *float_level == 1.0\nCOMBINED: [{}]", !flip_float && float_level == 0.0, flip_float && float_level == 1.0,
    smooth_queue.len() == feature_levels.smooth_rate as usize && (!flip_float && float_level == 0.0 || flip_float && float_level == 1.0)
);
    // Reached smooth rate maximum and not a 0 value
    if smooth_queue.len() == feature_levels.smooth_rate as usize {
        debug!("smooth_queue: {}", smooth_queue.len());
        if !flip_float && float_level == 0.0 || flip_float && float_level == 1.0 {
            // Don't return just set to 0
            debug!("float level is 0 but will be forgotten!");

            // Clear smooth queue bc restarting from 0
            smooth_queue.clear();
        } else {
            debug!("Setting float_level with smoothed float");
            // Get Mean of all numbers in smoothing rate and then round to hundredths and cast as f64
            float_level = (smooth_queue.iter().sum::<f64>() / smooth_queue.len() as f64 * 100.0)
                .round()
                / 100.0;
            smooth_queue.clear();

            smooth_queue.push(float_level);
            return SmoothParser::Smoothed(float_level);
        }

        // Have not reached smoothing maximum
    }

    // Maybe move this to be before queue is full check?
    if !flip_float && float_level == 0.0 || flip_float && float_level == 1.0 {
        debug!("Bypassing smoother: {:.5}", float_level);
        // let 0 through
        return SmoothParser::SkipZero(float_level);
    }

    debug!(
        "Adding float {} to smoothing.. queue size: {}",
        float_level,
        smooth_queue.len()
    );
    smooth_queue.push(float_level);
    // Continue receiving smooth floats
    SmoothParser::Smoothing
}
