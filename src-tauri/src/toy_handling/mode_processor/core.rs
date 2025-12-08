use crate::toy_handling::toyops::LevelTweaks;
use crate::toy_handling::toyops::ProcessingModeValues;
use crate::toy_handling::toyops::ToyParameter;

use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;

use std::time::Instant;

use super::mode_rate::parse_rate;
use super::mode_rate::RateParser;
use super::mode_smooth::parse_smoothing;
use super::mode_smooth::SmoothParser;

pub enum ModeProcessorInput<'processor> {
    InputProcessor((ModeProcessorInputType, &'processor mut ProcessingModeValues)),
    RawInput(ModeProcessorInputType, &'processor mut ToyParameter),
}

#[derive(Debug, Clone, TS, Serialize, Deserialize, Copy)]
pub enum ModeProcessorInputType {
    Float(f64),
    Boolean(bool),
}

impl ModeProcessorInputType {
    pub fn try_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn try_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

pub async fn mode_processor(
    input: ModeProcessorInput<'_>,
    feature_levels: LevelTweaks,
    flip_input: bool,
) -> Option<f64> {
    // Parse if input is from an Input Processor or raw input

    match input {
        // Input is from an Input Processor
        ModeProcessorInput::InputProcessor((input_type, processing_mode_values)) => {
            match input_type {
                ModeProcessorInputType::Float(f_input) => {
                    // Input Processor & Float
                    mode_processor_logic(
                        ModeProcessorInputType::Float(f_input),
                        processing_mode_values,
                        feature_levels,
                        flip_input,
                    )
                    .await
                }
                ModeProcessorInputType::Boolean(b_input) => {
                    // Input Processor & Boolean
                    mode_processor_logic(
                        ModeProcessorInputType::Boolean(b_input),
                        processing_mode_values,
                        feature_levels,
                        flip_input,
                    )
                    .await
                } // Input Processor & Boolean
            }
        }
        // Input is from parameter parsing
        ModeProcessorInput::RawInput(input_type, toy_parameter) => {
            match input_type {
                ModeProcessorInputType::Float(f_input) => {
                    // Raw Input & Float
                    mode_processor_logic(
                        ModeProcessorInputType::Float(f_input),
                        &mut toy_parameter.processing_mode_values,
                        feature_levels,
                        flip_input,
                    )
                    .await
                }
                ModeProcessorInputType::Boolean(b_input) => {
                    // Raw Input & Boolean
                    mode_processor_logic(
                        ModeProcessorInputType::Boolean(b_input),
                        &mut toy_parameter.processing_mode_values,
                        feature_levels,
                        flip_input,
                    )
                    .await
                } // Raw Input & Boolean
            }
        }
    }
}

async fn mode_processor_logic(
    input: ModeProcessorInputType,
    processor: &mut ProcessingModeValues,
    feature_levels: LevelTweaks,
    flip_input: bool,
) -> Option<f64> {
    // Process logic for each mode processing type
    match processor {
        // Raw Mode Handling
        // Raw = mode processing so just return the original value
        ProcessingModeValues::Raw => match input {
            ModeProcessorInputType::Float(float_level) => Some(float_level),
            ModeProcessorInputType::Boolean(b) => {
                if b {
                    // True == 1.0
                    Some(1.0)
                } else {
                    //False == 0.0
                    Some(0.0)
                }
            }
        },
        // Smoothing Mode Handling
        // Smooth = do smoothing logic with input and processor
        ProcessingModeValues::Smooth(values) => {
            //trace!("parse_moothing()");

            match input {
                ModeProcessorInputType::Float(float_level) => {
                    match parse_smoothing(
                        &mut values.smooth_queue,
                        feature_levels,
                        float_level,
                        flip_input,
                    ) {
                        // If smooth parser calculates a smooth value or the input is 0 return it
                        SmoothParser::SkipZero(f_out) | SmoothParser::Smoothed(f_out) => {
                            Some(f_out)
                        }
                        // None so that we don't send the value to the device
                        // None because smoother is still smoothing
                        SmoothParser::Smoothing => None,
                    }
                }
                ModeProcessorInputType::Boolean(_b) => None, // No support for Smoothing mode and Boolean
            }
            // Return processed input
        }
        // Rate Mode Handling
        ProcessingModeValues::Rate(values) => {
            //trace!("parse_rate()");
            // Need to set rate_timestamp when feature enabled
            if values.rate_timestamp.is_none() {
                values.rate_timestamp = Some(Instant::now());
            }

            match input {
                ModeProcessorInputType::Float(float_level) => {
                    match parse_rate(values, feature_levels.rate_tune, float_level, flip_input) {
                        RateParser::SkipZero => Some(0.), // Skip zero and send to toy
                        RateParser::RateCalculated(f_out, reset_timer) => {
                            // Rate calculated reset timer and send calculated value to toy
                            if reset_timer {
                                values.rate_timestamp = Some(Instant::now())
                            }
                            Some(f_out)
                        }
                    }
                }
                ModeProcessorInputType::Boolean(_b) => None, // No support for Rate and Boolean
            }
        }
        // Constant Mode Handling
        ProcessingModeValues::Constant => match input {
            ModeProcessorInputType::Float(float_level) => {
                if float_level >= 0.5 {
                    Some(feature_levels.constant_level)
                } else {
                    Some(0.0)
                }
            }
            ModeProcessorInputType::Boolean(b) => {
                if b {
                    Some(feature_levels.constant_level)
                } else {
                    Some(0.0)
                }
            }
        },
    }
}
