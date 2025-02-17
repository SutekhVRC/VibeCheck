use log::{debug, trace};

use crate::toy_handling::toyops::RateProcessingValues;

pub enum RateParser {
    RateCalculated(f64, bool),
    SkipZero,
}

#[inline(always)]
pub fn parse_rate(
    processor: &mut RateProcessingValues,
    decrement_rate: f64,
    mut float_level: f64,
    flip_float: bool,
) -> RateParser {
    // Skip because got 0 value to stop toy.
    if !flip_float && float_level <= 0.0 || flip_float && float_level >= 1.0 {
        debug!("Bypassing rate input");
        processor.rate_saved_level = float_level;
        processor.rate_saved_osc_input = float_level;
        return RateParser::SkipZero;
    } else {
        // Increase toy level

        // Store new input then get the distance of the new input from the last input
        // Add that distance to the internal float level

        // get distance between newest input and last input
        // Set the distance between as the new motor speed
        if processor.rate_saved_osc_input > float_level {
            processor.rate_saved_level +=
                (processor.rate_saved_osc_input - float_level).clamp(0.00, 1.0);
        } else {
            processor.rate_saved_level +=
                (float_level - processor.rate_saved_osc_input).clamp(0.00, 1.0);
        }

        // Dont let internal level go over 1.0
        processor.rate_saved_level = processor.rate_saved_level.clamp(0.00, 1.00);

        // Set the newest input as the recent input
        processor.rate_saved_osc_input = float_level;

        // Set the internal rate state to the float level
        float_level = processor.rate_saved_level;

        // Save the internal motor speed
        //*rate_internal_level += *float_level;

        trace!("float level rate increased");
    }

    // Decrement testing
    if let Some(instant) = processor.rate_timestamp {
        // Decrease tick
        if instant.elapsed().as_secs_f64() >= 0.15 {
            // Decrease the internal rate level
            // This decrease rate should be tuneable
            processor.rate_saved_level =
                (processor.rate_saved_level - decrement_rate).clamp(0.00, 1.0);
            debug!(
                "internal level after decrement: {}",
                processor.rate_saved_level
            );

            // Set float level to decremented internal rate
            float_level = processor.rate_saved_level;

            trace!("decrease timer reset");
            return RateParser::RateCalculated(float_level, true);
        }
    }

    RateParser::RateCalculated(float_level, false)
}

